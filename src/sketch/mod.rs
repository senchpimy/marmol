mod config;
mod llm;
mod preview;

use egui::{Color32, PointerButton, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};
use std::sync::{mpsc, Mutex};

#[derive(Clone)]
struct Trazo {
    puntos: Vec<Pos2>,
    color: Color32,
    ancho: f32,
}

enum Estado {
    Idle,
    Enviando,
    Listo,
    Error { msg: String },
}

pub struct SketchGui {
    trazos: Vec<Trazo>,
    actual: Option<Trazo>,
    historial: Vec<Vec<Trazo>>,
    color: Color32,
    grosor: f32,

    cfg: config::ConfigLlm,
    mostrar_config: bool,
    estado: Estado,
    receptor: Option<Mutex<mpsc::Receiver<Result<String, String>>>>,
    ultimo_png: Option<Vec<u8>>,
    latex_text: String,
    preview_cache: Option<(String, Result<String, String>)>,
    confirmado: bool,

    pub target_path: String,
    pub target_cursor: usize,
}

const PALETA: [Color32; 6] = [
    Color32::BLACK,
    Color32::from_rgb(224, 49, 49),
    Color32::from_rgb(34, 139, 230),
    Color32::from_rgb(47, 158, 68),
    Color32::from_rgb(255, 169, 77),
    Color32::from_rgb(156, 54, 181),
];

impl Default for SketchGui {
    fn default() -> Self {
        let saved = config::cargar();
        Self {
            trazos: vec![],
            actual: None,
            historial: vec![],
            color: PALETA[0],
            grosor: 3.0,

            cfg: saved.llm,
            mostrar_config: false,
            estado: Estado::Idle,
            receptor: None,
            ultimo_png: None,
            latex_text: String::new(),
            preview_cache: None,
            confirmado: false,

            target_path: String::new(),
            target_cursor: 0,
        }
    }
}

impl SketchGui {
    fn empujar_historial(&mut self) {
        self.historial.push(self.trazos.clone());
        if self.historial.len() > 50 {
            self.historial.remove(0);
        }
    }

    fn deshacer(&mut self) {
        if let Some(prev) = self.historial.pop() {
            self.trazos = prev;
        }
    }

    fn limpiar(&mut self) {
        if !self.trazos.is_empty() {
            self.empujar_historial();
            self.trazos.clear();
        }
    }

    fn confirmar_trazo(&mut self) {
        if let Some(t) = self.actual.take() {
            if !t.puntos.is_empty() {
                self.empujar_historial();
                self.trazos.push(t);
            }
        }
    }

    fn renderizar_png_bytes(&self) -> Option<Vec<u8>> {
        let mut todos: Vec<&Trazo> = self.trazos.iter().collect();
        if let Some(a) = &self.actual {
            todos.push(a);
        }

        let mut hay = false;
        let mut min = Pos2::new(f32::MAX, f32::MAX);
        let mut max = Pos2::new(f32::MIN, f32::MIN);
        for t in &todos {
            for p in &t.puntos {
                hay = true;
                min.x = min.x.min(p.x - t.ancho);
                min.y = min.y.min(p.y - t.ancho);
                max.x = max.x.max(p.x + t.ancho);
                max.y = max.y.max(p.y + t.ancho);
            }
        }

        if !hay {
            return None;
        }

        let pad = 16.0;
        let w = ((max.x - min.x) + pad * 2.0).ceil().max(1.0) as u32;
        let h = ((max.y - min.y) + pad * 2.0).ceil().max(1.0) as u32;

        let mut img = image::RgbaImage::from_pixel(w, h, image::Rgba([255, 255, 255, 255]));

        for t in todos {
            rasterizar_trazo(&mut img, t, min, pad);
        }

        let mut buf = std::io::Cursor::new(Vec::new());
        img.write_to(&mut buf, image::ImageFormat::Png).ok()?;
        Some(buf.into_inner())
    }

    fn enviar(&mut self) {
        if matches!(self.estado, Estado::Enviando) {
            return;
        }
        if self.trazos.is_empty() && self.actual.is_none() {
            return;
        }
        match self.renderizar_png_bytes() {
            Some(png) => self.enviar_bytes(png),
            None => {
                self.estado = Estado::Error {
                    msg: "Nada que enviar".to_string(),
                }
            }
        }
    }

