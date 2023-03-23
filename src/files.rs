use std::fs::File;
use std::io::Read;
use filebuffer::FileBuffer;


pub fn read_file(file_name: &str) -> String {
    let file = File::open(file_name);
    let mut contents = String::new();
    match file{
        Ok(mut t)=>{t.read_to_string(&mut contents).unwrap();},//.expect("Unable to read from file"),
        Err(e)=>{contents=format!("Error Reading File\n\n {}",e.to_string());}
    }
    contents
}

pub fn read_image(path: &str) ->Vec<u8>{
    let fbuffer = FileBuffer::open(path).expect("failed to open file");
    let vec =fbuffer.leak().to_vec();
    vec
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
