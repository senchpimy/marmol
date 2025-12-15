use base64::{engine::general_purpose, Engine as _};
use egui::UiBuilder;
use egui::{
    emath::Rot2, Color32, ColorImage, Context, FontFamily, FontId, PointerButton, Pos2, Rect,
    Sense, Shape, Stroke, TextureHandle, TextureOptions, Ui, Vec2,
};
use image::imageops::FilterType;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;

#[derive(Clone, Copy, PartialEq, Debug)]
pub enum Tool {
    Selection,
    Rectangle,
    Ellipse,
    Diamond,
    Line,
    Arrow,
}

#[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
struct ExcalidrawRoundness {
    #[serde(rename = "type")]
    round_type: i32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct BoundElement {
    id: String,
    #[serde(rename = "type")]
    element_type: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExcalidrawFile {
    #[serde(rename = "mimeType")]
    mime_type: String,
    id: String,
    #[serde(rename = "dataURL")]
    data_url: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExcalidrawElement {
    #[serde(default)]
    id: String,
    #[serde(rename = "type")]
    element_type: String,
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    #[serde(default)]
    angle: f32,
    #[serde(default, rename = "strokeColor")]
    stroke_color: String,
    #[serde(default, rename = "backgroundColor")]
    background_color: String,
    #[serde(default, rename = "fillStyle")]
    fill_style: String,
    #[serde(default, rename = "strokeWidth")]
    stroke_width: f32,
    #[serde(default, rename = "strokeStyle")]
    stroke_style: String,
    #[serde(default)]
    opacity: f32,
    #[serde(default)]
    roughness: f32,
    #[serde(default)]
    points: Vec<[f32; 2]>,
    #[serde(default)]
    text: String,
    #[serde(default, rename = "fontSize")]
    font_size: f32,
    #[serde(default)]
    roundness: Option<ExcalidrawRoundness>,
    #[serde(default, rename = "endArrowhead")]
    end_arrowhead: Option<String>,
    #[serde(default, rename = "boundElements")]
    bound_elements: Option<Vec<BoundElement>>,
    #[serde(default, rename = "containerId")]
    container_id: Option<String>,
    #[serde(default, rename = "fileId")]
    file_id: Option<String>,
    #[serde(default)]
    scale: Option<[f32; 2]>,
}

impl Default for ExcalidrawElement {
    fn default() -> Self {
        Self {
            id: "".to_string(),
            element_type: "rectangle".to_string(),
            x: 0.0,
            y: 0.0,
            width: 0.0,
            height: 0.0,
            angle: 0.0,
            stroke_color: "#000000".to_string(),
            background_color: "transparent".to_string(),
            fill_style: "solid".to_string(),
            stroke_width: 1.0,
            stroke_style: "solid".to_string(),
            opacity: 100.0,
            roughness: 1.0,
            points: vec![],
            text: "".to_string(),
            font_size: 20.0,
            roundness: Some(ExcalidrawRoundness { round_type: 3 }),
            end_arrowhead: None,
            bound_elements: None,
            container_id: None,
            file_id: None,
            scale: None,
        }
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
struct ExcalidrawScene {
    #[serde(default)]
    type_: String,
    #[serde(default)]
    version: i32,
    #[serde(default)]
    source: String,
    elements: Vec<ExcalidrawElement>,
    #[serde(default)]
    files: HashMap<String, ExcalidrawFile>,
    #[serde(flatten)]
    extra: serde_json::Map<String, serde_json::Value>,
}

#[derive(Serialize, Deserialize)]
pub struct ExcalidrawGui {
    path: String,
    #[serde(skip)]
    scene: Option<ExcalidrawScene>,
    #[serde(skip)]
    error_msg: Option<String>,
    pan: Vec2,
    scale: f32,
    #[serde(skip)]
    active_tool: Option<Tool>,
    #[serde(skip)]
    selected_element_idx: Option<usize>,
    #[serde(skip)]
    default_props: ExcalidrawElement,
    #[serde(skip)]
    dragged_element_idx: Option<usize>,
    #[serde(skip)]
    last_mouse_pos_world: Option<Pos2>,
    #[serde(skip)]
    drawing_start_pos: Option<Pos2>,
    #[serde(skip)]
    drawing_element: Option<ExcalidrawElement>,
    #[serde(skip)]
    is_dirty: bool,
    #[serde(skip)]
    is_panning: bool,
    #[serde(skip)]
    texture_cache: HashMap<String, TextureHandle>,
    #[serde(skip)]
    failed_textures: HashSet<String>,
}

impl Default for ExcalidrawGui {
    fn default() -> Self {
        Self {
            path: String::new(),
            scene: None,
            error_msg: None,
            pan: Vec2::ZERO,
            scale: 1.0,
            active_tool: Some(Tool::Selection),
            selected_element_idx: None,
            default_props: ExcalidrawElement::default(),
            dragged_element_idx: None,
            last_mouse_pos_world: None,
            drawing_start_pos: None,
            drawing_element: None,
            is_dirty: false,
            is_panning: false,
            texture_cache: HashMap::new(),
            failed_textures: HashSet::new(),
        }
    }
}

impl ExcalidrawGui {
    pub fn set_path(&mut self, path: &str) {
        if self.path != path || self.scene.is_none() {
            self.path = path.to_string();
            self.reload();
        }
    }

    pub fn reload(&mut self) {
        if self.path.is_empty() {
            return;
        }
        match fs::read_to_string(&self.path) {
            Ok(json) => match serde_json::from_str::<ExcalidrawScene>(&json) {
                Ok(mut sc) => {
                    for (i, el) in sc.elements.iter_mut().enumerate() {
                        if el.id.is_empty() {
                            el.id = format!("gen_{}", i);
                        }
                    }
                    self.scene = Some(sc);
                    self.error_msg = None;
                    self.is_dirty = false;
                    self.texture_cache.clear();
                    self.failed_textures.clear();
                    if self.scale == 0.0 {
                        self.scale = 1.0;
                    }
                    if self.active_tool.is_none() {
                        self.active_tool = Some(Tool::Selection);
                    }
                }
                Err(e) => self.error_msg = Some(format!("Error parsing: {}", e)),
            },
            Err(e) => self.error_msg = Some(format!("Error reading: {}", e)),
        }
    }

    fn save_file(&mut self) {
        if let Some(scene) = &self.scene {
            if let Ok(json) = serde_json::to_string_pretty(scene) {
                let _ = fs::write(&self.path, json);
                self.is_dirty = false;
            }
        }
    }

    fn get_or_load_texture(
        &mut self,
        ctx: &Context,
        fid: &str,
        files: &HashMap<String, ExcalidrawFile>,
    ) -> Option<TextureHandle> {
        if let Some(h) = self.texture_cache.get(fid) {
            return Some(h.clone());
        }
        if self.failed_textures.contains(fid) {
            return None;
        }
        if let Some(fd) = files.get(fid) {
            if fd.mime_type.contains("svg") {
                self.failed_textures.insert(fid.to_string());
                return None;
            }
            let parts: Vec<&str> = fd.data_url.split(',').collect();
            if parts.len() == 2 {
                if let Ok(b) = general_purpose::STANDARD.decode(parts[1]) {
                    if let Ok(mut img) = image::load_from_memory(&b) {
                        if img.width() > 1024 || img.height() > 1024 {
                            img = img.resize(1024, 1024, FilterType::Triangle);
                        }
                        let t = ctx.load_texture(
                            fid,
                            ColorImage::from_rgba_unmultiplied(
                                [img.width() as _, img.height() as _],
                                img.to_rgba8().as_flat_samples().as_slice(),
                            ),
                            TextureOptions::LINEAR,
                        );
                        self.texture_cache.insert(fid.to_string(), t.clone());
                        return Some(t);
                    }
                }
            }
        }
        self.failed_textures.insert(fid.to_string());
        None
    }

    pub fn show(&mut self, ui: &mut Ui) {
        if let Some(e) = &self.error_msg {
            ui.colored_label(Color32::RED, e);
            return;
        }
        if self.scene.is_none() {
            self.reload();
            if self.scene.is_none() {
                ui.label("Loading...");
                return;
            }
        }

        ui.horizontal(|ui| {
            ui.label("Tools:");
            if ui
                .selectable_label(self.active_tool == Some(Tool::Selection), "✋")
                .clicked()
            {
                self.active_tool = Some(Tool::Selection);
            }
            ui.separator();
            if ui
                .selectable_label(self.active_tool == Some(Tool::Rectangle), "⬜")
                .clicked()
            {
                self.active_tool = Some(Tool::Rectangle);
            }
            if ui
                .selectable_label(self.active_tool == Some(Tool::Ellipse), "⭕")
                .clicked()
            {
                self.active_tool = Some(Tool::Ellipse);
            }
            if ui
                .selectable_label(self.active_tool == Some(Tool::Diamond), "🔶")
                .clicked()
            {
                self.active_tool = Some(Tool::Diamond);
            }
            ui.separator();
            if ui
                .selectable_label(self.active_tool == Some(Tool::Line), "➖")
                .clicked()
            {
                self.active_tool = Some(Tool::Line);
            }
            if ui
                .selectable_label(self.active_tool == Some(Tool::Arrow), "➡")
                .clicked()
            {
                self.active_tool = Some(Tool::Arrow);
            }
            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                if ui.button("💾").clicked() {
                    self.save_file();
                }
            });
        });

        let (response, painter) = ui.allocate_painter(ui.available_size(), Sense::click_and_drag());
        painter.rect_filled(response.rect, 0.0, Color32::WHITE);

        let panel_width = 220.0;
        let panel_rect = Rect::from_min_size(
            Pos2::new(
                response.rect.max.x - panel_width - 10.0,
                response.rect.min.y + 10.0,
            ),
            Vec2::new(panel_width, 400.0),
        );

        let scroll_delta = ui.input(|i| i.raw_scroll_delta);
        let screen_rect_min = response.rect.min;
        if ui.input(|i| i.modifiers.ctrl) && scroll_delta.y != 0.0 {
            let zf = if scroll_delta.y > 0.0 { 1.1 } else { 0.9 };
            let old_s = self.scale;
            self.scale = (self.scale * zf).clamp(0.1, 10.0);
            if let Some(mp) = response.hover_pos() {
                let mc = mp - screen_rect_min;
                self.pan = mc - ((mc - self.pan) / old_s) * self.scale;
            }
        } else if scroll_delta != Vec2::ZERO {
            self.pan += scroll_delta;
        }

        if response.dragged_by(PointerButton::Middle)
            || (ui.input(|i| i.modifiers.alt) && response.dragged_by(PointerButton::Primary))
        {
            self.pan += response.drag_delta();
            self.is_panning = true;
        } else {
            self.is_panning = false;
        }

        let cp = self.pan;
        let cs = self.scale;
        let to_screen = move |pw: Pos2| screen_rect_min + cp + (pw.to_vec2() * cs);
        let to_world = move |ps: Pos2| Pos2::ZERO + (ps - screen_rect_min - cp) / cs;
        let vis_rect = Rect::from_min_max(to_world(response.rect.min), to_world(response.rect.max));

        if let Some(mut scene) = self.scene.take() {
            let mut dirty = self.is_dirty;
            let mut save = false;
            let tool = self.active_tool.unwrap_or(Tool::Selection);

            let mouse_over_panel = if let Some(mp) = response.hover_pos() {
                panel_rect.contains(mp)
            } else {
                false
            };

            if !mouse_over_panel {
                match tool {
                    Tool::Selection => {
                        if response.drag_started_by(PointerButton::Primary)
                            && !self.is_panning
                            && !ui.input(|i| i.modifiers.alt)
                        {
                            if let Some(mp) = response.interact_pointer_pos() {
                                let wp = to_world(mp);
                                let mut f = false;
                                for (i, el) in scene.elements.iter().enumerate().rev() {
                                    if is_point_inside(el, wp) {
                                        self.dragged_element_idx = Some(i);
                                        self.selected_element_idx = Some(i);
                                        self.last_mouse_pos_world = Some(wp);
                                        f = true;
                                        break;
                                    }
                                }
                                if !f {
                                    self.selected_element_idx = None;
                                }
                            }
                        }
                        if let Some(idx) = self.dragged_element_idx {
                            if response.dragged_by(PointerButton::Primary) {
                                if let Some(mp) = response.interact_pointer_pos() {
                                    let cwp = to_world(mp);
                                    if let Some(lp) = self.last_mouse_pos_world {
                                        let d = cwp - lp;
                                        if d.length_sq() > 0.0001 {
                                            move_element_group(&mut scene.elements, idx, d);
                                            dirty = true;
                                        }
                                    }
                                    self.last_mouse_pos_world = Some(cwp);
                                }
                            }
                        }
                    }
                    _ => {
                        // Dibujo
                        if response.drag_started_by(PointerButton::Primary)
                            && !self.is_panning
                            && !ui.input(|i| i.modifiers.alt)
                        {
                            if let Some(mp) = response.interact_pointer_pos() {
                                let sw = to_world(mp);
                                self.drawing_start_pos = Some(sw);
                                self.selected_element_idx = None;
                                let mut new_el = self.default_props.clone();
                                let ts = std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_nanos();
                                new_el.id = format!("gen_{}", ts);
                                new_el.x = sw.x;
                                new_el.y = sw.y;
                                new_el.points = vec![];
                                match tool {
                                    Tool::Line => {
                                        new_el.element_type = "line".into();
                                        new_el.points = vec![[0.0, 0.0]];
                                    }
                                    Tool::Arrow => {
                                        new_el.element_type = "arrow".into();
                                        new_el.points = vec![[0.0, 0.0]];
                                        new_el.end_arrowhead = Some("arrow".into());
                                    }
                                    Tool::Rectangle => new_el.element_type = "rectangle".into(),
                                    Tool::Ellipse => new_el.element_type = "ellipse".into(),
                                    Tool::Diamond => new_el.element_type = "diamond".into(),
                                    _ => {}
                                }
                                self.drawing_element = Some(new_el);
                            }
                        }
                        if let Some(sp) = self.drawing_start_pos {
                            if response.dragged_by(PointerButton::Primary) {
                                if let Some(mp) = response.interact_pointer_pos() {
                                    let cw = to_world(mp);
                                    if let Some(el) = &mut self.drawing_element {
                                        if tool == Tool::Line || tool == Tool::Arrow {
                                            let dx = cw.x - sp.x;
                                            let dy = cw.y - sp.y;
                                            el.points = vec![[0.0, 0.0], [dx, dy]];
                                            el.width = dx.abs();
                                            el.height = dy.abs();
                                        } else {
                                            el.x = sp.x.min(cw.x);
                                            el.y = sp.y.min(cw.y);
                                            el.width = (sp.x - cw.x).abs();
                                            el.height = (sp.y - cw.y).abs();
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if response.drag_stopped() {
                self.dragged_element_idx = None;
                self.last_mouse_pos_world = None;
                if let Some(el) = self.drawing_element.take() {
                    if el.width > 2.0 || el.height > 2.0 {
                        scene.elements.push(el);
                        self.selected_element_idx = Some(scene.elements.len() - 1);
                        dirty = true;
                        save = true;
                    }
                    self.active_tool = Some(Tool::Selection);
                }
                if dirty && self.dragged_element_idx.is_some() {
                    save = true;
                }
                self.drawing_start_pos = None;
            }

            let ctx = ui.ctx().clone();
            let files = &scene.files;
            for (i, el) in scene.elements.iter().enumerate() {
                let rect =
                    Rect::from_min_size(Pos2::new(el.x, el.y), Vec2::new(el.width, el.height))
                        .expand(100.0);
                if !vis_rect.intersects(rect) {
                    continue;
                }
                let tex = if let Some(fid) = &el.file_id {
                    self.get_or_load_texture(&ctx, fid, files)
                } else {
                    None
                };
                draw_element(&painter, el, tex, &to_screen, cs);
                if self.selected_element_idx == Some(i) {
                    draw_selection_border(&painter, el, &to_screen, cs);
                }
            }
            if let Some(el) = &self.drawing_element {
                draw_element(&painter, el, None, &to_screen, cs);
            }

            self.scene = Some(scene);
            self.is_dirty = dirty;
            if save {
                self.save_file();
            }

            ui.scope_builder(UiBuilder::new().max_rect(panel_rect), |ui| {
                egui::Frame::NONE
                    .fill(Color32::from_rgba_premultiplied(30, 30, 30, 240))
                    .stroke(Stroke::new(1.0, Color32::from_gray(60)))
                    .corner_radius(12.0)
                    .inner_margin(16.0)
                    .show(ui, |ui| {
                        ui.set_width(panel_rect.width() - 32.0);

                        ui.vertical_centered(|ui| {
                            ui.label(egui::RichText::new("Propiedades").size(18.0).strong());
                        });
                        ui.add_space(8.0);
                        ui.separator();
                        ui.add_space(8.0);

                        let props_opt = if let Some(sc) = self.scene.as_mut() {
                            if let Some(idx) = self.selected_element_idx {
                                if idx < sc.elements.len() {
                                    Some(&mut sc.elements[idx])
                                } else {
                                    None
                                }
                            } else {
                                Some(&mut self.default_props)
                            }
                        } else {
                            None
                        };

                        if let Some(p) = props_opt {
                            let mut ch = false;

                            egui::Grid::new("pgrid")
                                .num_columns(2)
                                .spacing([12.0, 16.0]) // Más espacio entre elementos
                                .min_col_width(70.0)
                                .striped(false)
                                .show(ui, |ui| {
                                    // Borde
                                    ui.label("Borde:");
                                    ui.horizontal(|ui| {
                                        let mut c = hex_to_color(&p.stroke_color);
                                        if egui::color_picker::color_edit_button_srgba(
                                            ui,
                                            &mut c,
                                            egui::color_picker::Alpha::Opaque,
                                        )
                                        .changed()
                                        {
                                            p.stroke_color = color_to_hex(c);
                                            ch = true;
                                        }
                                        ui.weak(&p.stroke_color);
                                    });
                                    ui.end_row();

                                    // Fondo
                                    ui.label("Fondo:");
                                    ui.horizontal(|ui| {
                                        let mut c = hex_to_color(&p.background_color);
                                        if egui::color_picker::color_edit_button_srgba(
                                            ui,
                                            &mut c,
                                            egui::color_picker::Alpha::Opaque,
                                        )
                                        .changed()
                                        {
                                            p.background_color = color_to_hex(c);
                                            ch = true;
                                        }
                                        if ui.button("🚫").on_hover_text("Sin fondo").clicked() {
                                            p.background_color = "transparent".into();
                                            ch = true;
                                        }
                                    });
                                    ui.end_row();

                                    // Grosor
                                    ui.label("Grosor:");
                                    if ui
                                        .add(
                                            egui::Slider::new(&mut p.stroke_width, 0.5..=20.0)
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
                                        .selected_text(match p.stroke_style.as_str() {
                                            "solid" => "Sólido",
                                            "dashed" => "Guiones",
                                            "dotted" => "Puntos",
                                            _ => &p.stroke_style,
                                        })
                                        .width(130.0)
                                        .show_ui(ui, |ui| {
                                            if ui
                                                .selectable_value(
                                                    &mut p.stroke_style,
                                                    "solid".into(),
                                                    "Sólido ────",
                                                )
                                                .clicked()
                                            {
                                                ch = true;
                                            }
                                            if ui
                                                .selectable_value(
                                                    &mut p.stroke_style,
                                                    "dashed".into(),
                                                    "Guiones ─ ─",
                                                )
                                                .clicked()
                                            {
                                                ch = true;
                                            }
                                            if ui
                                                .selectable_value(
                                                    &mut p.stroke_style,
                                                    "dotted".into(),
                                                    "Puntos . . .",
                                                )
                                                .clicked()
                                            {
                                                ch = true;
                                            }
                                        });
                                    ui.end_row();

                                    if p.element_type == "rectangle" || p.element_type == "diamond"
                                    {
                                        ui.label("Esquinas:");
                                        ui.horizontal(|ui| {
                                            ui.style_mut().spacing.item_spacing.x = 2.0;
                                            let is_r = p
                                                .roundness
                                                .as_ref()
                                                .map(|r| r.round_type == 3)
                                                .unwrap_or(false);

                                            if ui
                                                .add(egui::Button::selectable(!is_r, "Rectas"))
                                                .clicked()
                                            {
                                                p.roundness = None;
                                                ch = true;
                                            }
                                            if ui
                                                .add(egui::Button::selectable(is_r, "Curvas"))
                                                .clicked()
                                            {
                                                p.roundness =
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
                                            egui::Slider::new(&mut p.opacity, 0.0..=100.0)
                                                .show_value(true)
                                                .suffix("%"),
                                        )
                                        .changed()
                                    {
                                        ch = true;
                                    }
                                    ui.end_row();
                                });

                            if ch {
                                self.is_dirty = true;
                                if self.selected_element_idx.is_some() {
                                    self.save_file();
                                }
                            }
                        } else {
                            ui.vertical_centered(|ui| {
                                ui.label("Selecciona un elemento");
                            });
                        }
                    });
            });
        }
    }
}

fn hex_to_color(hex: &str) -> Color32 {
    if hex == "transparent" {
        return Color32::TRANSPARENT;
    }
    let h = hex.trim_start_matches('#');
    if let Ok(n) = u32::from_str_radix(h, 16) {
        if h.len() == 6 {
            return Color32::from_rgb(
                ((n >> 16) & 0xFF) as u8,
                ((n >> 8) & 0xFF) as u8,
                (n & 0xFF) as u8,
            );
        }
    }
    Color32::BLACK
}

fn color_to_hex(c: Color32) -> String {
    if c == Color32::TRANSPARENT {
        return "transparent".into();
    }
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

fn move_element_group(elements: &mut Vec<ExcalidrawElement>, root_idx: usize, delta: Vec2) {
    let mut indices = vec![root_idx];
    let rid = elements[root_idx].id.clone();
    let cid = elements[root_idx].container_id.clone();
    if let Some(c) = cid {
        if let Some(p) = elements.iter().position(|e| e.id == c) {
            indices.push(p);
        }
    }
    if let Some(bs) = &elements[root_idx].bound_elements {
        for b in bs {
            if let Some(c) = elements.iter().position(|e| e.id == b.id) {
                indices.push(c);
            }
        }
    }
    for (i, el) in elements.iter().enumerate() {
        if let Some(c) = &el.container_id {
            if *c == rid && !indices.contains(&i) {
                indices.push(i);
            }
        }
    }
    indices.sort_unstable();
    indices.dedup();
    for idx in indices {
        if let Some(el) = elements.get_mut(idx) {
            el.x += delta.x;
            el.y += delta.y;
        }
    }
}

fn is_point_inside(el: &ExcalidrawElement, p: Pos2) -> bool {
    Rect::from_min_size(Pos2::new(el.x, el.y), Vec2::new(el.width, el.height))
        .expand(10.0)
        .contains(p)
}

fn draw_selection_border<F>(painter: &egui::Painter, el: &ExcalidrawElement, to_screen: &F, _s: f32)
where
    F: Fn(Pos2) -> Pos2,
{
    let c = Pos2::new(el.x + el.width / 2.0, el.y + el.height / 2.0);
    let cl = Pos2::new(el.width / 2.0, el.height / 2.0);
    let r = Rot2::from_angle(el.angle);
    let pts: Vec<Pos2> = [
        Pos2::new(0.0, 0.0),
        Pos2::new(el.width, 0.0),
        Pos2::new(el.width, el.height),
        Pos2::new(0.0, el.height),
    ]
    .iter()
    .map(|&p| to_screen(c + r * (p - cl)))
    .collect();
    painter.add(Shape::closed_line(
        pts,
        Stroke::new(1.0, Color32::from_rgb(100, 100, 255)),
    ));
}
fn draw_stroke(painter: &egui::Painter, pts: Vec<Pos2>, s: Stroke, st: &str, sc: f32, cl: bool) {
    if pts.len() < 2 {
        return;
    }
    let mut fp = pts.clone();
    if cl {
        fp.push(pts[0]);
    }
    match st {
        "dashed" => {
            painter.add(Shape::dashed_line(&fp, s, 10.0 * sc, 10.0 * sc));
        }
        "dotted" => {
            painter.add(Shape::dashed_line(&fp, s, 2.0 * sc, 6.0 * sc));
        }
        _ => {
            if cl {
                painter.add(Shape::closed_line(pts, s));
            } else {
                painter.add(Shape::line(pts, s));
            }
        }
    };
}

fn draw_arrow_head(painter: &egui::Painter, end: Pos2, prev: Pos2, sc: f32, s: Stroke) {
    let v = end - prev;
    let a = v.angle();
    let l = 20.0 * sc;
    let sp = 0.52; // 30deg
    painter.add(Shape::line(
        vec![
            end + Vec2::new(l * (a + 3.14 - sp).cos(), l * (a + 3.14 - sp).sin()),
            end,
            end + Vec2::new(l * (a + 3.14 + sp).cos(), l * (a + 3.14 + sp).sin()),
        ],
        s,
    ));
}

fn draw_element<F>(
    painter: &egui::Painter,
    el: &ExcalidrawElement,
    tex: Option<TextureHandle>,
    to_screen: &F,
    sc: f32,
) where
    F: Fn(Pos2) -> Pos2,
{
    if el.opacity == 0.0 {
        return;
    }
    let a = ((el.opacity / 100.0) * 255.0) as u8;
    let sc_col = hex_to_color(&el.stroke_color).linear_multiply(a as f32 / 255.0); // Simple alpha fix
    let bg_col = hex_to_color(&el.background_color).linear_multiply(a as f32 / 255.0);
    let s = Stroke::new(el.stroke_width * sc, sc_col);
    let cw = Pos2::new(el.x + el.width / 2.0, el.y + el.height / 2.0);
    let cl = Pos2::new(el.width / 2.0, el.height / 2.0);
    let rot = Rot2::from_angle(el.angle);
    let tr =
        |ps: &[Pos2]| -> Vec<Pos2> { ps.iter().map(|&p| to_screen(cw + rot * (p - cl))).collect() };

    match el.element_type.as_str() {
        "rectangle" | "diamond" | "ellipse" => {
            let pts = if el.element_type == "rectangle" {
                let r = el
                    .roundness
                    .as_ref()
                    .map(|x| if x.round_type == 3 { 20.0 } else { 4.0 })
                    .unwrap_or(0.0);
                discretize_rect(
                    Rect::from_min_size(Pos2::ZERO, Vec2::new(el.width, el.height)),
                    r,
                )
            } else if el.element_type == "diamond" {
                vec![
                    Pos2::new(el.width / 2.0, 0.0),
                    Pos2::new(el.width, el.height / 2.0),
                    Pos2::new(el.width / 2.0, el.height),
                    Pos2::new(0.0, el.height / 2.0),
                ]
            } else {
                discretize_ellipse(el.width, el.height)
            };
            let sp = tr(&pts);
            if el.background_color != "transparent" {
                painter.add(Shape::convex_polygon(sp.clone(), bg_col, Stroke::NONE));
            }
            draw_stroke(painter, sp, s, &el.stroke_style, sc, true);
        }
        "image" => {
            if let Some(t) = tex {
                let r = Rect::from_min_size(Pos2::ZERO, Vec2::new(el.width, el.height));
                let sp = tr(&[r.min, r.right_top(), r.max, r.left_bottom()]);
                let mut m = egui::Mesh::with_texture(t.id());
                let tint = Color32::from_white_alpha(a);
                m.add_triangle(0, 1, 2);
                m.add_triangle(0, 2, 3);
                let uv = [
                    Pos2::new(0.0, 0.0),
                    Pos2::new(1.0, 0.0),
                    Pos2::new(1.0, 1.0),
                    Pos2::new(0.0, 1.0),
                ];
                for (i, p) in sp.iter().enumerate() {
                    m.vertices.push(egui::epaint::Vertex {
                        pos: *p,
                        uv: uv[i],
                        color: tint,
                    });
                }
                painter.add(Shape::mesh(m));
            }
        }
        "line" | "arrow" | "draw" | "freedraw" => {
            if !el.points.is_empty() {
                let raw: Vec<Pos2> = el.points.iter().map(|p| Pos2::new(p[0], p[1])).collect();
                let sp = tr(&raw);
                draw_stroke(painter, sp.clone(), s, &el.stroke_style, sc, false);
                if let Some(at) = &el.end_arrowhead {
                    if at == "arrow" && sp.len() >= 2 {
                        draw_arrow_head(painter, sp[sp.len() - 1], sp[sp.len() - 2], sc, s);
                    }
                }
            }
        }
        "text" => {
            painter.text(
                tr(&[Pos2::ZERO])[0],
                egui::Align2::LEFT_TOP,
                &el.text,
                FontId::new(el.font_size * sc, FontFamily::Proportional),
                sc_col,
            );
        }
        _ => {}
    }
}

fn discretize_rect(r: Rect, rad: f32) -> Vec<Pos2> {
    if rad <= 1.0 {
        return vec![r.min, r.right_top(), r.max, r.left_bottom()];
    }
    let mut p = Vec::new();
    let rad = rad.min(r.width() / 2.0).min(r.height() / 2.0);
    add_arc(
        &mut p,
        Pos2::new(r.max.x - rad, r.min.y + rad),
        rad,
        -1.57,
        0.0,
    );
    add_arc(
        &mut p,
        Pos2::new(r.max.x - rad, r.max.y - rad),
        rad,
        0.0,
        1.57,
    );
    add_arc(
        &mut p,
        Pos2::new(r.min.x + rad, r.max.y - rad),
        rad,
        1.57,
        3.14,
    );
    add_arc(
        &mut p,
        Pos2::new(r.min.x + rad, r.min.y + rad),
        rad,
        3.14,
        4.71,
    );
    p
}

fn discretize_ellipse(w: f32, h: f32) -> Vec<Pos2> {
    let rx = w / 2.0;
    let ry = h / 2.0;
    let c = Pos2::new(rx, ry);
    (0..64)
        .map(|i| {
            let t = (i as f32 / 64.0) * 6.28;
            c + Vec2::new(rx * t.cos(), ry * t.sin())
        })
        .collect()
}

fn add_arc(p: &mut Vec<Pos2>, c: Pos2, r: f32, s: f32, e: f32) {
    for i in 0..=8 {
        let t = i as f32 / 8.0;
        let a = s + (e - s) * t;
        p.push(c + Vec2::new(r * a.cos(), r * a.sin()));
    }
}