    fn enviar_bytes(&mut self, png: Vec<u8>) {
        self.ultimo_png = Some(png.clone());
        self.estado = Estado::Enviando;
        self.confirmado = false;
        let (tx, rx) = mpsc::channel();
        self.receptor = Some(Mutex::new(rx));
        let cfg = self.cfg.clone();
        std::thread::spawn(move || {
            let _ = tx.send(llm::reconocer(&cfg, &png));
        });
    }

    fn procesar_respuestas(&mut self, ctx: &egui::Context) {
        let (resultado, desconectado) = match &self.receptor {
            Some(rx_lock) => {
                let rx = rx_lock.lock().unwrap();
                match rx.try_recv() {
                    Ok(v) => (Some(v), false),
                    Err(mpsc::TryRecvError::Empty) => (None, false),
                    Err(mpsc::TryRecvError::Disconnected) => (None, true),
                }
            }
            None => (None, false),
        };

        let mut repaint = false;
        match (resultado, desconectado) {
            (Some(Ok(latex)), _) => {
                self.latex_text = latex;
                self.confirmado = false;
                self.estado = Estado::Listo;
                self.receptor = None;
            }
            (Some(Err(msg)), _) => {
                self.estado = Estado::Error { msg };
                self.receptor = None;
            }
            (None, true) => {
                if matches!(self.estado, Estado::Enviando) {
                    self.estado = Estado::Error {
                        msg: "El reconocimiento terminó inesperadamente".to_string(),
                    };
                }
                self.receptor = None;
            }
            (None, false) => {
                if self.receptor.is_some() {
                    repaint = true;
                }
            }
        }

        if repaint {
            ctx.request_repaint();
        }
    }

    fn confirmar(&mut self, ctx: &egui::Context) {
        let latex = self.latex_text.trim();
        if latex.is_empty() {
            return;
        }
        self.confirmado = true;
        ctx.data_mut(|d| {
            d.insert_temp(
                egui::Id::new("sketch_confirm_signal"),
                Some((
                    self.target_path.clone(),
                    self.target_cursor,
                    latex.to_string(),
                )),
            )
        });
    }

    fn guardar_config(&self) {
        config::guardar(&config::SketchConfig {
            llm: self.cfg.clone(),
        });
    }

