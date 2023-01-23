extern crate walkdir;
extern crate regex;

use regex::Regex;
use std::path::Path;
use walkdir::WalkDir;
use std::fs::File;
use std::io::{BufRead, BufReader};

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
                result.path = file.file_name().to_str().unwrap().to_string();
                check_file(file.path(), query, regexp, &mut result);
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
                if line.contains(query) {
                    let line: String = line.trim().chars().take(200).collect();
                    result.text=line.as_str().to_string();
                    if *regexp {
                        let re = Regex::new(query).unwrap();
                        if re.is_match(&line){
                            result.text=line;
                        }
                    }
                }
            }
        }
        None => println!("Error in reading File"),
    }
}

//fn check_dir(path: &str, query: &str, regexp: &bool) -> Vec<MenuItem> {
//    let re = if *regexp {
//        Some(Regex::new(query).unwrap())
//    } else {
//        None
//    };
//
//    WalkDir::new(path)
//        .into_iter()
//        .filter_map(|file| file.ok())
//        .filter(|file| file.metadata().unwrap().is_file())
//        .filter_map(|file| {
//            if let Some(re) = &re {
//                if !re.is_match(file.file_name().to_str().unwrap()) {
//                    return None;
//                }
//            }
//            let file = File::open(file.path()).unwrap();
//            let reader = BufReader::new(file);
//            let line = reader.lines().filter(|line| line.unwrap().contains(query)).next();
//            println!("{:?}",&file);
//            if let Some(line) = line {
//                Some(MenuItem {
//                    //path: file.file_name().to_str().unwrap().to_string(),
//                    path: "strinf".to_string(),
//                    text: line.unwrap().trim().chars().take(50).collect(),
//                })
//            } else {
//                None
//            }
//        })
//        .collect()
//}

//fn check_dir(path: &str, query: &str, regexp: &bool) -> Vec<MenuItem>{
//    let results:Vec<MenuItem>=vec![];
//    for (_, file) in WalkDir::new(path)
//        .into_iter()
//        .filter_map(|file| file.ok())
//        .enumerate()
//    {
//        if file.metadata().unwrap().is_file() {
//            match fstream::contains(file.path(), query) {
//                Some(b) => {
//                    if b {
//                        let mut result = MenuItem::new();
//                        result.path=file.file_name().to_str().unwrap().to_string();
//                        check_file(file.path(), query, regexp, &mut results,&mut result);
//                    }
//                }
//                None => println!("Error in walking Dir"),
//            }
//        }
//    }
//    results
//}
//fn check_file(file_path: &Path, query: &str, regexp:&bool, results:&mut Vec<MenuItem>,result:&mut MenuItem) {
//    match fstream::read_lines(file_path) {
//        Some(lines) => {
//            for (pos, line) in &mut lines.iter().enumerate() {
//                if line.contains(query) {
//                    let line: String = line.trim().chars().take(50).collect();
//                        result.text=line;
//                        println!("=> {}", line);
//                    if *regexp {
//                        let re = Regex::new(query).unwrap();
//                        if re.is_match(&line){
//                            let line: String = line.trim().chars().take(50).collect();
//                                result.text=line;
//                                println!("=> {}", line);
//                        }
//                    }
//
//                }
//            }
//        }
//        None => println!("Error in reading File"),
//    }
//    results.push(result.clone());
//}
