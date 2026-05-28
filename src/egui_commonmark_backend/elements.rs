use egui::{self, NumExt, RichText, Sense, TextBuffer, TextStyle, Ui, Vec2, epaint};

#[inline]
pub fn rule(ui: &mut Ui, end_line: bool) {
    ui.add(egui::Separator::default().horizontal());
    // This does not add a new line, but instead ends the separator
    if end_line {
        newline(ui);
    }
}

#[inline]
pub fn soft_break(ui: &mut Ui) {
    ui.label(" ");
}

#[inline]
pub fn newline(ui: &mut Ui) {
    ui.label("\n");
}

pub fn bullet_point(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width_body_space(ui) * 4.0, height_body(ui)),
        Sense::hover(),
    );
    ui.painter().circle_filled(
        rect.center(),
        rect.height() / 6.0,
        ui.visuals().strong_text_color(),
    );
}

pub fn bullet_point_hollow(ui: &mut Ui) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width_body_space(ui) * 4.0, height_body(ui)),
        Sense::hover(),
    );
    ui.painter().circle(
        rect.center(),
        rect.height() / 6.0,
        egui::Color32::TRANSPARENT,
        egui::Stroke::new(0.6, ui.visuals().strong_text_color()),
    );
}

pub fn number_point(ui: &mut Ui, number: &str) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width_body_space(ui) * 4.0, height_body(ui)),
        Sense::hover(),
    );
    ui.painter().text(
        rect.right_center(),
        egui::Align2::RIGHT_CENTER,
        format!("{number}."),
        TextStyle::Body.resolve(ui.style()),
        ui.visuals().strong_text_color(),
    );
}

#[inline]
pub fn footnote_start(ui: &mut Ui, note: &str) {
    ui.label(RichText::new(note).raised().strong().small());
}

pub fn footnote(ui: &mut Ui, text: &str) {
    let (rect, _) = ui.allocate_exact_size(
        egui::vec2(width_body_space(ui) * 4.0, height_body(ui)),
        Sense::hover(),
    );
    ui.painter().text(
        rect.right_top(),
        egui::Align2::RIGHT_TOP,
        format!("{text}."),
        TextStyle::Small.resolve(ui.style()),
        ui.visuals().strong_text_color(),
    );
}

fn height_body(ui: &Ui) -> f32 {
    ui.text_style_height(&TextStyle::Body)
}

fn width_body_space(ui: &Ui) -> f32 {
    let id = TextStyle::Body.resolve(ui.style());
    ui.fonts(|f| f.glyph_width(&id, ' '))
}

/// Enhanced/specialized version of egui's code blocks. This one features copy button and borders
pub fn code_block<'t>(
    ui: &mut Ui,
    max_width: f32,
    text: &str,
    lang: Option<&str>,
    layouter: &'t mut dyn FnMut(&Ui, &dyn TextBuffer, f32) -> std::sync::Arc<egui::Galley>,
) {
    let mut text = text.strip_suffix('\n').unwrap_or(text);

    // ID único para este bloque basado en su contenido
    let block_id = ui.make_persistent_id(text);
    let mut is_collapsed = ui.data_mut(|d| *d.get_temp_mut_or_default::<bool>(block_id));

    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 0.0;

        // --- CABECERA ---
        let visuals = ui.style().visuals.widgets.noninteractive;
        let header_bg = ui.visuals().faint_bg_color; // Color ligeramente distinto al fondo
        
        let _header_response = egui::Frame::new()
            .fill(header_bg)
            .corner_radius(egui::CornerRadius {
                nw: 4,
                ne: 4,
                sw: if is_collapsed { 4 } else { 0 },
                se: if is_collapsed { 4 } else { 0 },
            })
            .stroke(visuals.bg_stroke)
            .inner_margin(egui::Margin::symmetric(8, 4))
            .show(ui, |ui| {
                ui.horizontal(|ui| {
                    // Flecha y Lenguaje (clicable para colapsar)
                    let arrow = if is_collapsed { "⏵" } else { "⏷" };
                    let label = format!("{} {}", arrow, lang.unwrap_or("Code"));
                    
                    if ui.selectable_label(false, label).clicked() {
                        is_collapsed = !is_collapsed;
                        ui.data_mut(|d| d.insert_temp(block_id, is_collapsed));
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        // Botón de Copiar en la esquina derecha
                        let persistent_id = block_id.with("copy_btn");
                        let copied_icon = ui.data_mut(|d| *d.get_temp_mut_or_default::<bool>(persistent_id));

                        let copy_btn = ui.button(if copied_icon { "✔" } else { "🗐" })
                            .on_hover_text("Copy code");

                        if copy_btn.clicked() {
                            ui.ctx().copy_text(text.to_owned());
                            ui.data_mut(|d| d.insert_temp(persistent_id, true));
                        }
                    });
                });
            }).response;

        // --- CONTENIDO ---
        if !is_collapsed {
            egui::Frame::new()
                .fill(ui.visuals().extreme_bg_color)
                .stroke(visuals.bg_stroke)
                .corner_radius(egui::CornerRadius {
                    nw: 0,
                    ne: 0,
                    sw: 4,
                    se: 4,
                })
                .inner_margin(egui::Margin::same(0)) // El margen lo da el TextEdit internamente
                .show(ui, |ui| {
                    let mut text_val = text;
                    ui.add(
                        egui::TextEdit::multiline(&mut text_val)
                            .layouter(layouter)
                            .desired_width(max_width)
                            .desired_rows(1)
                            .frame(false)
                    );
                });
        }
    });
}

