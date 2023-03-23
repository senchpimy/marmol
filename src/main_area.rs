use crate::search;
use crate::screens;
use eframe::egui::{ScrollArea,Separator,TopBottomPanel,SidePanel,Context,Layout,Align,ImageButton,TextureId, Style,Frame, Button,RichText};
use egui_extras::RetainedImage;
use json;
use egui::{ TextFormat, Color32,text::LayoutJob, Widget};
use std::fs;
use std::path::Path;
use chrono::prelude::*;
use std::fs::File;
use yaml_rust::{YamlLoader,YamlEmitter};

#[derive(PartialEq)]
pub enum Content {
    Edit,
    View,
}
//#[derive(Debug)]
pub struct LeftControls {
    current_left_tab: i8,
    search_string_menu:String,
    prev_search_string_menu:String,
    search_results:Vec<search::MenuItem>,
    regex_search:bool,

    //right_collpased:bool,
    colapse_image:RetainedImage,
    files_image:RetainedImage,
    search_image:RetainedImage,
    new_file:RetainedImage,
    starred_image:RetainedImage,
    config_image:RetainedImage,
    vault_image:RetainedImage,
    help_image:RetainedImage,
    switcher_image:RetainedImage,
    graph_image:RetainedImage,
    canvas_image:RetainedImage,
    daynote_image:RetainedImage,
    command_image:RetainedImage,
}

