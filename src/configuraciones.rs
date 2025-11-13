use crate::screens;
use directories::BaseDirs;
use egui::{style, Color32, Rounding, Stroke};
use json;
use std::fs;
use std::path::Path;
use yaml_rust::{Yaml, YamlLoader};

pub struct VaultConfig {
    graph_json_config: String,
}

pub fn load_vault() -> (
    String,
    Vec<String>,
    Option<String>,
    String,
    screens::Screen,
    bool,
    f32,
    bool,
) {
    let binding = BaseDirs::new().unwrap();
    let home_dir = binding.home_dir().to_str().unwrap();

    let config_dir_path = format!("{}/.config/marmol", home_dir);
    let config_file_path = format!("{}/ProgramState", config_dir_path);

    if Path::new(&config_file_path).exists() {
        println!("Fichero de configuración encontrado. Cargando estado...");

        let data =
            fs::read_to_string(&config_file_path).expect("No se pudo leer el archivo de estado");
        let docs = YamlLoader::load_from_str(&data).unwrap_or_default();

        if docs.is_empty() {
            println!("Advertencia: El fichero de configuración está vacío o corrupto. Creando uno nuevo.");
            return create_default_vault(&config_dir_path);
        }

        let doc = &docs[0];

        let vault_var = doc["vault"].as_str().unwrap_or("").to_string();
        if vault_var.is_empty() {
            return create_default_vault(&config_dir_path);
        }

        let mut current = doc["current"].as_str().unwrap_or("").to_string();
        if current.is_empty() {
            current = vault_var.clone();
        }

        let vault_vec_var: Vec<String> = doc["vault_vec"]
            .as_vec()
            .unwrap_or(&Vec::<Yaml>::new())
            .iter()
            .map(|x| x.as_str().unwrap_or("").to_owned())
            .collect();
        let collpased_left = doc["left_menu"].as_bool().unwrap_or(true);
        let center_size = doc["center_size"].as_f64().unwrap_or(0.8) as f32;
        let sort_files = doc["sort_files"].as_bool().unwrap_or(false);

        (
            vault_var,
            vault_vec_var,
            Some(current),
            config_dir_path,
            screens::Screen::Main,
            collpased_left,
            center_size,
            sort_files,
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
) {
    let default_vault = String::new();
    let default_vault_vec = vec![];
    let default_current = None;
    let default_collpased_left = true;
    let default_center_size = 0.8;
    let default_sort_files = true;
    let default_window = screens::Screen::Default;

    (
        default_vault,
        default_vault_vec,
        default_current,
        config_dir.to_string(),
        default_window,
        default_collpased_left,
        default_center_size,
        default_sort_files,
    )
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

pub fn load_colors() -> style::Visuals {
    let binding = BaseDirs::new().unwrap();
    let home_dir = binding.home_dir().to_str().unwrap();
    let mut config_path_var = String::from(home_dir);
    config_path_var = config_path_var + "/.config/marmol/themes.json";
    let themes = Path::new(&config_path_var);
    if !themes.exists() {
        return style::Visuals::default();
    };
    let data = fs::read_to_string(themes).expect("Unable to read file");
    let data = json::parse(&data).unwrap_or(return style::Visuals::default());
    let vis = json::parse(&data["visuals"].dump()).unwrap().entries();
    for theme in vis {}

    //https://docs.rs/egui/0.21.0/egui/style/struct.Visuals.html
    let widget_visuals_active = style::WidgetVisuals {
        bg_fill: Color32::WHITE,
        weak_bg_fill: Color32::BLUE,
        bg_stroke: Stroke {
            width: 5.0,
            color: Color32::GREEN,
        },
        rounding: Rounding::default(),
        fg_stroke: Stroke {
            width: 5.0,
            color: Color32::RED,
        },
        expansion: 10.0,
    };

    let widget_visuals_nonineractive = style::WidgetVisuals {
        bg_fill: Color32::WHITE,
        weak_bg_fill: Color32::BLUE,
        bg_stroke: Stroke {
            width: 5.0,
            color: Color32::GREEN,
        },
        rounding: Rounding::default(),
        fg_stroke: Stroke {
            width: 5.0,
            color: Color32::RED,
        },
        expansion: 10.0,
    };

    let widgets = style::Widgets {
        noninteractive: widget_visuals_nonineractive,
        hovered: widget_visuals_active,
        ..Default::default()
    };

    style::Visuals {
        widgets,
        dark_mode: true,
        ..Default::default()
    }
}
