use chrono::prelude::*;
use std::process::Command;

use crate::files;
//Reads when the last backup where made
fn read_date() {
    let date = files::read_file("file");
    let _paserd_date = match NaiveDate::parse_from_str(&date, "") {
        Ok(some) => some,
        Err(_) => NaiveDate::MIN,
    };
}

//uses Rsync to sync uses Rsync to sync
fn update(source: &str, destination: &str) {
    let out = Command::new("rsync")
        .arg("--archive")
        .arg(source)
        .arg(destination)
        .output()
        .expect("Failed to execute");

    println!("{:?}", out.stdout);
}
