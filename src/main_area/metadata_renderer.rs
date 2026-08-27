use egui::{text::LayoutJob, Color32, TextFormat};
use std::collections::HashMap;
use std::sync::{Arc, Mutex, OnceLock};
use yaml_rust::{YamlEmitter, YamlLoader};

/// Caché del YAML re-serializado (el parseo + emisión de YAML es costoso y se
/// repetía cada frame). Se cachea por cadena de metadata.
fn emitted_cache() -> &'static Mutex<HashMap<String, Arc<str>>> {
    static CACHE: OnceLock<Mutex<HashMap<String, Arc<str>>>> = OnceLock::new();
    CACHE.get_or_init(|| Mutex::new(HashMap::new()))
}

pub fn create_metadata(metadata: String, ui: &mut egui::Ui) {
    let emitted: Option<Arc<str>> = {
        let cache = emitted_cache();
        if let Some(s) = cache.lock().unwrap().get(&metadata) {
            Some(s.clone())
        } else {
            None
        }
    };

    let emitted = match emitted {
        Some(s) => s,
        None => {
            let result = YamlLoader::load_from_str(&metadata);
            let Ok(docs) = result else {
                egui::Frame::group(ui.style())
                    .fill(ui.visuals().faint_bg_color)
                    .corner_radius(5.0)
                    .inner_margin(10.0)
                    .stroke(egui::Stroke::new(1.0, ui.visuals().error_fg_color))
                    .show(ui, |ui| {
                        ui.set_width(ui.available_width());
                        ui.label("Bad Formatting! :(");
                    });
                ui.add_space(5.0);
                return;
            };

            if docs.is_empty() {
                return;
            }

            let mut out_str = String::new();
            let mut emitter = YamlEmitter::new(&mut out_str);
            emitter.dump(&docs[0]).unwrap();
            let s: Arc<str> = out_str.into();
            emitted_cache()
                .lock()
                .unwrap()
                .insert(metadata, s.clone());
            s
        }
    };

    let mut job = LayoutJob::default();

    emitted.split('\n').skip(1).for_each(|s| {
        if s.is_empty() {
            return;
        }
        if s.ends_with(':') {
            job.append(
                &(s.to_owned() + "\n"),
                0.0,
                TextFormat {
                    color: ui.style().visuals.widgets.inactive.fg_stroke.color,
                    ..Default::default()
                },
            )
        } else if s.starts_with(' ') {
            job.append(
                &(s.to_owned() + "\n"),
                0.0,
                TextFormat {
                    color: ui.style().visuals.override_text_color.unwrap_or(Color32::WHITE),
                    ..Default::default()
                },
            )
        } else {
            let mut splitted = s.split(' ');
            let mut content: &str;
            let mut text = splitted.next();
            match text {
                Some(x) => content = x,
                None => content = "Error parsing",
            }
            job.append(
                content,
                0.0,
                TextFormat {
                    color: ui.style().visuals.widgets.inactive.fg_stroke.color,
                    ..Default::default()
                },
            );
            text = splitted.next();
            match text {
                Some(x) => content = x,
                None => content = "Error parsing",
            }
            job.append(
                &format!("{}\n", content),
                0.0,
                TextFormat {
                    color: ui.style().visuals.override_text_color.unwrap_or(Color32::WHITE),
                    ..Default::default()
                },
            );
        }
    });
    egui::Frame::group(ui.style())
        .fill(ui.visuals().faint_bg_color)
        .corner_radius(5.0)
        .inner_margin(10.0)
        .stroke(egui::Stroke::new(
            1.0,
            ui.visuals().widgets.noninteractive.bg_stroke.color,
        ))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            ui.label(job);
        });
}