    pub fn show(&mut self, ui: &mut Ui) {
        self.procesar_respuestas(ui.ctx());

        let mut enviar = false;

        ui.horizontal(|ui| {
            ui.label("Color:");
            for &c in &PALETA {
                if ui
                    .add(
                        egui::Button::new(" ")
                            .fill(c)
                            .min_size(Vec2::splat(20.0))
                            .selected(self.color == c),
                    )
                    .clicked()
                {
                    self.color = c;
                }
            }

            ui.separator();
            ui.label("Grosor:");
            ui.add_sized(
                [70.0, 18.0],
                egui::Slider::new(&mut self.grosor, 1.0..=24.0),
            );

            ui.separator();
            if ui
                .button("↩ Deshacer")
                .on_hover_text("Deshacer último trazo (Ctrl+Z)")
                .clicked()
            {
                self.deshacer();
            }
            if ui
                .button("🗑 Limpiar")
                .on_hover_text("Borrar todo el lienzo")
                .clicked()
            {
                self.limpiar();
            }
            if ui.button("⚙").on_hover_text("Configuración del LLM").clicked() {
                self.mostrar_config = !self.mostrar_config;
            }

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                let puede_enviar =
                    !self.trazos.is_empty() && !matches!(self.estado, Estado::Enviando);
                if ui
                    .add_enabled(
                        puede_enviar,
                        egui::Button::new("✨ Convertir a LaTeX")
                            .fill(ui.visuals().selection.bg_fill),
                    )
                    .on_hover_text("Enviar el dibujo al LLM (Ctrl+Enter)")
                    .clicked()
                {
                    enviar = true;
                }
            });
        });

        if self.mostrar_config {
            ui.separator();
            let mut cfg_cambio = false;
            egui::Grid::new("sketch_config")
                .num_columns(2)
                .spacing([8.0, 4.0])
                .show(ui, |ui| {
                    ui.label("URL:");
                    if ui
                        .add(egui::TextEdit::singleline(&mut self.cfg.url).desired_width(300.0))
                        .changed()
                    {
                        cfg_cambio = true;
                    }
                    ui.end_row();

                    ui.label("API key:");
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut self.cfg.api_key)
                                .password(true)
                                .desired_width(300.0),
                        )
                        .changed()
                    {
                        cfg_cambio = true;
                    }
                    ui.end_row();

                    ui.label("Modelo:");
                    if ui
                        .add(
                            egui::TextEdit::singleline(&mut self.cfg.modelo)
                                .desired_width(300.0),
                        )
                        .changed()
                    {
                        cfg_cambio = true;
                    }
                    ui.end_row();
                });
            if cfg_cambio {
                self.guardar_config();
            }
            ui.weak("Compatible con OpenAI: OpenAI, Groq, OpenRouter, Ollama, LM Studio…");
            ui.separator();
        }

        if enviar {
            self.enviar();
        }

        let mostrar_resultado =
            !matches!(self.estado, Estado::Idle) || !self.latex_text.is_empty();

        let result_w = 360.0;
        let cursor = ui.cursor().min;
        let avail = ui.available_size();
        let canvas_w = if mostrar_resultado {
            (avail.x - result_w).max(200.0)
        } else {
            avail.x
        };

        let canvas_rect = Rect::from_min_size(cursor, Vec2::new(canvas_w, avail.y));
        let result_rect = Rect::from_min_size(
            Pos2::new(cursor.x + canvas_w, cursor.y),
            Vec2::new((avail.x - canvas_w).max(0.0), avail.y),
        );

        let canvas_painter = ui.painter_at(canvas_rect);
        let canvas_response = ui.interact(canvas_rect, ui.id().with("sketch_canvas"), Sense::drag());

        canvas_painter.rect_filled(canvas_rect, 0.0, Color32::WHITE);

        for t in &self.trazos {
            dibujar_trazo(&canvas_painter, t, canvas_rect.min);
        }
        if let Some(t) = &self.actual {
            dibujar_trazo(&canvas_painter, t, canvas_rect.min);
        }

        if canvas_response.drag_started_by(PointerButton::Primary) {
            if let Some(mp) = canvas_response.interact_pointer_pos() {
                self.actual = Some(Trazo {
                    puntos: vec![(mp - canvas_rect.min).to_pos2()],
                    color: self.color,
                    ancho: self.grosor,
                });
            }
        }
        if canvas_response.dragged_by(PointerButton::Primary) {
            if let Some(mp) = canvas_response.interact_pointer_pos() {
                let p = (mp - canvas_rect.min).to_pos2();
                if let Some(t) = &mut self.actual {
                    if t.puntos.last() != Some(&p) {
                        t.puntos.push(p);
                    }
                }
            }
        }
        if canvas_response.drag_stopped() {
            self.confirmar_trazo();
        }

        if mostrar_resultado {
            let builder = egui::UiBuilder::new()
                .id_salt("sketch_result_panel")
                .max_rect(result_rect);
            ui.scope_builder(builder, |ui| {
                ui.horizontal(|ui| {
                    ui.heading("Resultado");
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .button("✕")
                            .on_hover_text("Cerrar")
                            .clicked()
                        {
                            self.estado = Estado::Idle;
                            self.latex_text.clear();
                            self.confirmado = false;
                        }
                    });
                });
                ui.separator();

                if matches!(self.estado, Estado::Enviando) {
                    ui.vertical_centered(|ui| {
                        ui.add_space(24.0);
                        ui.add(egui::Spinner::new().size(32.0));
                        ui.add_space(10.0);
                        ui.label("Reconociendo…");
                    });
                }

                if matches!(self.estado, Estado::Listo) {
                    egui::ScrollArea::vertical().show(ui, |ui| {
                        preview::mostrar_preview(ui, &mut self.preview_cache, &self.latex_text);
                        ui.add_space(6.0);
                        ui.separator();
                        ui.label("LaTeX:");
                        ui.add(
                            egui::TextEdit::multiline(&mut self.latex_text)
                                .font(egui::TextStyle::Monospace)
                                .desired_rows(4)
                                .desired_width(f32::INFINITY),
                        );

                        if ui
                            .button("📋 Copiar")
                            .on_hover_text("Copiar LaTeX al portapapeles")
                            .clicked()
                        {
                            ui.ctx().copy_text(self.latex_text.clone());
                        }

                        ui.add_space(4.0);
                        if self.confirmado {
                            ui.colored_label(
                                Color32::from_rgb(47, 158, 68),
                                "✓ Insertado en la nota en la posición del cursor",
                            );
                        } else if ui
                            .button("✅ Confirmar")
                            .on_hover_text("Insertar $$…$$ en la nota en la posición del cursor")
                            .clicked()
                        {
                            self.confirmar(ui.ctx());
                        }
                    });
                }

                if let Estado::Error { msg } = &self.estado {
                    let msg = msg.clone();
                    ui.colored_label(Color32::RED, &msg);
                    ui.add_space(4.0);
                    ui.horizontal(|ui| {
                        if ui.button("↻ Reintentar").clicked() {
                            if let Some(png) = self.ultimo_png.clone() {
                                self.enviar_bytes(png);
                            }
                        }
                        if ui.button("✕ Cerrar").clicked() {
                            self.estado = Estado::Idle;
                        }
                    });
                }
            });
        }
    }
}

