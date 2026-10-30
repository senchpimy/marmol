use json::{object, JsonValue};
use std::collections::HashMap;
use std::fs;
use std::fs::File;
use std::io::Read;
use std::io::Write;
use std::path::Path;
use std::sync::{Arc, Mutex, OnceLock};
use std::time::SystemTime;
use walkdir::WalkDir;

fn content_cache() -> &'static Mutex<HashMap<String, (SystemTime, Arc<str>)>> {
    static CACHE: OnceLock<Mutex<HashMap<String, (SystemTime, Arc<str>)>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

/// Devuelve la fecha de modificación de un archivo (None si no existe o falla el stat).
pub fn file_modified(file_name: &str) -> Option<SystemTime> {
    fs::metadata(file_name).and_then(|m| m.modified()).ok()
}

/// Lee un archivo pero solo lo vuelve a leer de disco si su fecha de modificación cambió.
/// Actúa como un "watcher" ligero basado en mtime: el I/O de disco solo ocurre cuando
/// el archivo fue modificado realmente, no en cada frame.
pub fn read_file_cached_arc(file_name: &str) -> Arc<str> {
    let modified = file_modified(file_name);
    if let Some(m) = modified {
        let cache = content_cache();
        {
            let guard = cache.lock().unwrap();
            if let Some((cached_m, content)) = guard.get(file_name) {
                if *cached_m == m {
                    return content.clone();
                }
            }
        }
        let content: Arc<str> = Arc::from(read_file(file_name));
        let mut guard = cache.lock().unwrap();
        // Caché acotada: al abrir muchos archivos no crece indefinidamente.
        if guard.len() >= 256 && !guard.contains_key(file_name) {
            guard.clear();
        }
        guard.insert(file_name.to_string(), (m, content.clone()));
        drop(guard);
        content
    } else {
        Arc::from(read_file(file_name))
    }
}

/// Igual que [`read_file_cached_arc`] pero devuelve un `String`. Preferir la
/// versión `Arc` en rutas calientes (render por frame) para no clonar el
/// documento completo en cada frame.
pub fn read_file_cached(file_name: &str) -> String {
    read_file_cached_arc(file_name).to_string()
}

/// Devuelve la ruta canónica de un archivo, cacheando el resultado (evita
/// syscalls `canonicalize` repetidas en cada frame).
pub fn canonicalize_cached(path: &str) -> String {
    static CACHE: OnceLock<Mutex<HashMap<String, String>>> = OnceLock::new();
    let cache = CACHE.get_or_init(|| Mutex::new(HashMap::new()));
    if let Some(c) = cache.lock().unwrap().get(path) {
        return c.clone();
    }
    let canon = Path::new(path)
        .canonicalize()
        .map(|p| p.to_string_lossy().to_string())
        .unwrap_or_else(|_| path.to_string());
    cache
        .lock()
        .unwrap()
        .insert(path.to_string(), canon.clone());
    canon
}

pub fn delete_file(trgt: &str) -> bool {
    //let path_to_delete = trgt.to_string();
    let delete = fs::remove_file(trgt);
    dbg!("Eliminado archivo:");
    dbg!(trgt);
    match delete {
        Ok(_) => {
            true
            // Solo es necesario para mandar la senal de eliminado pero desaparece cuando se lee el
            // dir otra vez
            // *error = String::new();
            // ui.data_mut(|d| d.remove_temp::<bool>(id));
            // ui.data_mut(|d| {
            //     d.insert_temp(egui::Id::new("file_deleted_signal"), Some(path_to_delete))
            // });
            // ui.close();
        }
        Err(r) => {
            println!("Error eliminando {}: {}", trgt, r);
            false
        }
    }
}

pub fn delete_folder(trgt: &str) {
    let delete = fs::remove_dir_all(trgt);
    match delete {
        Ok(_) => {}
        Err(r) => {
            println!("Error eliminando folder {}: {}", trgt, r)
        }
    }
}

pub fn copy_file(origin: &str, dest: &str) {
    let s_copy = Path::new(&dest);
    let copy = fs::copy(origin, &s_copy);
    match copy {
        Ok(_) => {}
        Err(r) => println!("Error Copiando {}: {}", dest, r),
    }
}

pub fn add_starred(vault: &str, file: &str) {
    let stared_path = format!("{}/.obsidian/starred.json", vault);
    let nw_json = object! {
        "type":"file",
        "title":Path::new(file).file_stem().unwrap().to_str().unwrap(),
        "path":"testi" //TODO Cambiar esto al valor correcto
    };
    if Path::new(&stared_path).exists() {
        let contents =
            fs::read_to_string(&stared_path).expect("Should have been able to read the file");
        let mut parsed = json::parse(&contents).unwrap();
        let json_arr: &mut JsonValue = &mut parsed["items"];
        json_arr.push(nw_json).unwrap();
        let mut f = std::fs::OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(stared_path)
            .unwrap();
        f.write_all(parsed.pretty(2).as_bytes()).unwrap();
        f.flush().unwrap();
    } else {
        let n_file = File::create(stared_path);
        match n_file {
            Ok(mut w) => {
                let text = format!("{{ items:[{}] }}", nw_json.dump());
                match w.write(text.as_bytes()) {
                    Ok(_) => {}
                    Err(r) => println!("Error Star {}: {}", file, r),
                }
            }
            Err(r) => println!("Error Star {}: {}", file, r),
        }
    }
}

