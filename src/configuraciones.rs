use crate::screens;
use directories::BaseDirs;
use egui::{style, Color32, Rounding, Stroke};
use json;
use std::fs;
use std::path::Path;
use yaml_rust::{Yaml, YamlLoader};

pub struct VaultConfig{
    graph_json_config:String
}

pub fn load_vault() -> (
    String,
    Vec<String>,
    String,
    String,
    screens::Screen,
    bool,
    f32,
    bool,
) {
    let mut current = String::new();
    let mut vault_var = String::new();
    let mut vault_vec_var: Vec<String> = vec![];
    let mut window = screens::Screen::Default;
    let binding = BaseDirs::new().unwrap();
    let home_dir = binding.home_dir().to_str().unwrap();
    let mut config_path_var = String::from(home_dir);
    config_path_var = config_path_var + "/.config/marmol";
    let mut collpased_left = true;
    let mut sort_files = true;
    let mut center_size = 0.8;
    let dir = Path::new(&config_path_var);
    if dir.exists() {
        let file_saved = String::from(&config_path_var) + "/ProgramState";
        let dir2 = Path::new(&file_saved);
        if dir2.exists() {
            println!("Configuration file exists");
            let data = fs::read_to_string(file_saved).expect("Unable to read file");
            let docs = YamlLoader::load_from_str(&data).unwrap_or(vec![]);
            window = screens::Screen::Main;
            let docs = &docs[0];
            vault_var = docs["vault"].as_str().unwrap().to_string();
            current = docs["current"].as_str().unwrap_or("None").to_string();
            vault_vec_var = docs["vault_vec"]
                .as_vec()
                .unwrap_or(&Vec::<Yaml>::new())
            .iter().map(|x|x.as_str().unwrap_or("").to_owned()).collect();
            //vault_vec_var = tmp.iter().map(|x|x.as_str().unwrap().to_owned()).collect();
            collpased_left = docs["left_menu"].as_bool().unwrap_or(true);
            center_size = docs["center_size"].as_f64().unwrap_or(0.8) as f32;
            sort_files = docs["sort_files"].as_bool().unwrap_or(false);
        } else {
            let res = fs::create_dir(&dir);
            match res {
                Ok(_) => println!("Dir created"),
                Err(r) => println!("Dir cannot be created: {}", r),
            }
        }
    }
    return (
        vault_var.to_string(),
        vault_vec_var,
        current.to_string(),
        config_path_var.to_string(),
        window,
        collpased_left,
        center_size,
        sort_files,
    );
}

pub fn load_context() -> f32 {
    let binding = BaseDirs::new().unwrap();
    let home_dir = binding.home_dir().to_str().unwrap();
    let config_path_var = format!("{}{}",&home_dir,"/.config/marmol");
    let dir = Path::new(&config_path_var);

    let mut font_size: f32 = 12.0;
    if dir.exists() {
        let file_saved = format!("{}{}",&config_path_var,"/ContextState");
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
