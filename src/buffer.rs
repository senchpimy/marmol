#[derive(Debug)]
pub struct Window {
    pub vertical_buffers: Vec<Horizontal_Buffers>
}
#[derive(Debug)]
struct Horizontal_Buffers {
    buffers : Vec<Buffer>
}

#[derive(Debug)]
struct Buffer {
    path : String,
    content : String,
}
