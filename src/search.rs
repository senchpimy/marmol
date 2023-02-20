extern crate walkdir;
extern crate regex;

use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;

pub struct MenuItem {
    pub path:String,
    pub text:String,
}
impl MenuItem{
    fn new() -> Self{
        MenuItem { path: "".to_owned(), text: "".to_owned() }
    }
}

pub fn check_dir(path: &str, query: &str, regexp: &bool) -> Vec<MenuItem>{
    let walkdir = WalkDir::new(path);
    let files = walkdir.into_iter().filter_map(|e| e.ok())
        .filter(|e| e.metadata().unwrap().is_file());

    let results:Vec<MenuItem>=files
        .filter_map(|file| match fstream::contains(file.path(), query) {
            Some(b) => if b {
                let mut result = MenuItem::new();
                result.path = file.path().to_str().unwrap().to_string();
                check_file(file.path(), query, regexp, &mut result); //Se pueden evitar muchas
                                                                     //comparaciones guardando el
                                                                     //valor se regexp en un caso
                                                                     //especial
                Some(result)
            } else {
                None
            },
            None => None
        })
        .collect();
    results
}

fn check_file(file_path: &Path, query: &str, regexp:&bool, result: &mut MenuItem) {
    match fstream::read_lines(file_path) {
        Some(lines) => {
            for line in lines {
                if *regexp {
                    let re = Regex::new(query).unwrap();
                    if re.is_match(&line){
                        let line: String = line.trim().chars().take(200).collect();
                        result.text=line;
                    }
                } else{
                    if line.contains(query) {
                    let line: String = line.trim().chars().take(200).collect();
                    result.text=line.as_str().to_string();
                    }
                }
            }
        }
        None => println!("Error in reading File"),
    }
}

