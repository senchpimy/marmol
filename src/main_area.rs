use crate::search;
use eframe::{egui::{CentralPanel,ScrollArea,Separator,TopBottomPanel,SidePanel,Context,Layout,Align,ImageButton,TextureId, Link,Style,Frame}};
use egui_extras::RetainedImage;
use json;
use egui::{ TextFormat, Color32,text::LayoutJob, };
use std::fs;
use std::path::Path;


pub fn left_side_menu(ctx:&Context, colapse:&bool, images:Vec<&RetainedImage> , 
                  path:&str, current_file:&str, left_tab:&mut i8, search_string_menu:&mut String,
                  prev_search_string_menu:&mut String, search_results:&mut Vec<search::MenuItem>,
                  regex_search:&mut bool){
    let left_panel = SidePanel::left("buttons left menu").default_width(100.).min_width(100.).max_width(300.);
    let textures = vec![images[0].texture_id(ctx), images[1].texture_id(ctx), images[2].texture_id(ctx)];
    left_panel.show_animated(ctx, *colapse,|ui| {
        top_panel_menu_left(ui,textures, path, current_file,left_tab,search_string_menu,prev_search_string_menu,search_results,regex_search);
    });
}

fn top_panel_menu_left (ui:&mut egui::Ui, textures:Vec<TextureId>, path:&str, current_file:&str,left_tab:&mut i8, 
                        search_string_menu:&mut String,prev_search_string_menu:&mut String, search_results:&mut Vec<search::MenuItem>,
                        regex_search:&mut bool){
    TopBottomPanel::top("Left Menu").show_inside(ui, |ui|{
        ui.with_layout(Layout::left_to_right(Align::Min),|ui| {
     if ui.add(ImageButton::new(textures[0], egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("files");*left_tab=0;}
     if ui.add(ImageButton::new(textures[1], egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("search"); *left_tab=1;}
     if ui.add(ImageButton::new(textures[2], egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("starred"); *left_tab=2;}
        });
    });
    if *left_tab==0{
        let scrolling_files = ScrollArea::vertical();
        scrolling_files.show(ui,|ui| {
        render_files(ui,path, &current_file);
        });
    }else if *left_tab==1{
        ui.text_edit_singleline(search_string_menu);
        ui.checkbox(regex_search,"regex");
        if *regex_search{
            if ui.button("search").clicked(){
                unimplemented!();
            }
        }
        if search_string_menu!=prev_search_string_menu{
            *search_results = search::check_dir(path,search_string_menu,regex_search);
            *prev_search_string_menu=search_string_menu.to_string();
        }
        let style_frame = Style::default();
        let frame = Frame::group(&style_frame);
        if search_string_menu.len()<1{
            *search_results = vec![];
        }
        let scrolling_search = ScrollArea::vertical();
        scrolling_search.show(ui,|ui| {
            for i in search_results{
                frame.show(ui, |ui|{
                    let mut title = LayoutJob::default();
                    title.append(&i.path,0.0,TextFormat{color:Color32::RED,..Default::default()});
                    ui.label(title);
                    ui.label(&i.text);
                    ui.button("open file");
                });
            }
        });
    }else if *left_tab==2{
        let contents = fs::read_to_string(format!("{}/.obsidian/starred.json",path))
            .expect("Should have been able to read the file");
        let parsed = json::parse(&contents).unwrap();
        for (_key, value) in parsed.entries() {
            for i in 0..value.len(){
                let text=format!("{}",parsed["items"][i]["path"]);
                ui.label(&text);
            }
        }
    }
}

fn render_files(ui:&mut egui::Ui, path:&str, current_file:&str){
        for entry in fs::read_dir(path).unwrap(){
            let file_location = entry.unwrap().path().to_str().unwrap().to_string();
            let file_name=Path::new(&file_location).file_name().expect("No fails").to_str().unwrap();
            if Path::new(&file_location).is_dir(){
                let col = egui::containers::collapsing_header::CollapsingHeader::new(file_name);
                col.show(ui, |ui| {
                render_files(ui,&file_location, current_file);
                });
            }else{
                let clickable = Link::new(file_name);
                if ui.add(clickable).clicked() {
                    let current_file = &file_location;
                    println!("{}",current_file);
                }
            }
        }

 }

pub fn left_side_settings(ctx:&Context, colapse:&mut bool, images:&[&RetainedImage]){
    let left_panel = SidePanel::left("buttons left").resizable(false).default_width(1.);
    let space = 10.;
    left_panel.show(ctx,|ui| {
        ui.add_space(5.);
        ui.vertical(|ui| {
        if ui.add(ImageButton::new(images[0].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){
            if *colapse{
                *colapse=false;
            }else{
                *colapse=true;
            }
        }
        ui.add(Separator::default());
        if ui.add(ImageButton::new(images[1].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("switcher")} //quick switcher
        ui.add_space(space);
        if ui.add(ImageButton::new(images[2].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("graph")}//graph
        ui.add_space(space);
        if ui.add(ImageButton::new(images[3].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("canvas")}//canvas
        ui.add_space(space);
        if ui.add(ImageButton::new(images[4].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("daynote")}//note
        ui.add_space(space);
        if ui.add(ImageButton::new(images[5].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("commandpale")}//palette
        ui.with_layout(Layout::bottom_up(Align::Max),|ui|{
        ui.add_space(5.);
             if ui.add(ImageButton::new(images[6].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("conf")}
        ui.add_space(5.);
             if ui.add(ImageButton::new(images[7].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("help")}
        ui.add_space(5.);
             if ui.add(ImageButton::new(images[8].texture_id(ctx), egui::vec2(18.0, 18.0)).frame(false)).clicked(){println!("vault")}

        });
        });
    });
}
