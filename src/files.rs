use std::fs::File;
use std::io::Read;


pub fn read_file(file_name: &str) -> String {
let mut file = File::open(file_name).expect("Unable to open file");
let mut contents = String::new();
file.read_to_string(&mut contents).expect("Unable to read from file");
contents
}

pub fn read_image(path: &str) ->Vec<u8>{
    std::fs::read(path).unwrap()
}

pub fn contents(contents:&String)->(String,String){
    let metadata= String::new();
    if contents.starts_with("---"){
        let test = contents.splitn(3,"---");
        let test: Vec<&str> = test.collect();
        return (test[2].to_string(),test[1].to_string());
    }else{
    return (contents.to_string(), metadata);
    }
}
