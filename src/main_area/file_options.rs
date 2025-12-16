use std::fs;
use std::io::Write;
use std::path::Path;
use std::fs::File;

use eframe::egui::{Button, Color32, Popup, PopupCloseBehavior, RichText};
use egui::Id;
use json::{object, JsonValue};

pub fn file_options(
    ui: &mut egui::Ui,
    s: &str,
    _path: &str,
    rename: &mut String,
    renaming_path: &mut Option<String>,
    error: &mut String,
    vault: &str,
) {
    let stared_path = format!("{}/.obsidian/starred.json", vault);

    ui.label(RichText::new(&*error).color(Color32::RED));
    let copy = egui::Button::new("Copy file").frame(false);
    let star = egui::Button::new("Star this file").frame(false);
    let path_s = Path::new(s).file_name().unwrap();

    ui.label("Move");

    if ui.add(copy).clicked() {
        let tmp = s.to_owned() + ".copy";
        let s_copy = Path::new(&tmp);
        let copy = fs::copy(s, &s_copy);
        match copy {
            Ok(_) => {
                *error = String::new();
                ui.close();
            }
            Err(r) => *error = r.to_string(),
        }
    }

    if ui.add(star).clicked() {
        let nw_json = object! {
            "type":"file",
            "title":Path::new(path_s).file_stem().unwrap().to_str().unwrap(),
            "path":"testi"
        };
        if Path::new(&stared_path).exists() {
            let contents =
                fs::read_to_string(&stared_path).expect("Should have been able to read the file");
            let mut parsed = json::parse(&contents).unwrap();
            let json_arr: &mut JsonValue = &mut parsed["items"];
            json_arr.push(nw_json).unwrap();
            let mut f = std::fs::OpenOptions::new()
                .write(true)
                .truncate(true)
                .open(stared_path)
                .unwrap();
            f.write_all(parsed.pretty(2).as_bytes()).unwrap();
            f.flush().unwrap();
        } else {
            let file = File::create(stared_path);
            match file {
                Ok(mut w) => {
                    let text = format!("{{ items:[{}] }}", nw_json.dump());
                    match w.write(text.as_bytes()) {
                        Ok(_) => *error = String::new(),
                        Err(r) => *error = r.to_string(),
                    }
                }
                Err(r) => *error = r.to_string(),
            }
        }
        ui.close();
    }

    if ui.button("Rename").clicked() {
        *renaming_path = Some(s.to_string());
        *rename = Path::new(s)
            .file_name()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();
        ui.close();
    }

    ui.separator();

    let id = ui.make_persistent_id(format!("del_{}", s));

    if ui.data(|d| d.get_temp(id).unwrap_or(false)) {
        ui.vertical_centered(|ui| {
            ui.label(RichText::new("Are you sure?").strong().color(Color32::RED));
            ui.label(RichText::new(path_s.to_str().unwrap()).italics().weak());
            ui.add_space(5.);

            ui.horizontal(|ui| {
                if ui.button("No").clicked() {
                    ui.data_mut(|d| d.insert_temp(id, false));
                }

                let btn_yes = egui::Button::new(RichText::new("Yes, Delete").color(Color32::WHITE))
                    .fill(Color32::RED);

                if ui.add(btn_yes).clicked() {
                    let delete = fs::remove_file(s);
                    match delete {
                        Ok(_) => {
                            *error = String::new();
                            ui.data_mut(|d| d.remove_temp::<bool>(id));
                            ui.close();
                        }
                        Err(r) => {
                            *error = r.to_string();
                        }
                    }
                }
            });
        });
    } else {
        let btn_delete =
            Button::selectable(false, RichText::new("Delete file").color(Color32::RED));

        if ui.add(btn_delete).clicked() {
            ui.data_mut(|d| d.insert_temp(id, true));
        }
    }
}

