//! Carga de una fuente matemática instalada en el sistema.
//!
//! Si no hay ninguna fuente con tabla OpenType `MATH` disponible, el render de
//! LaTeX cae a mostrar el código fuente como texto.

use std::sync::{Arc, OnceLock};

/// Variable de entorno para forzar una fuente concreta (ruta a un .ttf/.otf).
const ENV_OVERRIDE: &str = "MARMOL_MATH_FONT";

/// Nombre de la familia con la que registramos la fuente en egui.
pub const EGUI_FAMILY: &str = "marmol_math";

/// Familias habituales de fuentes matemáticas, en orden de preferencia.
#[cfg(not(target_os = "android"))]
const CANDIDATES: &[&str] = &[
    "Cambria Math",
    "Latin Modern Math",
    "STIX Two Math",
    "STIX Math",
    "XITS Math",
    "TeX Gyre Termes Math",
    "TeX Gyre Pagella Math",
    "TeX Gyre Bonum Math",
    "DejaVu Math TeX Gyre",
    "Libertinus Math",
    "Noto Sans Math",
];

struct LoadedFont {
    data: Vec<u8>,
    #[allow(dead_code)]
    name: String,
}

fn loaded() -> Option<&'static LoadedFont> {
    static FONT: OnceLock<Option<LoadedFont>> = OnceLock::new();
    FONT.get_or_init(load).as_ref()
}

/// Bytes de la fuente matemática del sistema, o `None` si no hay ninguna.
pub fn system_math_font_data() -> Option<&'static [u8]> {
    loaded().map(|font| font.data.as_slice())
}

/// Registra la fuente matemática del sistema en las definiciones de egui.
/// Devuelve `true` si se añadió.
pub fn install_in_font_definitions(fonts: &mut egui::FontDefinitions) -> bool {
    let Some(data) = system_math_font_data() else {
        return false;
    };
    let key = EGUI_FAMILY.to_string();
    fonts.font_data.insert(
        key.clone(),
        Arc::new(egui::FontData::from_owned(data.to_vec())),
    );
    // La fuente matemática primero y la proporcional como fallback para
    // símbolos que no estén en la primera (p. ej. ✓ o ■).
    let mut chain = vec![key];
    if let Some(defaults) = fonts
        .families
        .get(&egui::FontFamily::Proportional)
        .cloned()
    {
        chain.extend(defaults);
    }
    fonts
        .families
        .insert(egui::FontFamily::Name(EGUI_FAMILY.into()), chain);
    true
}

fn load() -> Option<LoadedFont> {
    if let Some(font) = load_override() {
        return Some(font);
    }

    #[cfg(not(target_os = "android"))]
    {
        use font_loader::system_fonts::{self, FontPropertyBuilder};

        for family in CANDIDATES {
            let property = FontPropertyBuilder::new().family(*family).build();
            if let Some((data, _)) = system_fonts::get(&property) {
                if has_math_table(&data) {
                    println!("Marmol: usando fuente matemática del sistema: {family}");
                    return Some(LoadedFont {
                        data,
                        name: (*family).to_string(),
                    });
                }
            }
        }
    }

    #[cfg(target_os = "android")]
    {
        for (name, data) in android_font_candidates() {
            if has_math_table(&data) {
                println!("Marmol: usando fuente matemática del sistema: {name}");
                return Some(LoadedFont { data, name });
            }
        }
    }

    eprintln!(
        "Marmol: no hay ninguna fuente matemática instalada; el LaTeX se mostrará como texto."
    );
    None
}

fn load_override() -> Option<LoadedFont> {
    let path = std::env::var(ENV_OVERRIDE).ok()?;
    match std::fs::read(&path) {
        Ok(data) if has_math_table(&data) => {
            println!("Marmol: usando fuente matemática de {ENV_OVERRIDE}: {path}");
            Some(LoadedFont {
                data,
                name: path,
            })
        }
        Ok(_) => {
            eprintln!("Marmol: {ENV_OVERRIDE} no contiene una tabla MATH: {path}");
            None
        }
        Err(err) => {
            eprintln!("Marmol: no se pudo leer {ENV_OVERRIDE}={path}: {err}");
            None
        }
    }
}

#[cfg(target_os = "android")]
fn android_font_candidates() -> Vec<(String, Vec<u8>)> {
    const DIRS: &[&str] = &["/system/fonts", "/product/fonts", "/system/product/fonts"];
    let mut found = Vec::new();
    for dir in DIRS {
        let Ok(entries) = std::fs::read_dir(dir) else {
            continue;
        };
        for entry in entries.flatten() {
            let path = entry.path();
            let name = path.file_name().and_then(|n| n.to_str()).unwrap_or("");
            let lower = name.to_lowercase();
            let is_font = lower.ends_with(".ttf") || lower.ends_with(".otf");
            if is_font && lower.contains("math") {
                if let Ok(data) = std::fs::read(&path) {
                    found.push((path.to_string_lossy().to_string(), data));
                }
            }
        }
    }
    found
}

/// Comprueba la tabla OpenType `MATH` para no aceptar una sustitución de
/// fontconfig que no sea una fuente matemática.
fn has_math_table(data: &[u8]) -> bool {
    if data.len() < 12 {
        return false;
    }

    // Colección de fuentes (TTC): probar cada fuente de la colección.
    if &data[0..4] == b"ttcf" {
        let count = u32::from_be_bytes([data[8], data[9], data[10], data[11]]) as usize;
        for i in 0..count.min(32) {
            let pos = 12 + i * 4;
            let Some(bytes) = data.get(pos..pos + 4) else {
                break;
            };
            let offset = u32::from_be_bytes([bytes[0], bytes[1], bytes[2], bytes[3]]) as usize;
            if has_math_table_at(data, offset) {
                return true;
            }
        }
        return false;
    }

    has_math_table_at(data, 0)
}

fn has_math_table_at(data: &[u8], offset: usize) -> bool {
    let Some(header) = data.get(offset..offset + 12) else {
        return false;
    };
    let num_tables = u16::from_be_bytes([header[4], header[5]]) as usize;
    for i in 0..num_tables {
        let record = offset + 12 + i * 16;
        let Some(tag) = data.get(record..record + 4) else {
            return false;
        };
        if tag == b"MATH" {
            return true;
        }
    }
    false
}
