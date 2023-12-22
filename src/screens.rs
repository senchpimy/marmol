use crate::main_area;
use crate::toggle_switch;
use eframe::egui::{Button, CentralPanel, Color32, FontId, RichText};
use egui::Widget;
use rfd::FileDialog;
use std::fs;
use std::path::Path;
use yaml_rust::Yaml;

#[derive(PartialEq, Copy, Clone)]
pub enum Screen {
    Main,
    Configuracion,
    Default,
    Server,
}

pub fn default(
    ctx: &egui::Context,
    current_window: &mut Screen,
    nuevo: &mut String,
    vaults_vec: &mut Vec<String>,
    vault: &mut String,
    content: &mut main_area::Content,
) {
    let mut nuevo_bool = false;
    CentralPanel::default().show(ctx, |ui| {
        let text = RichText::new("Marmol").strong().size(60.0);
        ui.with_layout(egui::Layout::top_down(egui::Align::Center), |ui| {
            ui.add_space(100.0);
            ui.label(text);
            ui.add_space(100.0);
            if ui.button("Select a Vault").clicked() {
                let files = FileDialog::new().set_title("Select a Folder").pick_folder();
                match files {
                    Some(x) => {
                        let selected_vault = x.to_str().unwrap();
                        vaults_vec.push(selected_vault.to_owned());
                        *vault = String::from(selected_vault);
                        *current_window = Screen::Main;
                        *content = main_area::Content::Blank;
                    }
                    None => {}
                }
            }
            ui.add_space(30.0);
            ui.add(egui::TextEdit::singleline(nuevo));
            if nuevo.len() > 2 {
                let path = Path::new(nuevo);
                let mut open_text = RichText::new("");
                if !path.exists() {
                    if path.is_dir() {
                        open_text = RichText::new("Good!").color(Color32::GREEN);
                        nuevo_bool = true;
                    }
                } else {
                    open_text = RichText::new("Path already exists").color(Color32::RED);
                }
                ui.label(open_text);
            }
            if ui.button("Create new Vault").clicked() && nuevo_bool {
                unimplemented!();
            };
            ui.add_space(30.0);
            if ui.button("configuration").clicked() {
                *current_window = Screen::Configuracion;
            };
        });
    });
}

pub fn configuracion(
    ctx: &egui::Context,
    prev_window: &mut Screen,
    current_window: &mut Screen,
    vaults: &mut Vec<String>,
    vault: &mut String,
    nw_vault_str: &mut String,
    show: &mut bool,
    folder: &mut String,
    error: &mut String,
    button: &mut bool,
    vault_changed: &mut bool,
    font_size: &mut f32,
    center_size: &mut f32,
    center_size_remain: &mut f32,
    sort_files: &mut bool,
) {
    CentralPanel::default().show(ctx, |ui| {
        if ui.button("Select theme").clicked() {}
        if ui.button("Create a New Vault").clicked() {
            let files = FileDialog::new().set_title("Select a Folder").pick_folder();
            match files {
                Some(x) => {
                    *show = true;
                    *folder = String::from(x.to_str().unwrap());
                }
                None => *show = false,
            }
        }
        if *show {
            let edit = egui::TextEdit::singleline(nw_vault_str);
            let response = ui.add(edit);
            if response.changed() {
                let full_path = format!("{}/{}", folder, nw_vault_str);
                let new_vault = Path::new(&full_path);
                if new_vault.exists() {
                    *error = String::from("Folder already Exists");
                    *button = false;
                } else {
                    *error = String::new();
                    *button = true;
                }
            }
        }
        if *button {
            if ui.button("Create!").clicked() {
                let full_path = format!("{}/{}", folder, nw_vault_str);
                vaults.push(full_path);
                let create = fs::create_dir(full_path);
                match create {
                    Ok(_) => {}
                    Err(x) => {
                        *error = x.to_string();
                        return;
                    }
                }
                let create = fs::create_dir(format!("{}/{}/.obsidian/", folder, nw_vault_str));
                match create {
                    Ok(_) => {}
                    Err(x) => {
                        *error = x.to_string();
                        return;
                    }
                }
                *nw_vault_str = String::new();
                *button = false;
                *show = false;
            }
        }
        ui.label(RichText::new(error.as_str()).color(Color32::RED));
        egui::CollapsingHeader::new("Manage Vault").show(ui, |ui| {
            let mut new_vaults = vaults.clone();
            let mut changed = false;
            egui::ScrollArea::vertical().show(ui, |ui| {
                for i in &mut *vaults {
                    let stri = i.as_str();
                            if stri == vault {
                                ui.label(stri);
                            } else {
                                let btn = Button::new(stri);
                                let menu = |ui: &mut egui::Ui| {
                                    remove_vault(ui, stri, &mut new_vaults, &mut changed)
                                };
                                if btn.ui(ui).context_menu(menu).clicked() {
                                    *vault = String::from(stri);
                                    *vault_changed = true;
                                }
                            }
                }
                if changed {
                    *vaults = new_vaults;
                }
            });
        });
        if ui.button("Add a Existing Vault").clicked() {
            let files = FileDialog::new().set_title("Select a Folder").pick_folder();
            match files {
                Some(x) => {
                    let selected_vault = x.to_str().unwrap().to_owned();
                    if !vaults.contains(&selected_vault) {
                        vaults.push(selected_vault.to_owned());
                        *vault = String::from(selected_vault);
                    };
                }
                None => {}
            }
        }
        ui.add_space(10.0);
        ui.add(toggle_switch::toggle_bool(sort_files));
        ui.label("Show files sorted");
        ui.add_space(10.0);
        if ui.button("Configure Backup Server").clicked() {
            *current_window = Screen::Server;
        };
        ui.add_space(10.0);

        if ui
            .add(egui::Slider::new(center_size, 0.35..=0.9).text("Line lenght"))
            .changed()
        {
            *center_size_remain = (1.0 - *center_size) / 2.0;
        };

        ui.add_space(10.0);
        if ui
            .add(egui::Slider::new(font_size, 10.0..=80.0).text("Font size"))
            .changed()
        {
            let mut style = (*ctx.style()).clone();
            let font_id = FontId::proportional(*font_size);
            style.override_font_id = Some(font_id);
            ctx.set_style(style);
        }
        ui.add_space(30.0);
        if ui.button("return").clicked() {
            *current_window = *prev_window;
        };
    });
}

fn remove_vault(ui: &mut egui::Ui, s: &str, vec: &mut Vec<String>, changed: &mut bool) {
    if ui.button("Delete").clicked() {
        vec.retain(|x| x != &s);
        *changed = true;
    }
    ui.label("This doens't delete the folder from your system, just from the program acces");
}

pub fn set_server(ctx: &egui::Context) {}
