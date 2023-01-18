use egui::{Color32,Button,Ui};

impl Into<Color32> for Tab{
    fn into(self) -> Color32{
        self.color
    }
}

#[derive (Debug)]
pub struct Tab {
    content:String,
    path:String,
    pub title:String,
    selected:bool,
    pub color:Color32,
}

impl Tab{
    pub fn new()->Self{
Tab{content:"hola".to_string(), path: "/home/test/path".to_string(),title:"Titulo".to_string(),selected:false,color:Color32::BLUE}
    }
}

pub fn create_tab(ui: &mut Ui,tab:&mut Tab){
    if tab.selected{
        tab.color = Color32::WHITE;
    }
        if ui.add(Button::new(&tab.title).fill(tab.color)).clicked(){
            tab.selected = true;
        }
}
