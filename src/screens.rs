use eframe::egui::{CentralPanel,RichText,Color32};
use std::path::Path;
use yaml_rust::Yaml;

#[derive(PartialEq)]
pub enum Screen{
Main,
Configuracion,
Default,
Server,
}

pub fn default(ctx:&egui::Context, current_window : &mut Screen, contenido:&mut String, nuevo:&mut String){
            let mut  open_bool=false;
            let mut  nuevo_bool=false;
            CentralPanel::default().show(ctx,|ui|{
                let text = RichText::new("Marmol").strong().size(60.0);
                ui.with_layout(egui::Layout::top_down(egui::Align::Center),|ui|{
                    ui.add_space(100.0);
                ui.label(text);
                    ui.add_space(100.0);
                ui.label("Select a Vault");
                ui.add(
                    egui::TextEdit::singleline( contenido )
                    );
                ui.add_space(10.0);
                if contenido.len()>2{
                    let path = Path::new(contenido);
                    let mut open_text = RichText::new("");
                    if path.exists(){
                        if path.is_dir(){
                            open_text = RichText::new("Good!").color(Color32::GREEN);
                            open_bool=true;
                        }else{
                            open_text = RichText::new("Path is not a dir").color(Color32::RED);
                        }
                    }else{
                            open_text = RichText::new("Path does not exists").color(Color32::RED);
                    }
                    ui.label(open_text);
                }
                if ui.button("Add vault").clicked() && open_bool{
                    unimplemented!();
                };
                    ui.add_space(30.0);
                ui.add(
                    egui::TextEdit::singleline( nuevo )
                    );
                if nuevo.len()>2{
                    let path = Path::new(nuevo);
                    let mut open_text = RichText::new("");
                    if !path.exists(){
                        if path.is_dir(){
                            open_text = RichText::new("Good!").color(Color32::GREEN);
                            nuevo_bool=true;
                        }
                    }else{
                            open_text = RichText::new("Path already exists").color(Color32::RED);
                    }
                    ui.label(open_text);
                }
                if ui.button("Create new Vault").clicked() && nuevo_bool{
                    unimplemented!();
                };
                    ui.add_space(30.0);
                if ui.button("configuration").clicked(){
                    *current_window=Screen::Configuracion;
                };
                 });
            });
}
pub fn configuracion(ctx:&egui::Context, current_window : &mut Screen, 
                     vaults:&Vec<Yaml>, vault:&mut String,){
            CentralPanel::default().show(ctx,|ui|{
                if ui.button("Select theme").clicked(){
                }
                if ui.button("Close Vault").clicked(){
                }
                if ui.button("Open new Vault").clicked(){
                }
                egui::CollapsingHeader::new("Open Vault").show(ui, |ui| {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for i in vaults{
                            let text= i.as_str();
                            match text{
                                None=>continue,
                                Some(stri)=>{
                                    if stri==vault{
                                        ui.label(stri);
                                    }else{
                                        if ui.button(stri).clicked(){
                                            *vault=String::from(stri);
                                        }
                                    }
                                }
                            }
                        }
                    });
                });
                if ui.button("return").clicked(){
                    *current_window=Screen::Main;
                };
                if ui.button("Configure Backup Server").clicked(){
                    *current_window=Screen::Server;
                };
            });
}

pub fn set_server(ctx:&egui::Context){
}
