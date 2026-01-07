extern crate regex;
extern crate walkdir;

use regex::Regex;
use walkdir::WalkDir;

pub struct MenuItem {
    pub path: String,
    pub text: String,
}

impl MenuItem {
    fn new() -> Self {
        MenuItem {
            path: "".to_owned(),
            text: "".to_owned(),
        }
    }
}

pub fn search_dir(path: &str, query: &str, use_regex: bool) -> Vec<MenuItem> {
    let walkdir = WalkDir::new(path);
    let files = walkdir
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.metadata().unwrap().is_file());

    let regex = if use_regex {
        Regex::new(query).ok()
    } else {
        None
    };

    files
        .filter_map(|file| {
            if fstream::contains(file.path(), query).unwrap_or(false) {
                let mut result = MenuItem::new();
                result.path = file.path().to_str().unwrap().to_string();
                
                if let Some(lines) = fstream::read_lines(file.path()) {
                    for line in lines {
                        let matched = if let Some(ref re) = regex {
                            re.is_match(&line)
                        } else {
                            line.contains(query)
                        };

                        if matched {
                            result.text = line.trim().chars().take(200).collect();
                            break;
                        }
                    }
                }
                Some(result)
            } else {
                None
            }
        })
        .collect()
}

pub fn check_dir_regex(path: &str, query: &str) -> Vec<MenuItem> {
    search_dir(path, query, true)
}

pub fn check_dir(path: &str, query: &str) -> Vec<MenuItem> {
    search_dir(path, query, false)
}
