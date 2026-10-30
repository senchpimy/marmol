//! Carga de una fuente matemática instalada en el sistema.
//!
//! Sustituye a la fuente que antes se incrustaba en el binario. Si no hay
//! ninguna fuente con tabla OpenType `MATH` disponible, el render de LaTeX
//! cae a mostrar el código fuente como texto.

use std::sync::OnceLock;
use typst::foundations::Bytes;
use typst::text::Font;

/// Variable de entorno para forzar una fuente concreta (ruta a un .ttf/.otf).
const ENV_OVERRIDE: &str = "MARMOL_MATH_FONT";

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

/// Devuelve la fuente matemática del sistema ya parseada, o `None` si no hay
/// ninguna instalada. Se resuelve una sola vez por proceso.
pub fn system_math_font() -> Option<&'static Font> {
    static FONT: OnceLock<Option<Font>> = OnceLock::new();
    FONT.get_or_init(load).as_ref()
}

fn load() -> Option<Font> {
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
                    if let Some(font) = parse(data) {
                        println!("Marmol: usando fuente matemática del sistema: {family}");
                        return Some(font);
                    }
                }
            }
        }
    }

    #[cfg(target_os = "android")]
    {
        for (name, data) in android_font_candidates() {
            if has_math_table(&data) {
                if let Some(font) = parse(data) {
                    println!("Marmol: usando fuente matemática del sistema: {name}");
                    return Some(font);
                }
            }
        }
    }

    eprintln!(
        "Marmol: no hay ninguna fuente matemática instalada; el LaTeX se mostrará como texto."
    );
    None
}

fn load_override() -> Option<Font> {
    let path = std::env::var(ENV_OVERRIDE).ok()?;
    match std::fs::read(&path) {
        Ok(data) if has_math_table(&data) => match parse(data) {
            Some(font) => {
                println!("Marmol: usando fuente matemática de {ENV_OVERRIDE}: {path}");
                Some(font)
            }
            None => {
                eprintln!("Marmol: {ENV_OVERRIDE} no apunta a una fuente válida: {path}");
                None
            }
        },
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

fn parse(data: Vec<u8>) -> Option<Font> {
    if data.is_empty() {
        return None;
    }
    Font::new(Bytes::from(data), 0)
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
