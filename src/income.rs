use chrono::Local;
use core::ops::RangeInclusive;


use std::io::Write;
use std::collections::HashMap;
use serde::{Serialize,Deserialize};
use std::path::Path;
use std::fs;
use egui::*;
use plot::{Points,PlotPoints,MarkerShape,Line};

#[derive(PartialEq)]
enum Ventana { Normal, Graficos }

#[derive(Debug,Deserialize,Serialize,Clone,PartialEq)]
enum TipoMovimiento{
    Ingreso,
    Gasto,
    Null
}

#[derive(Serialize, Deserialize, Debug,PartialEq,Clone)]
pub struct Movimiento{
    fecha:String,
    tipo:TipoMovimiento,
    description:String,
    categoria:usize,
    monto:f32
}

impl Movimiento{
    fn new(fecha:String,tipo:TipoMovimiento,description:String,categoria:usize,monto:f32)->Movimiento{
        Movimiento{fecha,tipo,description,categoria,monto}
    }
}


#[derive(Serialize, Deserialize, Debug)]
pub struct Transacciones{ //Json content
    transacciones:Vec<Movimiento>,
    categorias:Vec<String>,
    colores:Vec<[f32;3]>,
}

//impl Transacciones{
//    fn sort(&self)->Vec<Movimiento>{
//        let mut g = self.transacciones.clone();
//        g.sort_by(|x,y| x.fecha.cmp(&y.fecha));
//        g
//    }
//}

pub fn load_data(path:&str)->Transacciones{ 
    let data = fs::read_to_string(Path::new(path))
            .expect("Unable to read file");
    let data: Transacciones = serde_json::from_str(&data).unwrap();
    data
}

#[derive(PartialEq)]
enum GraficaVer{Grafica,Elemento}

pub struct IncomeGui{
    json_content:Transacciones,
    path:String,
    categorias:HashMap<usize,i32>, //Primero el indice, luego la cantidad
    valor:usize,
    description:String,
    amount:String,
    fecha:String,
    error:String,
    edit:(i32,TipoMovimiento),
    ventana:Ventana,
    mov_sort:Vec<String>,
    points:Vec<[f64;2]>,
    lines:Vec<[f64;2]>,
    ingresos:HashMap<String,f64>, //Primero el indice, luego la cantidad
    cambiar:bool,
    editar_index:i32,
    categorias_string:String,
    max:f64,
    ver_gra:GraficaVer,
    ver_gra_i:usize,
}

impl Default for IncomeGui{
    fn default() -> Self {
        Self{
            json_content:Transacciones{transacciones:Vec::new(),categorias:Vec::new(),colores:Vec::new()},
            path:String::new(),
            categorias:HashMap::new(),
            valor:0,
            description:String::new(),
            amount:String::new(),
            fecha: Local::now().format("%Y-%m-%d").to_string(),
            error:String::new(),
            edit:(-1,TipoMovimiento::Null),
            ventana:Ventana::Normal,
            points:Vec::new(),
            lines:Vec::new(),
            mov_sort:Vec::new(),
            ingresos:HashMap::new(),
            cambiar:false,
            editar_index:-1,
            categorias_string:String::new(),
            max:0.0,
            ver_gra:GraficaVer::Grafica,
            ver_gra_i:0,
        }
    }
}

impl IncomeGui{
    pub fn set_data(&mut self, json_content:Transacciones){
        self.json_content=json_content;
        self.update_categorias();
        self.get_points();
    }

    fn get_points(&mut self){
        self.ingresos=HashMap::new();
        self.mov_sort=Vec::new();
        self.points=Vec::new();
        self.lines=Vec::new();
        self.max=0.0;
        for j in &self.json_content.transacciones{
            self.ingresos.entry(j.fecha.clone()).and_modify(|x|{
                    if j.tipo==TipoMovimiento::Ingreso{
                        *x+=j.monto as f64;
                    }else{
                        *x-=j.monto as f64;
                    }
            }).or_insert({
                    if j.tipo==TipoMovimiento::Ingreso{
                        j.monto as f64
                    }else{
                        (j.monto*-1.) as f64
                    }
            });
        }
        for i in self.ingresos.keys(){
            self.mov_sort.push(i.clone());
        }
        self.mov_sort.sort();
        let mut j = 0.0;
        let mut total = 0.0;
        for i in &self.mov_sort{
            if self.ingresos.get(i).unwrap().abs()>self.max{
                self.max=self.ingresos.get(i).unwrap().abs();
            }
            total+=*self.ingresos.get(i).unwrap();
            self.points.push([j,total]);
            j+=1.;
        }
        self.lines=self.points.clone();
    }

