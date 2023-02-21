use std::fs::File;
use std::io::Read;
use filebuffer::FileBuffer;


pub fn read_file(file_name: &str) -> String {
println!("{}",file_name);
    let mut file2 = File::open(file_name).expect("Unable to open file");
    let mut contents = String::new();
    file2.read_to_string(&mut contents).expect("Unable to read from file");
    contents
}

pub fn read_image(path: &str) ->Vec<u8>{
// 30% faster
 println!("empieza a leer");
let fbuffer = FileBuffer::open(path).expect("failed to open file");
let test =fbuffer.leak().to_vec();
 println!("termina de leer");
test
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