impl Default for LeftControls{
    fn default() -> Self {
        Self{
            current_left_tab:0,
            search_string_menu:"".to_owned(),
            prev_search_string_menu:"".to_owned(),
            search_results:vec![],
            regex_search:false,
            colapse_image: RetainedImage::from_image_bytes("colapse",include_bytes!("../colapse.png"),).unwrap(),
            files_image: RetainedImage::from_image_bytes("files",include_bytes!("../files.png"),).unwrap(),
            search_image: RetainedImage::from_image_bytes("search",include_bytes!("../search.png"),).unwrap(),
            new_file: RetainedImage::from_image_bytes("search",include_bytes!("../new_file.png"),).unwrap(),
            starred_image: RetainedImage::from_image_bytes("starred",include_bytes!("../starred.png"),).unwrap(),
            config_image: RetainedImage::from_image_bytes("cpnfiguration",include_bytes!("../configuration.png"),).unwrap(),
            vault_image: RetainedImage::from_image_bytes("vault",include_bytes!("../vault.png"),).unwrap(),
            help_image: RetainedImage::from_image_bytes("help",include_bytes!("../help.png"),).unwrap(),
            switcher_image: RetainedImage::from_image_bytes("switcher",include_bytes!("../switcher.png"),).unwrap(),
            graph_image: RetainedImage::from_image_bytes("graph",include_bytes!("../graph.png"),).unwrap(),
            canvas_image: RetainedImage::from_image_bytes("canvas",include_bytes!("../canvas.png"),).unwrap(),
            daynote_image: RetainedImage::from_image_bytes("daynote",include_bytes!("../daynote.png"),).unwrap(),
            command_image: RetainedImage::from_image_bytes("command",include_bytes!("../command.png"),).unwrap(),
        }
    }
}
impl LeftControls{
pub fn left_side_menu(&mut self, ctx:&Context, colapse:&bool, 
                  path:&str, current_file:&mut String,){
    let left_panel = SidePanel::left("buttons left menu").default_width(100.).min_width(100.).max_width(300.);
    let textures = vec![self.files_image.texture_id(ctx), self.search_image.texture_id(ctx), 
                        self.starred_image.texture_id(ctx)];
    left_panel.show_animated(ctx, *colapse,|ui| {
        self.top_panel_menu_left(ui,textures, path, current_file);
    });
}

fn top_panel_menu_left (&mut self,ui:&mut egui::Ui, textures:Vec<TextureId>, path:&str, current_file:&mut String){
    TopBottomPanel::top("Left Menu").show_inside(ui, |ui|{
        ui.with_layout(Layout::left_to_right(Align::Min),|ui| {
     if ui.add(ImageButton::new(textures[0], egui::vec2(18.0, 18.0)).frame(false)).clicked(){self.current_left_tab=0;}
     if ui.add(ImageButton::new(textures[1], egui::vec2(18.0, 18.0)).frame(false)).clicked(){self.current_left_tab=1;}
     if ui.add(ImageButton::new(textures[2], egui::vec2(18.0, 18.0)).frame(false)).clicked(){self.current_left_tab=2;}
        });
    });
    if self.current_left_tab==0{
        let scrolling_files = ScrollArea::vertical();
        scrolling_files.show(ui,|ui| {
        Self::render_files(ui, path, current_file);
        });
    }else if self.current_left_tab==1{
        ui.text_edit_singleline(&mut self.search_string_menu);
        ui.checkbox(&mut self.regex_search,"regex");
        if self.search_string_menu!=self.prev_search_string_menu{
            if self.regex_search{
                self.search_results = search::check_dir_regex(path,&self.search_string_menu);
                self.prev_search_string_menu=self.search_string_menu.to_string();
            }else{
                self.search_results = search::check_dir(path,&self.search_string_menu);
                self.prev_search_string_menu=self.search_string_menu.to_string();
            }
        }
        let style_frame = Style::default();
        let frame = Frame::group(&style_frame);
        if self.search_string_menu.len()<1{
            self.search_results = vec![];
        }
        let scrolling_search = ScrollArea::vertical();
        scrolling_search.show(ui,|ui| {
            for i in &self.search_results{
                frame.show(ui, |ui|{
                    let mut title = LayoutJob::default();
                    title.append(&i.path.strip_prefix(&path).unwrap(),0.0,TextFormat{color:Color32::RED,..Default::default()});
                    ui.label(title);
                    ui.label(&i.text);
                    if ui.button("open file").clicked(){
                        *current_file=String::from(&i.path);
                    };
                });
            }
        });
    }else if self.current_left_tab==2{
        let contents = fs::read_to_string(format!("{}/.obsidian/starred.json",path))
            .expect("Should have been able to read the file");
        let parsed = json::parse(&contents).unwrap();
        for (_key, value) in parsed.entries() {
            for i in 0..value.len(){
                let text=parsed["items"][i]["path"].as_str().unwrap();
                let full_path = format!("{}/{}",path,text);
                if full_path == current_file.as_str(){
                    ui.label(RichText::new(text).color(ui.style().visuals.selection.bg_fill));
                }else{
                    let btn = Button::new(text).frame(false);
                    if btn.ui(ui).clicked() {
                        *current_file = Path::new(&full_path).to_str().unwrap().to_owned();
                    }
                }
            }
        }
    }
}

fn render_files(ui:&mut egui::Ui, path:&str, current_file:&mut String){
    let read_d =fs::read_dir(path);
    let entrys:fs::ReadDir;
    match read_d{
            Ok(t)=> entrys = t,
            Err(r)=>{
            ui.label("Nothing to see here");
            ui.label(egui::RichText::new(r.to_string()).strong());
            return;
        }
    }
    for entry in entrys{
        let file_location = entry.unwrap().path().to_str().unwrap().to_string();
        let file_name=Path::new(&file_location).file_name().expect("No fails").to_str().unwrap();
        if Path::new(&file_location).is_dir(){
            let col = egui::containers::collapsing_header::CollapsingHeader::new(file_name);
            col.show(ui, |ui| {
            Self::render_files(ui,&file_location, current_file);
            });
        }else{
            if file_location.as_str() == current_file.as_str(){
                ui.label(RichText::new(file_name).color(ui.style().visuals.selection.bg_fill));
            }else{
                let btn = Button::new(file_name).frame(false);
                if btn.ui(ui).context_menu(Self::file_options).clicked() {
                    *current_file = file_location;
                }
            }
        }
    }

 }

pub fn left_side_settings(&self,ctx:&Context, colapse:&mut bool, vault:&mut String,current_file:&mut String, current_window:&mut screens::Screen,){
    let left_panel = SidePanel::left("buttons left").resizable(false).default_width(1.);
    let space = 10.;
    left_panel.show(ctx,|ui| {
        ui.add_space(5.);
        ui.vertical(|ui| {
        if ui.add(ImageButton::new(self.colapse_image.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){
            if *colapse{
                *colapse=false;
            }else{
                *colapse=true;
            }
        }
        ui.add(Separator::default());
        if ui.add(ImageButton::new(self.switcher_image.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("switcher")} //quick switcher
        ui.add_space(space);
        if ui.add(ImageButton::new(self.graph_image.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("graph")}//graph
        ui.add_space(space);
        if ui.add(ImageButton::new(self.canvas_image.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("canvas")}//canvas
        ui.add_space(space);
        if ui.add(ImageButton::new(self.daynote_image.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){
            Self::create_date_file(vault, current_file);
        }//note
        ui.add_space(space);
        if ui.add(ImageButton::new(self.command_image.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("commandpale")}//palette
        ui.add_space(space);
        ui.menu_image_button(self.new_file.texture_id(ctx), egui::vec2(18.0, 18.0),Self::new_file_menu);//new_file
        ui.with_layout(Layout::bottom_up(Align::Max),|ui|{
        ui.add_space(5.);
             if ui.add(ImageButton::new(self.config_image.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){*current_window=screens::Screen::Configuracion;}
        ui.add_space(5.);
             if ui.add(ImageButton::new(self.help_image.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("help")}
        ui.add_space(5.);
             if ui.add(ImageButton::new(self.vault_image.texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("vault")}

        });
        });
    });
}

fn create_date_file(path:&String, current_file: &mut String) {
    let date = Local::now().format("%Y-%m-%d").to_string();
    let file_name = format!("{}/{}.md",path,date);
    if Path::new(&file_name).exists(){
        *current_file=file_name.to_string();
    }else{
        File::create(&file_name).expect("Unable to create file");
        *current_file=file_name.to_string();
    }
}

}

pub fn create_metadata(metadata:String, ui:&mut egui::Ui){
    let docs = YamlLoader::load_from_str(&metadata).unwrap();
    let metadata_parsed = &docs[0];
    let mut job = LayoutJob::default();

    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(metadata_parsed).unwrap();
    out_str.split("\n").skip(1).for_each(|s|{
        //dbg!(s);
        if s.as_bytes()[s.len()-1]==58 {
            job.append(
                &(s.to_owned()+"\n"),0.0,
                 TextFormat {
                 color: Color32::GRAY,
                 ..Default::default()
                 },
                )
        }else if  s.as_bytes()[0]==32 {
            job.append(
                &(s.to_owned()+"\n"),0.0,
                 TextFormat {
                 color: Color32::WHITE,
                 ..Default::default()
                 },
                )
        }else{
            let mut test =s.split(" ");
            job.append(
                test.next().unwrap(),0.0,
                 TextFormat {
                 color: Color32::GRAY,
                 ..Default::default()
                 },
                );
            job.append(
                &(test.next().unwrap().to_owned()+"\n"),0.0,
                 TextFormat {
                 color: Color32::WHITE,
                 ..Default::default()
                 },
                );
        }
    });
    ui.label(job);
}
impl LeftControls{
    fn new_file_menu(ui: &mut egui::Ui) {
        ui.label("Create a New File");
        let response = ui.add(egui::TextEdit::singleline(&mut String::from("")).hint_text("Name"));
        if response.changed(){
        }
        if ui.button("Create").clicked() {
            ui.close_menu();
        }
        if ui.button("Cancel").clicked() {
            ui.close_menu();
        }
    }

    fn file_options(ui: &mut egui::Ui) {
        let copy = egui::Button::new("Copy file").frame(false);
        ui.label("Move");
        if ui.add(copy).clicked() {
        }
        let response = ui.add(egui::TextEdit::singleline(&mut String::from("")).hint_text("Rename"));
        if response.changed(){
        }
        let delete = egui::Button::new(RichText::new("Delete file").color(Color32::RED));//.frame(false);
        if ui.add(delete).clicked() {
        }
    }
}
