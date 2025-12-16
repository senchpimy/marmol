use crate::screens;
use crate::tabs::Tabe;
use directories::BaseDirs;

use serde::{Deserialize, Serialize};
use serde_yaml;
use std::fs;
use std::path::Path;
use yaml_rust::YamlLoader;

#[allow(dead_code)]
pub struct VaultConfig {
    graph_json_config: String,
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
    pub dock_state: egui_dock::DockState<Tabe>,
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
        let default_dock_state = egui_dock::DockState::new(vec![]);

        Self {
            vault: default_vault,
            vault_vec: default_vault_vec,
            current_file: default_current,
            initial_screen: default_window,
            collapsed_left: default_collpased_left,
            center_size: default_center_size,
            sort_files: default_sort_files,
            dock_state: default_dock_state,
        }
    }
}

pub fn load_program_state() -> (
    String,
    Vec<String>,
    Option<String>,
    String,
    screens::Screen,
    bool,
    f32,
    bool,
    egui_dock::DockState<Tabe>,
) {
    let binding = BaseDirs::new().unwrap();
    let home_dir = binding.home_dir().to_str().unwrap();

    let config_dir_path = format!("{}/.config/marmol", home_dir);
    let config_file_path = format!("{}/ProgramState", config_dir_path);

    if Path::new(&config_file_path).exists() {
        println!("Fichero de configuración encontrado. Cargando estado...");

        let data =
            fs::read_to_string(&config_file_path).expect("No se pudo leer el archivo de estado");

        let state: MarmolProgramState = serde_yaml::from_str(&data).unwrap_or_else(|e| {
            eprintln!("Error deserializing ProgramState: {}. Creating default.", e);
            MarmolProgramState::default()
        });

        (
            state.vault,
            state.vault_vec,
            state.current_file,
            config_dir_path,
            state.initial_screen,
            state.collapsed_left,
            state.center_size,
            state.sort_files,
            state.dock_state,
        )
    } else {
        println!("Fichero de configuración no encontrado. Creando uno por defecto...");
        create_default_vault(&config_dir_path)
    }
}

fn create_default_vault(
    config_dir: &str,
) -> (
    String,
    Vec<String>,
    Option<String>,
    String,
    screens::Screen,
    bool,
    f32,
    bool,
    egui_dock::DockState<Tabe>,
) {
    let default_state = MarmolProgramState::default();

    (
        default_state.vault,
        default_state.vault_vec,
        default_state.current_file,
        config_dir.to_string(),
        default_state.initial_screen,
        default_state.collapsed_left,
        default_state.center_size,
        default_state.sort_files,
        default_state.dock_state,
    )
}

pub fn save_program_state(state: &MarmolProgramState) {
    let binding = BaseDirs::new().unwrap();
    let home_dir = binding.home_dir().to_str().unwrap();
    let config_dir_path = format!("{}/.config/marmol", home_dir);
    let config_file_path = format!("{}/ProgramState", config_dir_path);

    if !Path::new(&config_dir_path).exists() {
        fs::create_dir_all(&config_dir_path).expect("Could not create config directory");
    }

    let serialized_state =
        serde_yaml::to_string(state).expect("Could not serialize program state to YAML");

    fs::write(&config_file_path, serialized_state).expect("Could not write program state to file");

    println!("Program state saved to {}", config_file_path);
}

pub fn load_context() -> f32 {
    let binding = BaseDirs::new().unwrap();
    let home_dir = binding.home_dir().to_str().unwrap();
    let config_path_var = format!("{}{}", &home_dir, "/.config/marmol");
    let dir = Path::new(&config_path_var);

    let mut font_size: f32 = 12.0;
    if dir.exists() {
        let file_saved = format!("{}{}", &config_path_var, "/ContextState");
        let dir2 = Path::new(&file_saved);
        if dir2.exists() {
            let data = fs::read_to_string(file_saved).expect("Unable to read file");
            let docs = YamlLoader::load_from_str(&data).unwrap();
            let docs = &docs[0];
            font_size = docs["font_size"].as_i64().unwrap_or(12) as f32;
        }
    }
    return font_size;
}
