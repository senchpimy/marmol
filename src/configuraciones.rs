use crate::screens;
use crate::tabs::Tabe;

use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::path::Path;
use yaml_rust::YamlLoader;

#[allow(dead_code)]
pub struct VaultConfig {
    graph_json_config: String,
}

#[derive(Serialize, Deserialize, PartialEq, Copy, Clone)]
pub enum AndroidStorage {
    Unselected,
    Internal,
    System,
}

#[derive(Serialize, Deserialize)]
pub struct MarmolProgramState {
    pub vault: String,
    pub vault_vec: Vec<String>,
    pub current_file: Option<String>,
    pub initial_screen: screens::Screen,
    pub collapsed_left: bool,
    pub center_size: f32,
    pub sort_files: bool,
    pub dock_state: crate::egui_dock::DockState<Tabe>,
    #[serde(default)]
    pub enable_icon_folder: bool,
    #[serde(default)]
    pub android_storage: Option<AndroidStorage>,
}

impl Default for MarmolProgramState {
    fn default() -> Self {
        let default_vault = String::new();
        let default_vault_vec = vec![];
        let default_current = None;
        let default_collpased_left = true;
        let default_center_size = 0.8;
        let default_sort_files = false;
        let default_window = screens::Screen::Default;
        let default_dock_state = crate::egui_dock::DockState::new(vec![]);
        let default_enable_icon_folder = false;

        Self {
            vault: default_vault,
            vault_vec: default_vault_vec,
            current_file: default_current,
            initial_screen: default_window,
            collapsed_left: default_collpased_left,
            center_size: default_center_size,
            sort_files: default_sort_files,
            dock_state: default_dock_state,
            enable_icon_folder: default_enable_icon_folder,
            android_storage: Some(AndroidStorage::Unselected),
        }
    }
}

pub fn get_config_dir() -> String {
    #[cfg(target_os = "android")]
    {
        std::env::var("MARMOL_DATA_DIR").unwrap_or_else(|_| "/data/local/tmp/marmol".to_string())
    }
    #[cfg(not(target_os = "android"))]
    {
        use directories::BaseDirs;
        if let Some(binding) = BaseDirs::new() {
            let home_dir = binding.home_dir().to_str().unwrap_or(".");
            format!("{}/.config/marmol", home_dir)
        } else {
            "./.config/marmol".to_string()
        }
    }
}

pub fn get_external_dir() -> String {
    #[cfg(target_os = "android")]
    {
        "/storage/emulated/0/Documents/Marmol".to_string()
    }
    #[cfg(not(target_os = "android"))]
    {
        get_config_dir()
    }
}

pub fn load_program_state() -> (MarmolProgramState, String) {
    let config_dir_path = get_config_dir();
    let config_file_path = format!("{}/ProgramState", config_dir_path);

    println!("Config path: {}", config_file_path);

    if Path::new(&config_file_path).exists() {
        println!("Fichero de configuración encontrado. Cargando estado...");

        match fs::read_to_string(&config_file_path) {
            Ok(data) => {
                match serde_yaml::from_str(&data) {
                    Ok(state) => {
                        println!("Estado cargado correctamente.");
                        (state, config_dir_path)
                    }
                    Err(e) => {
                        eprintln!("Error deserializando ProgramState: {}. Creando uno por defecto.", e);
                        (MarmolProgramState::default(), config_dir_path)
                    }
                }
            }
            Err(e) => {
                eprintln!("No se pudo leer el archivo de estado: {}. Usando por defecto.", e);
                (MarmolProgramState::default(), config_dir_path)
            }
        }
    } else {
        println!("Fichero de configuración no encontrado en {}. Creando uno por defecto...", config_file_path);
        create_default_vault(&config_dir_path)
    }
}

fn create_default_vault(config_dir: &str) -> (MarmolProgramState, String) {
    let default_state = MarmolProgramState::default();
    (default_state, config_dir.to_string())
}

pub fn save_program_state(state: &MarmolProgramState) {
    let config_dir_path = get_config_dir();
    let config_file_path = format!("{}/ProgramState", config_dir_path);

    println!("Saving program state to {}...", config_file_path);

    if !Path::new(&config_dir_path).exists() {
        if let Err(e) = fs::create_dir_all(&config_dir_path) {
            eprintln!("Could not create config directory {}: {}", config_dir_path, e);
            return;
        }
    }

    match serde_yaml::to_string(state) {
        Ok(serialized_state) => {
            if let Err(e) = fs::write(&config_file_path, serialized_state) {
                eprintln!("Could not write program state to {}: {}", config_file_path, e);
            } else {
                println!("Program state saved successfully.");
            }
        }
        Err(e) => {
            eprintln!("Could not serialize program state: {}", e);
        }
    }
}

pub fn load_context() -> f32 {
    let config_path_var = get_config_dir();
    let dir = Path::new(&config_path_var);

    let mut font_size: f32 = 12.0;
    if dir.exists() {
        let file_saved = format!("{}{}", &config_path_var, "/ContextState");
        let dir2 = Path::new(&file_saved);
        if dir2.exists() {
            if let Ok(data) = fs::read_to_string(file_saved) {
                if let Ok(docs) = YamlLoader::load_from_str(&data) {
                    if !docs.is_empty() {
                        let docs = &docs[0];
                        font_size = docs["font_size"].as_i64().unwrap_or(12) as f32;
                    }
                }
            }
        }
    }
    return font_size;
}