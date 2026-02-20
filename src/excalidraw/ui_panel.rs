use egui::Ui;
use super::data::{ExcalidrawElement, ExcalidrawRoundness};
use super::utils::{hex_to_color, color_to_hex};

pub fn show_properties_panel(ui: &mut Ui, selected_element: Option<&mut ExcalidrawElement>, default_props: &mut ExcalidrawElement) -> bool {
    let mut ch = false;
    let props = if let Some(el) = selected_element {
        el
    } else {
        default_props
    };

    ui.vertical_centered(|ui| {
        ui.label(egui::RichText::new("Propiedades").size(18.0).strong());
    });
    ui.add_space(8.0);
    ui.separator();
    ui.add_space(8.0);

    egui::Grid::new("pgrid")
        .num_columns(2)
        .spacing([12.0, 16.0])
        .min_col_width(70.0)
        .striped(false)
        .show(ui, |ui| {
            // Borde
            ui.label("Borde:");
            ui.horizontal(|ui| {
                let mut c = hex_to_color(&props.stroke_color);
                if egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut c,
                    egui::color_picker::Alpha::Opaque,
                )
                .changed()
                {
                    props.stroke_color = color_to_hex(c);
                    ch = true;
                }
                ui.weak(&props.stroke_color);
            });
            ui.end_row();

            // Fondo
            ui.label("Fondo:");
            ui.horizontal(|ui| {
                let mut c = hex_to_color(&props.background_color);
                if egui::color_picker::color_edit_button_srgba(
                    ui,
                    &mut c,
                    egui::color_picker::Alpha::Opaque,
                )
                .changed()
                {
                    props.background_color = color_to_hex(c);
                    ch = true;
                }
                if ui.button("🚫").on_hover_text("Sin fondo").clicked() {
                    props.background_color = "transparent".into();
                    ch = true;
                }
            });
            ui.end_row();

            // Grosor
            ui.label("Grosor:");
            if ui
                .add(
                    egui::Slider::new(&mut props.stroke_width, 1..=20)
                        .show_value(true),
                )
                .changed()
            {
                ch = true;
            }
            ui.end_row();

            // Estilo
            ui.label("Estilo:");
            egui::ComboBox::from_id_salt("style")
                .selected_text(match props.stroke_style.as_str() {
                    "solid" => "Sólido",
                    "dashed" => "Guiones",
                    "dotted" => "Puntos",
                    _ => &props.stroke_style,
                })
                .width(130.0)
                .show_ui(ui, |ui| {
                    if ui
                        .selectable_value(
                            &mut props.stroke_style,
                            "solid".into(),
                            "Sólido ────",
                        )
                        .clicked()
                    {
                        ch = true;
                    }
                    if ui
                        .selectable_value(
                            &mut props.stroke_style,
                            "dashed".into(),
                            "Guiones ─ ─",
                        )
                        .clicked()
                    {
                        ch = true;
                    }
                    if ui
                        .selectable_value(
                            &mut props.stroke_style,
                            "dotted".into(),
                            "Puntos . . .",
                        )
                        .clicked()
                    {
                        ch = true;
                    }
                });
            ui.end_row();

            if props.element_type == "rectangle" || props.element_type == "diamond"
            {
                ui.label("Esquinas:");
                ui.horizontal(|ui| {
                    ui.style_mut().spacing.item_spacing.x = 2.0;
                    let is_r = props
                        .roundness
                        .as_ref()
                        .map(|r| r.round_type == 3)
                        .unwrap_or(false);

                    if ui
                        .add(egui::Button::selectable(!is_r, "Rectas"))
                        .clicked()
                    {
                        props.roundness = None;
                        ch = true;
                    }
                    if ui
                        .add(egui::Button::selectable(is_r, "Curvas"))
                        .clicked()
                    {
                        props.roundness =
                            Some(ExcalidrawRoundness { round_type: 3 });
                        ch = true;
                    }
                });
                ui.end_row();
            }

            // Opacidad
            ui.label("Opacidad:");
            if ui
                .add(
                    egui::Slider::new(&mut props.opacity, 0..=100)
                        .show_value(true)
                        .suffix("%"),
                )
                .changed()
            {
                ch = true;
            }
            ui.end_row();
        });

    ch
}
