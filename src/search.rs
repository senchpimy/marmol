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

pub fn check_dir_regex(path: &str, query: &str) -> Vec<MenuItem>{
    let walkdir = WalkDir::new(path);
        let files = walkdir.into_iter().filter_map(|e| e.ok())
            .filter(|e| e.metadata().unwrap().is_file());
            let results:Vec<MenuItem>=files
                .filter_map(|file| match fstream::contains(file.path(), query) {
                    Some(b) => if b {
                        let mut result = MenuItem::new();
                        result.path = file.path().to_str().unwrap().to_string();
                        check_file_regex(file.path(), query, &mut result);
                        Some(result)
                    } else {
                        None
                    },
                    None => None
                }).collect();
            return results;
}

pub fn check_dir(path: &str, query: &str) -> Vec<MenuItem>{
    let walkdir = WalkDir::new(path);
        let files = walkdir.into_iter().filter_map(|e| e.ok())
            .filter(|e| e.metadata().unwrap().is_file());
            let results:Vec<MenuItem>=files
                .filter_map(|file| match fstream::contains(file.path(), query) {
                    Some(b) => if b {
                        let mut result = MenuItem::new();
                        result.path = file.path().to_str().unwrap().to_string();
                        //check file
                        match fstream::read_lines(file.path()) {
                            Some(lines) => {
                                for line in lines {
                                        if line.contains(query) {
                                            let line: String = line.trim().chars().take(200).collect();
                                            result.text=line;
                                        }
                                }
                            }
                        //check file
        None => println!("Error in reading File"),
    }
                        Some(result)
                    } else {
                        None
                    },
                    None => None
                }).collect();
            return results;
}


fn check_file_regex(file_path: &Path, query: &str, result: &mut MenuItem) {
    let re:Regex;
    match Regex::new(query){
        Ok(t)=>re=t,
        Err(_)=>return
    };
    match fstream::read_lines(file_path) {
        Some(lines) => {
            for line in lines {
                if re.is_match(&line){
                    let line: String = line.trim().chars().take(200).collect();
                    result.text=line.as_str().to_string();
                }
            }
        }
        None => println!("Error in reading File"),
    }
}