fn rasterizar_trazo(img: &mut image::RgbaImage, t: &Trazo, min: Pos2, pad: f32) {
    let radio = t.ancho / 2.0;
    let mapa = |p: &Pos2| -> (f32, f32) { (p.x - min.x + pad, p.y - min.y + pad) };

    if t.puntos.is_empty() {
        return;
    }
    if t.puntos.len() == 1 {
        let c = mapa(&t.puntos[0]);
        sello_circulo(img, c.0, c.1, radio, t.color);
        return;
    }

    for v in t.puntos.windows(2) {
        let a = mapa(&v[0]);
        let b = mapa(&v[1]);
        let d = ((b.0 - a.0).powi(2) + (b.1 - a.1).powi(2)).sqrt();
        let pasos = (d.ceil() as usize).max(1);
        for s in 0..=pasos {
            let f = s as f32 / pasos as f32;
            sello_circulo(img, a.0 + (b.0 - a.0) * f, a.1 + (b.1 - a.1) * f, radio, t.color);
        }
    }
}

fn sello_circulo(img: &mut image::RgbaImage, cx: f32, cy: f32, r: f32, col: Color32) {
    let (w, h) = img.dimensions();
    let arr = col.to_array();
    let x0 = ((cx - r - 1.0).floor().max(0.0)) as u32;
    let y0 = ((cy - r - 1.0).floor().max(0.0)) as u32;
    let x1 = ((cx + r + 1.0).ceil().min(w as f32 - 1.0).max(0.0)) as u32;
    let y1 = ((cy + r + 1.0).ceil().min(h as f32 - 1.0).max(0.0)) as u32;
    if x1 < x0 || y1 < y0 {
        return;
    }
    for y in y0..=y1 {
        for x in x0..=x1 {
            let dx = x as f32 + 0.5 - cx;
            let dy = y as f32 + 0.5 - cy;
            let cobertura = (r + 0.5 - (dx * dx + dy * dy).sqrt()).clamp(0.0, 1.0);
            if cobertura <= 0.0 {
                continue;
            }
            let px = img.get_pixel_mut(x, y);
            let sa = px.0[3] as f32 / 255.0;
            let na = arr[3] as f32 / 255.0 * cobertura;
            let out_a = na + sa * (1.0 - na);
            if out_a <= 0.0 {
                continue;
            }
            for (ch, src) in px.0[..3].iter_mut().zip(arr[..3].iter()) {
                let dst = *ch as f32 / 255.0 * sa;
                let src = *src as f32 / 255.0 * na;
                let v = (src + dst * (1.0 - na)) / out_a;
                *ch = (v * 255.0).round().clamp(0.0, 255.0) as u8;
            }
            px.0[3] = (out_a * 255.0).round().clamp(0.0, 255.0) as u8;
        }
    }
}

fn dibujar_trazo(painter: &egui::Painter, t: &Trazo, origen: Pos2) {
    let pts: Vec<Pos2> = t.puntos.iter().map(|p| origen + p.to_vec2()).collect();
    if pts.len() >= 2 {
        painter.add(Shape::line(pts, Stroke::new(t.ancho, t.color)));
    } else if let Some(p) = pts.first() {
        painter.circle_filled(*p, t.ancho / 2.0, t.color);
    }
}
