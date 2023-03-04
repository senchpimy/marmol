use crate::screens;
use yaml_rust::{YamlLoader,Yaml};
use std::path::Path;
use directories::BaseDirs;
use std::fs;

pub fn load_vault()->(String, Vec<Yaml>, String, String, screens::Screen){
    let mut current=String::new();
    let mut vault_var=String::new();
    let mut vault_vec_var:Vec<Yaml> = vec![];
    let mut window = screens::Screen::Default;
    let binding = BaseDirs::new().unwrap();
    let home_dir = binding.home_dir().to_str().unwrap();
    let mut config_path_var = String::from(home_dir);
    config_path_var=config_path_var+"/.config/marmol";
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
                    vault_vec_var = docs["vault_vec"].as_vec().unwrap().to_vec();
                println!("Estado anterior cargado");
    }else{
        fs::create_dir(&dir);
        println!("Dir created");
    }
    }
    return (vault_var.to_string(), vault_vec_var, current.to_string(),config_path_var.to_string(),window);
}
