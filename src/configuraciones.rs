use crate::screens;
use yaml_rust::{YamlLoader,Yaml};
use std::path::Path;
use directories::BaseDirs;
use std::fs;

pub fn load_vault()->(String, Vec<Yaml>, String, String, screens::Screen,f32,bool,f32,bool){
    let mut current=String::new();
    let mut vault_var=String::new();
    let mut font_size:f32=12.0;
    let mut vault_vec_var:Vec<Yaml> = vec![];
    let mut window = screens::Screen::Default;
    let binding = BaseDirs::new().unwrap();
    let home_dir = binding.home_dir().to_str().unwrap();
    let mut config_path_var = String::from(home_dir);
    config_path_var=config_path_var+"/.config/marmol";
    let mut collpased_left = true;
    let mut sort_files = true;
    let mut center_size = 35.0;
    let dir = Path::new(&config_path_var);
    if dir.exists(){
        let file_saved = String::from(&config_path_var)+"/ProgramState";
        let dir2 = Path::new(&file_saved);
        window = screens::Screen::Main;
            if dir2.exists(){
                    let data = fs::read_to_string(file_saved)
                        .expect("Unable to read file");
                    let docs = YamlLoader::load_from_str(&data).unwrap();
                    let docs = &docs[0];
                    vault_var = docs["vault"].as_str().unwrap().to_string();
                    current = docs["current"].as_str().unwrap().to_string();
                    font_size = docs["font_size"].as_i64().unwrap() as f32;
                    vault_vec_var = docs["vault_vec"].as_vec().unwrap().to_vec();
                    collpased_left = docs["left_menu"].as_bool().unwrap();
                    center_size = docs["center_size"].as_f64().unwrap() as f32;
                    sort_files = docs["sort_files"].as_bool().unwrap_or(false);
                println!("Estado anterior cargado");
    }else{
        let res = fs::create_dir(&dir);
        match res{
            Ok(_)=>println!("Dir created"),
            Err(r)=>println!("{}",r)
        }
    }
    }
    return (vault_var.to_string(), vault_vec_var, current.to_string(),config_path_var.to_string(),window,font_size,collpased_left,center_size,sort_files);
}
