use std::fs::File;
use std::io::Write;
use std::io::Read;
use chrono::prelude::*;
use yaml_rust::{YamlLoader, Yaml};

pub fn create_date_file()->String {
let date = Local::now().format("%Y-%m-%d").to_string();
let file_name = format!("{}.md", date);
let mut file = File::create(file_name).expect("Unable to create file");
file.write_all(b"Hello, world!").expect("Unable to write to file");
date.to_string()
}

pub fn read_file(file_name: &str) -> String {
let mut file = File::open(file_name).expect("Unable to open file");
let mut contents = String::new();
file.read_to_string(&mut contents).expect("Unable to read from file");
contents
}

pub fn read_image(path: &str) ->Vec<u8>{
    std::fs::read(path).unwrap()
}

pub fn contents(contents:&String)->(String,Yaml){
    let metadata= Yaml::from_str("-123");
    if contents.starts_with("---"){
        let test = contents.split("---");
        let test: Vec<&str> = test.collect();
        let tags = test[1];
        let docs = YamlLoader::load_from_str(tags).unwrap();
        let metadata = &docs[0];
        dbg!(&metadata["tags"]);
    }
    (contents.to_string(), metadata)
}