// Stripped down version of egui's Checkbox. The only difference is that this
// creates a noninteractive checkbox. ui.add_enabled could have been used instead,
// but it makes the checkbox too grey.
pub struct ImmutableCheckbox<'a> {
    checked: &'a mut bool,
}

impl<'a> ImmutableCheckbox<'a> {
    pub fn without_text(checked: &'a mut bool) -> Self {
        ImmutableCheckbox { checked }
    }
}

impl egui::Widget for ImmutableCheckbox<'_> {
    fn ui(self, ui: &mut Ui) -> egui::Response {
        let ImmutableCheckbox { checked } = self;

        let spacing = &ui.spacing();
        let icon_width = spacing.icon_width;

        let mut desired_size = egui::vec2(icon_width, 0.0);
        desired_size = desired_size.at_least(Vec2::splat(spacing.interact_size.y));
        desired_size.y = desired_size.y.max(icon_width);
        let (rect, response) = ui.allocate_exact_size(desired_size, Sense::click());

        if ui.is_rect_visible(rect) {
            let visuals = ui.style().visuals.noninteractive();
            let (small_icon_rect, big_icon_rect) = ui.spacing().icon_rectangles(rect);
            ui.painter().add(epaint::RectShape::new(
                big_icon_rect.expand(visuals.expansion),
                visuals.corner_radius,
                visuals.bg_fill,
                visuals.bg_stroke,
                egui::StrokeKind::Inside,
            ));

            if *checked {
                // Check mark:
                ui.painter().add(egui::Shape::line(
                    vec![
                        egui::pos2(small_icon_rect.left(), small_icon_rect.center().y),
                        egui::pos2(small_icon_rect.center().x, small_icon_rect.bottom()),
                        egui::pos2(small_icon_rect.right(), small_icon_rect.top()),
                    ],
                    visuals.fg_stroke,
                ));
            }
        }

        response
    }
}

pub fn blockquote(ui: &mut Ui, accent: egui::Color32, add_contents: impl FnOnce(&mut Ui)) {
    let start = ui.painter().add(egui::Shape::Noop);
    let response = egui::Frame::new()
        // offset the frame so that we can use the space for the horizontal line and other stuff
        // By not using a separator we have better control
        .outer_margin(egui::Margin {
            left: 10,
            ..Default::default()
        })
        .show(ui, add_contents)
        .response;

    // FIXME: Add some rounding

    ui.painter().set(
        start,
        egui::epaint::Shape::line_segment(
            [
                egui::pos2(response.rect.left_top().x, response.rect.left_top().y + 5.0),
                egui::pos2(
                    response.rect.left_bottom().x,
                    response.rect.left_bottom().y - 5.0,
                ),
            ],
            egui::Stroke::new(3.0, accent),
        ),
    );
}
