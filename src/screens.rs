use eframe::egui::{CentralPanel,RichText,Color32,Button};
use std::fs;
use egui::Widget;
use std::path::Path;
use rfd::FileDialog;
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
                    let open_text:RichText;
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
                     vaults:&mut Vec<Yaml>, vault:&mut String, nw_vault_str:&mut String, show:&mut bool,
                     folder:&mut String, error:&mut String, button:&mut bool){
            CentralPanel::default().show(ctx,|ui|{
                if ui.button("Select theme").clicked(){
                }
                if ui.button("Create a New Vault").clicked(){
                    let files = FileDialog::new()
                        .set_title("Select a Folder")
                        .pick_folder();
                    match files{
                        Some(x)=>{
                            *show=true;
                            *folder=String::from(x.to_str().unwrap());
                        },
                        None=>*show=false
                    }
                }
                if *show{
                    let edit = egui::TextEdit::singleline(nw_vault_str);
                    let response = ui.add(edit);
                    if response.changed(){
                        let full_path=format!("{}/{}",folder,nw_vault_str); 
                        let new_vault = Path::new(&full_path);
                        if new_vault.exists(){
                            *error=String::from("Folder already Exists");
                            *button=false;
                        }else{
                            *error=String::new();
                            *button=true;
                        }
                            }
                }
                if *button{
                    if ui.button("Create!").clicked(){
                        let full_path=format!("{}/{}",folder,nw_vault_str); 
                        vaults.push(Yaml::from_str(&full_path));
                        let create = fs::create_dir(full_path);
                        match create{
                            Ok(_)=>{},
                            Err(x)=>{*error=x.to_string();return;}
                        }
                        let create = fs::create_dir(format!("{}/{}/.obsidian/",folder,nw_vault_str));
                        match create{
                            Ok(_)=>{},
                            Err(x)=>{*error=x.to_string();return;}
                        }
                        *nw_vault_str=String::new();
                        *button=false;
                        *show=false;
                    }
                }
                ui.label(RichText::new(error.as_str()).color(Color32::RED));
                egui::CollapsingHeader::new("Manage Vault").show(ui, |ui| {
                    let mut new_vaults=vaults.clone();
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        for i in &mut *vaults{
                            let text= i.as_str();
                            match text{
                                None=>continue,
                                Some(stri)=>{
                                    if stri==vault{
                                        ui.label(stri);
                                    }else{
                                        let btn = Button::new(stri);
                                        let menu = |ui:&mut egui::Ui| {remove_vault(ui,stri,&mut new_vaults)};
                                        if btn.ui(ui).context_menu(menu).clicked() {
                                            *vault=String::from(stri);
                                        }
                                    }
                                }
                            }
                        }
                        *vaults = new_vaults;
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

fn remove_vault(ui: &mut egui::Ui, s:&str,vec:&mut Vec<Yaml>) {
    if ui.button("Delete").clicked(){
            vec.retain(|x| x != &Yaml::from_str(s));
    }
    ui.label("This doens't delete the folder from your system, just from the program acces");
}

pub fn set_server(ctx:&egui::Context){
}
