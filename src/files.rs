use filebuffer::FileBuffer;
use std::fs::File;
use std::io::Read;

pub fn read_file(file_name: &str) -> String {
    let file = File::open(file_name);
    let mut contents = String::new();
    match file {
        Ok(mut t) => match t.read_to_string(&mut contents) {
            Ok(_) => {}
            Err(e) => return e.to_string(),
        },
        Err(e) => {
            contents = format!("Error Reading File\n\n {}", e.to_string());
        }
    }
    contents
}

pub fn read_image(path: &str) -> Vec<u8> {
    let fbuffer = FileBuffer::open(path);
    match fbuffer {
        Ok(val)=>{
    let vec = val.leak().to_vec();
    vec
        }
        Err(_)=>{vec![]}
    }
}

pub fn contents(contents: &String) -> (String, String) {
    let metadata = String::new();
    if contents.starts_with("---") {
        let test = contents.splitn(3, "---");
        let test: Vec<&str> = test.collect();
        return (test[2].to_string(), test[1].to_string());
    } else {
        return (contents.to_string(), metadata);
    }
}
