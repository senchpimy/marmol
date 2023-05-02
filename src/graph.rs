//use rand::Rng;
use std::path::Path;
use json;
use std::fs;
use std::collections::HashMap;
use crate::files;
use crate::main_area;
use yaml_rust::YamlLoader;


use egui::*;
use plot::{Line, LineStyle, MarkerShape, Plot, PlotPoint,Points,Text,};

#[derive(Clone)]
struct MarmolPoint {
    text: String,
    tags: Vec<String>,
}

pub struct Graph{
    points:Vec<MarmolPoint>,
    points_coord:Vec<(f32,f32)>,
    center_force:f32,
    repel_force:f32,
    dragable:bool,
    drag_delta:Vec2,
    tags_colors:HashMap<String,Color32>,
    prev_tags_colors:HashMap<String,Color32>,
    prev_colors:Vec<Color32>,
    colors:Vec<Color32>,
    //tags:Vec<String>,
    orphan_color:Color32,
}

impl MarmolPoint{
    fn new(val:&str, tags:Vec<String>)->Self{
        Self{
            text: format!("{}",val),
            tags: tags,
        }
    }
}

impl Graph {
    pub fn new(vault:&str) -> Self {
        let mut tags_hashmap = HashMap::new();
        //let mut tags = vec![];
        let colors_vec = vec![Color32::from_rgb(235, 64, 52),Color32::from_rgb(38, 28, 128),Color32::from_rgb(128, 28, 101)];
        let prev_colors_vec = vec![Color32::from_rgb(235, 64, 52),Color32::from_rgb(38, 28, 128),Color32::from_rgb(128, 28, 101)];
        //tags_hashmap.insert(tags[0].clone(),colors_vec[0]);
        //tags_hashmap.insert(tags[1].clone(),colors_vec[1]);
        //tags_hashmap.insert(tags[2].clone(),colors_vec[2]);
        let mut data = vec![];
        let mut coords = vec![];
        let mut total_entries =0;
        get_data(&Path::new(vault),
        &mut data,&mut total_entries,vault);
        get_coords(&mut coords,total_entries);
        Self {
            points:data,
            points_coord:coords,
            center_force:1.0,
            repel_force:1.0,
            dragable:true,
            drag_delta:vec2(0.0,0.0),
            tags_colors:tags_hashmap.clone(),
            prev_tags_colors:tags_hashmap,
            //tags:tags,
            orphan_color:Color32::from_rgb(66,77,92),
            colors:colors_vec,
            prev_colors:prev_colors_vec,
        }
    }
}

impl Graph {
    pub fn update_vault(&mut self, vault:&Path){
        let mut new_points=vec![];
        let mut new_coords=vec![];
        let mut elements = 0;
        get_data(vault,&mut new_points,&mut elements,vault.to_str().unwrap());
        get_coords(&mut new_coords,elements);
        self.points=new_points;
        self.points_coord=new_coords;
    }
    pub fn controls(&mut self,ctx:&Context){
        egui::Window::new("Configuration").show(ctx, |ui| {
            ui.label("Repel force");
            ui.add(egui::Slider::new(&mut self.repel_force, 5.0..=20.0));
            ui.label("Center force");
            ui.add(egui::Slider::new(&mut self.center_force, 5.0..=20.0));
            let mut j =0;
            for i in self.tags_colors.clone().into_keys(){
                ui.horizontal(|ui| {
                    match self.tags_colors.get(&i){
                        Some(x)=>{
                            let mut col = x.clone();
                            color_picker::color_edit_button_srgba(ui,&mut col,egui::widgets::color_picker::Alpha::Opaque);
                        },
                        None=>{}
                    }
                    ui.label(i);
                });
                j+=1;
            }
            for key in self.tags_colors.clone().into_keys(){
                if self.tags_colors.get(&key)!=self.prev_tags_colors.get(&key){
                    println!("diferente");
                }
            }
                ui.horizontal(|ui| {
                    color_picker::color_edit_button_srgba(ui,&mut self.orphan_color,
                                      egui::widgets::color_picker::Alpha::Opaque);
                    ui.label("orphans");
                });
        });
    }