    fn update_categorias(&mut self){
        if !self.json_content.transacciones.is_empty(){
            self.valor = self.json_content.transacciones[0].categoria;
        }
        self.categorias=HashMap::new();
        for elemento in &self.json_content.transacciones{
            self.categorias.entry(elemento.categoria).and_modify(|x| *x += 1).or_insert(1);
        }
    }

    pub fn set_path(&mut self,path:&str) {
        if path!=self.path{
            println!("Datos actualizados");
            self.path=String::from(path);
            self.set_data(load_data(&self.path));
        }
    }

    pub fn show(&mut self, ui:&mut egui::Ui){
        let scroll = ScrollArea::vertical().max_height(350.0);
        self.escojer(ui);
        if self.ventana==Ventana::Normal{
            scroll.show(ui,|ui|{
                    self.vista_separada(ui);
            });
            ui.add_space(20.0);
            ui.horizontal(|ui|{
                ui.vertical(|ui|{
                    self.add_record(ui);
                });
                ui.vertical(|ui|{
                    self.categorias(ui);
                });
            //self.add_record(ui);
            });
        }else{
            self.grafica(ui);
            //self.canvas(ui);
        }
    }
    
    fn categorias(&mut self,ui:&mut egui::Ui){
        ui.group(|ui|{
            ui.separator();
            egui::ComboBox::from_label("Editar categoria")
            .selected_text(&self.json_content.categorias[self.valor])
            .show_ui(ui, |ui| {
            for (val,key) in self.json_content.categorias.iter().enumerate(){
                ui.horizontal(|ui|{
                    ui.selectable_value(&mut self.valor,val ,key);
                    if ui.add(Button::new("❌").frame(false)).on_hover_text("Delete").clicked(){
                        self.editar_index=val as i32;
                    }
                });
            }
            });
            egui::widgets::color_picker::color_edit_button_rgb(ui,&mut self.json_content.colores[self.valor]);
            ui.text_edit_singleline(&mut self.json_content.categorias[self.valor]);
            egui::CollapsingHeader::new("Añadir Categoria")
            .show(ui, |ui| {
                ui.text_edit_singleline(&mut self.categorias_string);
                if ui.button("Añadir categoria").clicked(){
                    self.json_content.categorias.push(self.categorias_string.clone());
                    self.json_content.colores.push([0.,0.,0.]);
                    self.categorias_string=String::new();
                }
            });
            if self.editar_index!=-1{
                if self.json_content.categorias.len()-1==0{
                    return;
                }
                self.json_content.categorias.remove(self.editar_index as usize);
                for g in &mut self.json_content.transacciones{
                    if g.categoria==self.editar_index as usize{
                        g.categoria=0;
                    }
                }
                self.editar_index= -1;
            }
        });
    }

    fn escojer(&mut self, ui:&mut egui::Ui){
        egui::TopBottomPanel::top("my_panel").show_inside(ui, |ui| {
            ui.horizontal(|ui|{
                ui.selectable_value(&mut self.ventana, Ventana::Normal, "Normal");
                ui.selectable_value(&mut self.ventana, Ventana::Graficos, "Graph");
            });
        });
    }

