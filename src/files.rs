use std::fs::File;
use std::io::Read;
use std::path::Path;
use walkdir::WalkDir;

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

pub fn contents(contents: &String) -> (String, String) {
    let metadata = String::new();
    if contents.starts_with("---") {
        let test = contents.splitn(3, "---");
        let test: Vec<&str> = test.collect();
        return (test[2].to_string(), test[1].to_string());
    } else {
        return (contents.to_string(), metadata);
    }
}

pub fn resolve_path(vault: &str, current_file_path: &str, target: &str) -> Option<String> {
    let clean_target = target.trim();
    if clean_target.is_empty() {
        return None;
    }

    // 1. Ruta absoluta dentro del vault (si empieza con /)
    if clean_target.starts_with('/') {
        let p = format!("{}{}", vault, clean_target);
        if Path::new(&p).exists() {
            return Some(p);
        }
    }

    // 2. Ruta relativa al archivo actual
    if let Some(current_dir) = Path::new(current_file_path).parent() {
        let joined = current_dir.join(clean_target);
        if joined.exists() && joined.is_file() {
            return Some(joined.to_string_lossy().to_string());
        }

        // Probar con extensiones comunes relativas
        for ext in &[".md", ".excalidraw.md"] {
            if !clean_target.to_lowercase().ends_with(ext) {
                let joined_ext = current_dir.join(format!("{}{}", clean_target, ext));
                if joined_ext.exists() && joined_ext.is_file() {
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
    let targets_names: Vec<String> = targets.iter().map(|t| {
        Path::new(t).file_name().unwrap_or_default().to_string_lossy().to_string()
    }).collect();

    for entry in WalkDir::new(vault).into_iter().filter_map(|e| e.ok()) {
        if entry.file_type().is_file() {
            if let Some(fname) = entry.file_name().to_str() {
                let fname_lower = fname.to_lowercase();
                for t in &targets_names {
                    if fname_lower == *t {
                        return Some(entry.path().to_string_lossy().to_string());
                    }
                }
            }
        }
    }

    None
}
