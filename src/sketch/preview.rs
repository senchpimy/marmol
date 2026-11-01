use crate::latex_math::MathBox;
use egui::{RichText, Sense, Ui};

pub fn latex_a_box(ui: &Ui, latex: &str, font_size: f32) -> Result<MathBox, String> {
    let node = crate::latex_math::parse_latex(latex)
        .ok_or_else(|| "LaTeX no soportado por el motor nativo".to_string())?;
    let family = egui::FontFamily::Name(crate::math_font::EGUI_FAMILY.into());
    let mut measurer = crate::latex_math::EguiMeasurer::new(ui, family);
    Ok(crate::latex_math::layout(
        &node,
        font_size,
        true,
        &mut measurer,
    ))
}

pub fn mostrar_preview(
    ui: &mut Ui,
    cache: &mut Option<(String, Result<MathBox, String>)>,
    latex: &str,
) {
    if latex.trim().is_empty() {
        return;
    }

    if crate::math_font::system_math_font_data().is_none() {
        ui.vertical_centered(|ui| {
            ui.label(
                RichText::new(latex)
                    .monospace()
                    .color(ui.visuals().weak_text_color()),
            );
        });
        return;
    }

    let cambiar = cache.as_ref().map(|(k, _)| k != latex).unwrap_or(true);

    if cambiar {
        *cache = Some((latex.to_string(), latex_a_box(ui, latex, 22.0)));
    }

    if let Some((_, resultado)) = cache.as_ref() {
        match resultado {
            Ok(math_box) => {
                let size = egui::vec2(math_box.width.max(1.0), math_box.height.max(1.0));
                let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
                crate::latex_math::render::draw(
                    ui,
                    math_box,
                    rect.min,
                    ui.visuals().text_color(),
                );
            }
            Err(e) => {
                ui.colored_label(egui::Color32::RED, format!("Error en el preview: {}", e));
            }
        }
    }
}