    pub fn vista_separada(&mut self, ui:&mut egui::Ui){
        let mut tot=0.0;
        let mut remove:i32=-1;
        ui.add_space(10.);
        ui.horizontal(|ui|{
            ui.vertical(|ui|{
                ui.add_sized([ui.available_width()*0.5,ui.available_height()],|ui:&mut egui::Ui|{
                    ui.vertical(|ui|{
                        ui.group(|ui|{
                            for (this,elemento) in self.json_content.transacciones.iter_mut().enumerate(){
                                let vals = array_to_color(self.json_content.colores[elemento.categoria]);
                                let f = Frame::none().fill(faded(vals,ui))
                                    .rounding(Rounding::same(2.0));
                                if self.edit==(this as i32,TipoMovimiento::Gasto){
                                    edit_valor(ui,elemento,&mut self.edit,&mut self.cambiar,&self.json_content.categorias);
                                    continue;
                                }
                                if elemento.tipo==TipoMovimiento::Gasto{
                                    tot-=elemento.monto;
                                    f.show(ui,|ui:&mut egui::Ui|{
                                        ui.label(
                                                RichText::new(&elemento.fecha).color(ui.visuals().strong_text_color())
                                            );
                                        ui.horizontal(|ui|{
                                            ui.label(
                                                RichText::new(format!("-{}",elemento.monto)).color(Color32::RED)
                                                );
                                            ui.label(&elemento.description);
                                        if ui.button("X").clicked(){remove=this as i32;}
                                        if ui.button("a").clicked(){self.edit=(this as i32,TipoMovimiento::Gasto);}
                                        });
                                    ui.separator();
                                    });
                                }
                            }
                            ui.separator();
                        });
                    });
                    ui.separator()
                });
            });
            ui.vertical(|ui|{
                ui.add(|ui:&mut egui::Ui|{
                    ui.group(|ui|{
                        for (this,elemento) in self.json_content.transacciones.iter_mut().enumerate(){
                            let vals = array_to_color(self.json_content.colores[elemento.categoria]);
                            let f = Frame::none().fill(faded(vals,ui))
                                .rounding(Rounding::same(2.0));
                            if self.edit==(this as i32,TipoMovimiento::Ingreso){
                                edit_valor(ui,elemento,&mut self.edit,&mut self.cambiar,&self.json_content.categorias);
                                continue;
                            }
                            if elemento.tipo==TipoMovimiento::Ingreso{
                                tot+=elemento.monto;
                                f.show(ui,|ui:&mut egui::Ui|{
                                    ui.label(
                                            RichText::new(&elemento.fecha).color(ui.visuals().strong_text_color())
                                        );
                                    ui.horizontal(|ui|{
                                        ui.label(
                                            RichText::new(format!("+{}",elemento.monto)).color(Color32::GREEN)
                                            );
                                        ui.label(&elemento.description);
                                        if ui.button("X").clicked(){remove=this as i32;}
                                        if ui.button("a").clicked(){self.edit=(this as i32,TipoMovimiento::Ingreso);}
                                    });
                                    ui.separator();
                                });
                            }
                        }
                        ui.separator()
                    });
                    ui.separator()
                });
            });
        });
        if remove!=-1{
            self.json_content.transacciones.remove(remove as usize);
            self.update_categorias();
        }
        if self.cambiar{
            self.get_points();
            self.cambiar=false;
        }
        ui.horizontal(|ui|{
            ui.label("Total:");
            if tot>0.0{
            ui.label(
                RichText::new(format!("{}",tot)).color(Color32::GREEN)
                 );
            }else{
            ui.label(
                RichText::new(format!("{}",tot)).color(Color32::RED)
                 );
            }
        });
    }


    pub fn save(&self){
        println!("Guardado");
        let file = String::from(&self.path);
        let mut file2 = fs::File::create(file).unwrap();
        let conts = serde_json::to_string(&self.json_content).unwrap();
        file2.write_all(conts.as_bytes()).unwrap();
    }

    pub fn add_record(&mut self, ui:&mut egui::Ui){
            ui.group(|ui|{
                ui.add_sized([ui.available_width()*0.7, 10.0],|ui:&mut egui::Ui|{ui.separator()});
                ui.label(RichText::new(&self.error).color(Color32::RED));
                ui.horizontal(|ui|{
                    egui::CollapsingHeader::new("Editar Fecha")
                    .show(ui, |ui| {
                        ui.add(egui::TextEdit::singleline(&mut self.fecha));
                    });
                });

                ui.add_space(10.0);

                ui.horizontal(|ui|{
                    ui.vertical(|ui|{
                        ui.label("Amount");
                        ui.add(egui::TextEdit::singleline(&mut self.amount));
                    });
                });

                ui.add_space(10.0);

                egui::ComboBox::from_label("Seleccionar categoria")
                .selected_text(&self.json_content.categorias[self.valor])
                .show_ui(ui, |ui| {
                    for (val,key) in self.json_content.categorias.iter().enumerate(){
                        ui.selectable_value(&mut self.valor, val, key);
                    }
                });

                ui.add_space(10.0);

                ui.vertical(|ui|{
                    ui.label("Description");
                    ui.add(egui::TextEdit::multiline(&mut self.description));
                });
                if ui.button("Añadir registro").clicked(){
                    if self.fecha.is_empty(){
                        self.error=String::from("Fecha Incompleta");
                        return;
                    }
                    let mut val:f32;
                    match self.amount.parse::<f32>(){
                        Ok(result)=>val=result,
                        Err(_)=>{self.error=String::from("Valor no valido");return;},
                    }
                    let tipo= if val<0.0{
                        val*=-1.0;
                        TipoMovimiento::Gasto
                    }else{
                        TipoMovimiento::Ingreso
                    };
                    self.json_content.transacciones.push(
                        Movimiento::new(self.fecha.clone(),tipo,self.description.clone(),self.valor,val)
                        );
                    self.fecha=Local::now().format("%Y-%m-%d").to_string();
                    self.description=String::new();
                    self.amount=String::new();
                    self.error=String::new();
                    self.update_categorias();
                    self.get_points();
                }
            });
    }