    pub fn ui(&mut self, ui: &mut Ui,current_file:&mut String,content:&mut main_area::Content,vault:&str) -> Response {
        let markers_plot = Plot::new("Graph")
            .data_aspect(1.0)
            .allow_drag(self.dragable)
            .show_axes([false,false])
            .label_formatter(|name, value| {
                if !name.is_empty() {
                    format!("here be a point{}: {:.*}%", name, 1, value.y)
                } else {
                    "".to_owned()
                }
            });

        markers_plot.show(ui, |plot_ui| {
            self.dragable=true;
                let mut index = 0;
                if self.points.len()==0{
                    return;
                }
                for point in &self.points {
                    let point_color = self.tags_to_color(&point.tags);
                    let punto=Points::new([self.points_coord[index].0 as f64,self.points_coord[index].1 as f64])
                    .radius(9.0)
                    .color(point_color)
                    .shape(MarkerShape::Circle);
                    plot_ui.points(punto);
                    let bounds = plot_ui.plot_bounds();
                    let diff=bounds.max()[1]-bounds.min()[1];
                    if diff<6.0{
                        let texto = Text::new(PlotPoint::new(self.points_coord[index].0, 
                                                             (self.points_coord[index].1)-(diff*0.02) as f32), &point.text);
                        plot_ui.text(texto);
                    }
                    if plot_ui.plot_clicked() && is_close(plot_ui.pointer_coordinate(),self.points_coord[index],0.05){
                        *current_file=format!("{}/{}",vault,&self.points[index].text);
                        *content=main_area::Content::View;
                    }
                    if is_close(plot_ui.pointer_coordinate(),self.points_coord[index],0.05){
                        self.dragable=false;
                     nueva_ubicacion(plot_ui.pointer_coordinate_drag_delta(),&mut self.points_coord[index]);
                    }
                    index+=1;
                }
                self.drag_delta = plot_ui.pointer_coordinate_drag_delta();
            })
            .response
    }
    fn tags_to_color(&self, point_tags:&[String])->Color32{
        if point_tags.len() == 0 {
            return self.orphan_color;
        }
        let mut matches:Vec<(u8,u8,u8)> = vec![];
        for tag in point_tags{
            match self.tags_colors.get(tag) {
                Some(v) => matches.push((v.r(),v.g(),v.b())),
                None => continue,
            }
        }
        if matches.len() == 0 {
            return self.orphan_color;
        }
        let mut r:u8 = matches[0].0;
        let mut g:u8 = matches[0].1;
        let mut b:u8 = matches[0].2;
    
        for color in matches{
            r=(r/2)+(color.0/2);
            g=(g/2)+(color.1/2);
            b=(b/2)+(color.2/2);
        }
        Color32::from_rgb(r,g,b)
    }
//fn update_vault_points(){
//    
//}

//fn move_points(){
//}
}

fn is_close(delta:Option<PlotPoint>, point_pos:(f32,f32), tol:f32)->bool{
    match delta {
        Some(v) => {
        if ((v.x as f32-point_pos.0).abs() < tol) && ((v.y  as f32 - point_pos.1).abs() < tol){
            return true;
        }else{
            return false;
        }},
        None => return false,
    }
}
fn nueva_ubicacion(val:Vec2,punto:&mut (f32,f32)){
    *punto = (punto.0+(val.x as f32),punto.1+(val.y as f32));
}