pub fn read_file(file_name: &str) -> String {
    let file = File::open(file_name);
    let mut contents = String::new();
    match file {
        Ok(mut t) => match t.read_to_string(&mut contents) {
            Ok(_) => {}
            Err(e) => return e.to_string(),
        },
        Err(e) => {
            contents = format!("Error Reading File\n\n {}", e.to_string());
        }
    }
    contents
}

/// Separa frontmatter y cuerpo sin asignar memoria.
pub fn contents_borrowed(contents: &str) -> (&str, &str) {
    if let Some(rest) = contents.strip_prefix("---") {
        if let Some((metadata, body)) = rest.split_once("---") {
            return (body, metadata);
        }
    }
    (contents, "")
}

/// Versión que asigna memoria. En rutas calientes usar [`contents_borrowed`].
pub fn contents(contents: &str) -> (String, String) {
    let (body, metadata) = contents_borrowed(contents);
    (body.to_string(), metadata.to_string())
}

pub fn resolve_path(vault: &str, current_file_path: &str, target: &str) -> Option<String> {
    let clean_target = target.trim();
    if clean_target.is_empty() {
        return None;
    }

    // 1. Ruta absoluta dentro del vault (si empieza con /)
    if clean_target.starts_with('/') {
        let p = format!("{}{}", vault, clean_target);
        let path = Path::new(&p);
        if path.exists() {
            if let Ok(abs) = path.canonicalize() {
                return Some(abs.to_string_lossy().to_string());
            }
            return Some(p);
        }
    }

    // 2. Ruta relativa al archivo actual
    if let Some(current_dir) = Path::new(current_file_path).parent() {
        let joined = current_dir.join(clean_target);
        if joined.exists() && joined.is_file() {
            if let Ok(abs) = joined.canonicalize() {
                return Some(abs.to_string_lossy().to_string());
            }
            return Some(joined.to_string_lossy().to_string());
        }

        // Probar con extensiones comunes relativas
        for ext in &[".md", ".excalidraw.md"] {
            if !clean_target.to_lowercase().ends_with(ext) {
                let joined_ext = current_dir.join(format!("{}{}", clean_target, ext));
                if joined_ext.exists() && joined_ext.is_file() {
                    if let Ok(abs) = joined_ext.canonicalize() {
                        return Some(abs.to_string_lossy().to_string());
                    }
                    return Some(joined_ext.to_string_lossy().to_string());
                }
            }
        }
    }

    // 2b. Ruta relativa a la raíz del vault (enlaces tipo `Carpeta/Nota`).
    if !clean_target.starts_with('/') {
        let base = Path::new(vault);
        let joined = base.join(clean_target);
        if joined.exists() && joined.is_file() {
            if let Ok(abs) = joined.canonicalize() {
                return Some(abs.to_string_lossy().to_string());
            }
            return Some(joined.to_string_lossy().to_string());
        }
        for ext in &[".md", ".excalidraw.md"] {
            if !clean_target.to_lowercase().ends_with(ext) {
                let joined_ext = base.join(format!("{}{}", clean_target, ext));
                if joined_ext.exists() && joined_ext.is_file() {
                    if let Ok(abs) = joined_ext.canonicalize() {
                        return Some(abs.to_string_lossy().to_string());
                    }
                    return Some(joined_ext.to_string_lossy().to_string());
                }
            }
        }
    }

    // 3. Búsqueda global en el vault (por nombre de archivo, case-insensitive)
    let mut targets = vec![clean_target.to_lowercase()];
    if !clean_target.to_lowercase().ends_with(".md") {
        targets.push(format!("{}.md", clean_target.to_lowercase()));
    }
    if !clean_target.to_lowercase().ends_with(".excalidraw.md") {
        targets.push(format!("{}.excalidraw.md", clean_target.to_lowercase()));
    }

    // Normalizar targets para comparación (solo nombre de archivo en minúsculas)
    let targets_names: Vec<String> = targets
        .iter()
        .map(|t| {
            Path::new(t)
                .file_name()
                .unwrap_or_default()
                .to_string_lossy()
                .to_string()
        })
        .collect();

    for entry in WalkDir::new(vault).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(fname) = entry.file_name().to_str() {
                let fname_lower = fname.to_lowercase();
                for t in &targets_names {
                    if fname_lower == *t {
                        if let Ok(abs) = entry.path().canonicalize() {
                            return Some(abs.to_string_lossy().to_string());
                        }
                        return Some(entry.path().to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    None
}
