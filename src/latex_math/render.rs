//! Adaptador entre el display list del layout y egui.
//!
//! Basado en el exportador de Tinta (`src/math_render.cpp`, MIT,
//! Copyright (c) 2024 oipoistar): los `MathRun` se dibujan con su texto, tamaño
//! y línea base; las reglas y líneas se pintan como rectángulos/segmentos.

use super::layout::{MathBox, Measurer, Metrics};
use egui::text::{LayoutJob, TextFormat};
use egui::{Color32, FontFamily, FontId, Pos2, Rect, Stroke, Ui, Vec2};

/// Medición de texto usando las fuentes que egui ya tiene cargadas.
pub struct EguiMeasurer<'a> {
    pub ui: &'a Ui,
    pub family: FontFamily,
}

impl<'a> EguiMeasurer<'a> {
    pub fn new(ui: &'a Ui, family: FontFamily) -> Self {
        Self { ui, family }
    }

    fn job(&self, text: &str, size: f32, italic: bool) -> LayoutJob {
        let font_id = FontId::new(size, self.family.clone());
        let mut job = LayoutJob::default();
        job.append(
            text,
            0.0,
            TextFormat {
                font_id,
                color: Color32::WHITE,
                italics: italic,
                ..Default::default()
            },
        );
        job
    }
}

impl Measurer for EguiMeasurer<'_> {
    fn measure(&mut self, text: &str, size: f32, italic: bool, _bold: bool) -> Metrics {
        let painter = self.ui.painter();
        let galley = painter.layout_job(self.job(text, size, italic));
        let width = galley.size().x;
        let (ascent, height) = galley
            .rows
            .first()
            .and_then(|row| row.row.glyphs.first())
            .map(|g| (g.font_ascent, g.font_height))
            .unwrap_or_else(|| {
                let font_id = FontId::new(size, self.family.clone());
                let h = painter.fonts_mut(|fonts| fonts.row_height(&font_id));
                (h, h)
            });
        Metrics {
            width,
            ascent,
            descent: (height - ascent).max(0.0),
        }
    }
}

/// Dibuja el display list con el origen en `origin`.
pub fn draw(ui: &mut Ui, math_box: &MathBox, origin: Pos2, color: Color32) {
    let painter = ui.painter();

    // Reglas: barras de fracción, overlines, líneas radicales.
    for rule in &math_box.rules {
        let rect = Rect::from_min_size(
            origin + Vec2::new(rule.x, rule.y),
            Vec2::new(rule.w, rule.h),
        );
        painter.rect_filled(rect, 0.0, color);
    }

    // Líneas: delimitadores grandes, flechas, raíces, reglas de tablas.
    for line in &math_box.lines {
        painter.line_segment(
            [
                origin + Vec2::new(line.x1, line.y1),
                origin + Vec2::new(line.x2, line.y2),
            ],
            Stroke::new(line.thickness.max(1.0), color),
        );
    }

    // Runs de texto. `run.y` es el borde superior de la caja del texto; el
    // `galley` de egui también se posiciona por su esquina superior izquierda,
    // así que la línea base queda alineada igual que en el original.
    for run in &math_box.runs {
        let font_id = FontId::new(run.size, FontFamily::Name(crate::math_font::EGUI_FAMILY.into()));
        let mut job = LayoutJob::default();
        job.append(
            &run.text,
            0.0,
            TextFormat {
                font_id,
                color,
                italics: run.italic,
                ..Default::default()
            },
        );
        let galley = painter.layout_job(job);
        painter.galley(origin + Vec2::new(run.x, run.y), galley, color);
    }
}
