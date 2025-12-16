use egui::{text::LayoutJob, Color32, TextFormat};
use yaml_rust::{YamlEmitter, YamlLoader};

pub fn create_metadata(metadata: String, ui: &mut egui::Ui) {
    let result = YamlLoader::load_from_str(&metadata);
    let Ok(docs) = result else {
        egui::Frame::group(ui.style())
            .fill(ui.visuals().faint_bg_color)
            .corner_radius(5.0)
            .inner_margin(10.0)
            .stroke(egui::Stroke::new(1.0, Color32::RED))
            .show(ui, |ui| {
                ui.set_width(ui.available_width());
                ui.label("Bad Formatting! :(");
            });
        ui.add_space(5.0);
        return;
    };
    let metadata_parsed = &docs[0];
    let mut job = LayoutJob::default();

    let mut out_str = String::new();
    let mut emitter = YamlEmitter::new(&mut out_str);
    emitter.dump(metadata_parsed).unwrap();
    out_str.split("\n").skip(1).for_each(|s| {
        if s.as_bytes()[s.len() - 1] == 58 {
            job.append(
                &(s.to_owned() + "\n"),
                0.0,
                TextFormat {
                    color: Color32::GRAY,
                    ..Default::default()
                },
            )
        } else if s.as_bytes()[0] == 32 {
            job.append(
                &(s.to_owned() + "\n"),
                0.0,
                TextFormat {
                    color: Color32::WHITE,
                    ..Default::default()
                },
            )
        } else {
            let mut splitted = s.split(" ");
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
                    color: Color32::GRAY,
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
                    color: Color32::WHITE,
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