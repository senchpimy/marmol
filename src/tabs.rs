use egui::{Color32,Button,Ui};

impl Into<Color32> for Tab{
    fn into(self) -> Color32{
        self.color
    }
}

#[derive (Debug)]
pub struct Tab {
    pub content: Vec<HorizontalBuffers>,
    pub title:String,
    selected:bool,
    pub color:Color32,
}

impl Tab{
    pub fn new()->Self{
    Tab{content:vec![], title:"Titulo".to_string(),selected:false,color:Color32::BLUE}
    }
    pub fn default(path:String)->Self{
    Tab{
        content:vec![HorizontalBuffers::default(path)],
        title:"Titulo".to_string(),
        selected:false,
        color:Color32::BLUE
    }
    }
}

#[derive(Debug)]
pub struct HorizontalBuffers {
    buffers : Vec<Buffer>
}

impl HorizontalBuffers{
    pub fn default(path:String)->Self{
        HorizontalBuffers { buffers: vec![Buffer::new(path)] }
    }
    pub fn new()->Self{
        HorizontalBuffers { buffers: vec![] }
    }
} 

#[derive(Debug)]
struct Buffer {
    path : String,
    content: String,
}

impl Buffer{
    pub fn new(path_in:String)->Self{
        Buffer { path: path_in, content:String::new() }
    }
} 

pub fn start(path:String){
    let _i = Buffer::new(path);
}

pub fn create_tab(ui: &mut Ui,tab:&mut Tab){
    if tab.selected{
        tab.color = Color32::WHITE;
    }
        if ui.add(Button::new(&tab.title).fill(tab.color)).clicked(){
            tab.selected = true;
        }
}