    fn canvas(&mut self, ui:&mut egui::Ui){
        let f = Frame::none().fill(Color32::BLACK).rounding(Rounding::same(3.0));
        //El canvas empieza desde 10,40
        f.show(ui,|ui|{
            let (_, painter) =
                ui.allocate_painter(Vec2::new(ui.available_width(), ui.available_height()), Sense::click());
            let fig2 = epaint::PathShape{points:vec![Pos2::new(200.,40.),Pos2::new(10.,200.),Pos2::new(200.,400.)],
                       stroke:Stroke::new(2.,Color32::RED),closed:false,fill:Color32::GREEN};
            let result = dibujar_arco(0,361,10.,200.,200.);
            let fig3 = epaint::PathShape{points:result,
                       stroke:Stroke::new(2.,Color32::RED),closed:false,fill:Color32::TRANSPARENT};
            painter.add(fig2);
            painter.add(fig3);
        });
    }
    
    fn grafica(&mut self, ui:&mut egui::Ui){
        if self.ver_gra==GraficaVer::Grafica{
            let mov_sort = self.mov_sort.clone();
            let g = move |x:f64, _:& RangeInclusive<f64>|->String{format!("{}",&mov_sort[x as usize])};
            let plot = egui::plot::Plot::new("items_demo")
                .show_x(false)
                .show_y(false)
                .clamp_grid(true)
                .auto_bounds_y()
                .auto_bounds_x()
                .x_axis_formatter(g);
            let p = PlotPoints::new(self.points.clone());
            let l = PlotPoints::new(self.lines.clone());
            let p2 = Points::new(p).shape(MarkerShape::Circle).radius(5.);
            let line = Line::new(l).fill(0.);
            plot.show(ui, |plot_ui| {
                plot_ui.line(line);
                plot_ui.points(p2);
                let mut j =0;
                for i in &self.points{
                    let pp = plot_ui.pointer_coordinate();
                    match pp{
                        Some(p)=>{
                            let x = i[0].max(p.x)-i[0].min(p.x);
                            let y = i[1].max(p.y)-i[1].min(p.y);
                            if plot_ui.plot_clicked() && x<0.1 && y<(self.max*0.05) {
                                self.ver_gra=GraficaVer::Elemento;
                                self.ver_gra_i=j;
                            }
                        },
                        None=>{}
                    }
                    j+=1;
                }
            });
        }else{
            if ui.button("Regresar").clicked(){
                self.ver_gra=GraficaVer::Grafica;
            }
            ui.label(RichText::from(&self.mov_sort[self.ver_gra_i]).color(Color32::WHITE));
            for j in &self.json_content.transacciones{
                    let vals = array_to_color(self.json_content.colores[j.categoria]);
                    let f = Frame::none().fill(faded(vals,ui))
                        .rounding(Rounding::same(2.0));
                if j.fecha==self.mov_sort[self.ver_gra_i]{
                    f.show(ui,|ui: &mut Ui|{
                    ui.heading(&j.description);
                    ui.label(&self.json_content.categorias[j.categoria]);
                    });
                    //ui.separator();
                };
            }
        }
    }
}

    fn edit_valor(ui:&mut egui::Ui,mov:&mut Movimiento, edit: &mut (i32,TipoMovimiento),
                  p:&mut bool,categorias_i:&[String],){
        let mut g = format!("{}",mov.monto);
        if ui.text_edit_singleline(&mut mov.description).changed() ||
        ui.text_edit_singleline(&mut mov.fecha).changed()||
        ui.text_edit_singleline(&mut g).changed(){
                    match g.parse::<f32>(){
                        Ok(result)=>mov.monto=result,
                        Err(_)=>{ui.colored_label(Color32::RED,"Valor no valido");},
                    }
            *p=true;
        }
        egui::ComboBox::from_label("Seleccionar categoria")
        .selected_text(&categorias_i[mov.categoria])
        .show_ui(ui, |ui| {
            for (val,key) in categorias_i.iter().enumerate(){
                ui.selectable_value(&mut mov.categoria,val ,key);
            }
        });
        if ui.button("Ok").clicked(){
            *edit=(-1,TipoMovimiento::Null);
            *p=true;
        }
    }

fn dibujar_arco(ang1:i32,ang2:i32,cx:f32,cy:f32,radio:f32)->Vec<Pos2>{
    let mut vect=Vec::new();
    let var = (std::f32::consts::PI *2.0)/360.;
    for i in ang1..ang2{
        let a = var* i as f32;
        let x:f32= radio * a.cos()+cx;
        let y:f32= radio * a.sin()+cy;
        vect.push(Pos2::new(x,y));
    }
    vect
}

fn array_to_color(arr:[f32;3])->Color32{
    let r = (255.*arr[0]) as u8;
    let g = (255.*arr[1]) as u8;
    let b = (255.*arr[2]) as u8;

    Color32::from_rgb(r,g,b)
}

fn faded(color:Color32,ui:&egui::Ui)->Color32{
        let dark_mode = ui.visuals().dark_mode;
        let faded_color = ui.visuals().window_fill();
        let t = if dark_mode { 0.95 } else { 0.8 };
        egui::lerp(Rgba::from(color)..=Rgba::from(faded_color), t).into()
}