fn get_data(dir:&Path,marmol_vec:&mut Vec<MarmolPoint>,total_entries:&mut i32,vault:&str){
    if Path::new(vault).exists()==false{return;}
            //tags:&mut Vec<String>){

    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_dir() {
            get_data(&path,marmol_vec,total_entries,vault);
        } else {
            if let Some(ext) = path.extension() {
                if ext == "md" {
                    let filename = path.to_str().unwrap().replace(vault,"");
                    *total_entries+=1;
                    let content = files::read_file(path.to_str().unwrap());
                    let (content,_)=files::contents(&content);
                    let content = YamlLoader::load_from_str(&content).unwrap_or(
                        {
                            let point = MarmolPoint::new(&filename,["Orphan".to_owned()].to_vec());
                            marmol_vec.push(point);
                            continue;
                        }
                        );
                    let content = &content[0];
                    if content["tags"].is_badvalue(){
                        if content["Tags"].is_badvalue(){
                            let point = MarmolPoint::new(&filename,["Orphan".to_owned()].to_vec());
                            marmol_vec.insert(0,point);
                        }else{
                            let mut tag_vecs=vec![];
                            for tag in content["Tags"].as_vec().unwrap(){
                                tag_vecs.push(tag.as_str().unwrap().to_owned());
                            }
                            let point = MarmolPoint::new(&filename,tag_vecs);
                            marmol_vec.push(point);
                        }
                    }else{
                        let mut tag_vecs=vec![];
                        for tag in content["tags"].as_vec().unwrap(){
                            tag_vecs.push(tag.as_str().unwrap().to_owned());
                        }
                        let point = MarmolPoint::new(&filename,tag_vecs);
                        marmol_vec.push(point);
                    }
                }
            }
        }
    }
}
fn get_coords(coords_vec:&mut Vec<(f32,f32)>,total_entries:i32){
    let elementos = total_entries as f32;
    let radio=2.;
    let var = 3.14*2.0;
    
    //if elements<10{
    //}else if elements<=30{
    //    for i in 0..elements{
    //        let a = (var/elementos)* i as f32;
    //        let x:f32= radio * a.cos();
    //        let y:f32= radio * a.sin();
    //        let punto:(f32,f32)=(x,y);
    //        coords_vec.push(punto);
    //    }
    //}else if elements<=70{
    //}else{}
    for i in 0..total_entries{
        let a = (var/elementos)* i as f32;
        let x:f32= radio * a.cos();
        let y:f32= radio * a.sin();
        coords_vec.push((x,y));
    }
}
#[derive(Debug)]
struct JsonConfiguration{
    show_orphans:bool,
    show_tags:bool,
    color_groups:Vec<ColorGroup>,
    center_strength:u8,
    repel_strength:u8,
}


#[derive(Debug)]
struct ColorGroup{
    tag:String,
    rgb:(u8,u8,u8)
}

pub fn get_data_uni()->String{
    let file = "./test.json";
    let file_content = fs::read_to_string(file).expect("Should have been able to read the file");
    let parsed = json::parse(&file_content).unwrap();
    let tag_values = &parsed["colorGroups"];
    let mut tags:Vec<ColorGroup> = Vec::new();
    for i in 0..tag_values.len(){
        let rgb_rec:(u8,u8,u8) = (tag_values[i]["rgb"][0].as_u8().unwrap(),
                                tag_values[i]["rgb"][1].as_u8().unwrap(),
                                tag_values[i]["rgb"][2].as_u8().unwrap() );
        tags.push(
            ColorGroup{tag: tag_values[i]["tag"].as_str().unwrap().to_string(), rgb:rgb_rec}
            );
    }
    let config = JsonConfiguration{
        show_orphans:parsed["showOrphans"].as_bool().unwrap(),
        show_tags:parsed["showTags"].as_bool().unwrap(),
        color_groups:tags,
        center_strength:parsed["centerStrength"].as_u8().unwrap(),
        repel_strength:parsed["repelStrength"].as_u8().unwrap(),
    };
    let mut text= String::new();
    for i in config.color_groups{
        text = format!("{}
        'tag':'{}'
        'rgb':'[{},{},{}]'
                       ",text,i.tag,i.rgb.0,i.rgb.1,i.rgb.2);
    }
    let config_text = format!("
{{
  'showTags': {},
  'showOrphans': {},
  'colorGroups': [
    {{
         {}
    }}
  ],
  'centerStrength': {},
  'repelStrength': {}
}}
 ",config.show_tags,config.show_orphans,text,config.center_strength,config.repel_strength);
    config_text
}

fn _distance(a: (f32, f32), b: (f32, f32)) -> f32 {
    let dx = a.0 - b.0;
    let dy = a.1 - b.1;
    (dx*dx + dy*dy).sqrt()
}

fn get_hex_string(num: u64) -> (u8,u8,u8) {
  let hex_string = format!("{:x}", num);
let hex_string = format!("{:0<6}", hex_string);
 let red = u8::from_str_radix(&hex_string[0..2], 16).ok();
    let green = u8::from_str_radix(&hex_string[2..4], 16).ok();
    let blue = u8::from_str_radix(&hex_string[4..6], 16).ok();
    (red.unwrap(), green.unwrap(), blue.unwrap())
}
