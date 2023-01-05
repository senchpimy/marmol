use std::fs::File;
use std::io::Write;
use std::io::Read;
use chrono::prelude::*;

pub fn create_date_file() {
let date = Local::now().format("%Y-%m-%d").to_string();
let file_name = format!("{}.md", date);
let mut file = File::create(file_name).expect("Unable to create file");
file.write_all(b"Hello, world!").expect("Unable to write to file");
}

pub fn read_file(file_name: &str) -> String {
let mut file = File::open(file_name).expect("Unable to open file");
let mut contents = String::new();
file.read_to_string(&mut contents).expect("Unable to read from file");
contents
}
