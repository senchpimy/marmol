//! Render de LaTeX usando el motor nativo portado de Tinta (sin Typst).
//!
//! Si no hay ninguna fuente matemática instalada, o la fórmula usa algo que el
//! parser no soporta, se muestra el código fuente como texto.

use crate::egui_commonmark_backend::misc::CommonMarkCache;
use crate::latex_math::{self, MathBox};
use egui::{Color32, RichText, Sense, TextStyle, Ui};
use std::sync::Arc;

/// Indica si hay una fuente matemática disponible para renderizar LaTeX.
pub fn has_math_font() -> bool {
    crate::math_font::system_math_font_data().is_some()
}

/// Muestra el LaTeX sin renderizar cuando no hay fuente matemática o el parser
/// no soporta la fórmula.
fn render_as_text(ui: &mut Ui, latex: &str, inline: bool) {
    let text = if inline {
        format!("${latex}$")
    } else {
        format!("$$ {latex} $$")
    };
    let label = RichText::new(text)
        .monospace()
        .color(ui.visuals().weak_text_color());
    if inline {
        ui.label(label);
    } else {
        ui.vertical_centered(|ui| {
            ui.label(label);
        });
    }
}

fn math_family() -> egui::FontFamily {
    egui::FontFamily::Name(crate::math_font::EGUI_FAMILY.into())
}

fn build_math_box(ui: &Ui, latex: &str, inline: bool, font_size: f32) -> Option<MathBox> {
    let node = latex_math::parse_latex(latex)?;
    let mut measurer = latex_math::EguiMeasurer::new(ui, math_family());
    Some(latex_math::layout(&node, font_size, !inline, &mut measurer))
}

fn draw_inline(ui: &mut Ui, math_box: &MathBox, color: Color32) {
    let size = egui::vec2(math_box.width.max(1.0), math_box.height.max(1.0));
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());

    // Alinear la línea base de la fórmula con la del texto que la rodea.
    let line_height = ui.text_style_height(&TextStyle::Body);
    let text_descent = (line_height * 0.2).max(0.0);
    let origin = egui::pos2(
        rect.left(),
        rect.bottom() - text_descent - math_box.baseline,
    );
    latex_math::render::draw(ui, math_box, origin, color);
}

fn draw_display(ui: &mut Ui, math_box: &MathBox, color: Color32) {
    let size = egui::vec2(math_box.width.max(1.0), math_box.height.max(1.0));
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    latex_math::render::draw(ui, math_box, rect.min, color);
}

pub fn render(ui: &mut Ui, cache: &mut CommonMarkCache, latex: &str, inline: bool, font_size: f32) {
    if !has_math_font() {
        render_as_text(ui, latex, inline);
        return;
    }

    let color = ui.visuals().text_color();
    let cache_key = format!("{latex}_{inline}_{font_size}_{color:?}");

    if let Some(math_box) = cache.latex_cache.get(&cache_key) {
        if inline {
            draw_inline(ui, math_box, color);
        } else {
            draw_display(ui, math_box, color);
        }
        return;
    }
    if cache.latex_error_cache.contains_key(&cache_key) {
        render_as_text(ui, latex, inline);
        return;
    }

    match build_math_box(ui, latex, inline, font_size) {
        Some(math_box) => {
            let math_box = Arc::new(math_box);
            super::misc::insert_cached(&mut cache.latex_cache, cache_key, math_box.clone());
            if inline {
                draw_inline(ui, &math_box, color);
            } else {
                draw_display(ui, &math_box, color);
            }
        }
        None => {
            super::misc::insert_cached(
                &mut cache.latex_error_cache,
                cache_key,
                "unsupported".to_string(),
            );
            render_as_text(ui, latex, inline);
        }
    }
}

/// Medidor aproximado para herramientas/tests que no tienen una `Ui`.
pub struct SimpleMeasurer;

impl latex_math::Measurer for SimpleMeasurer {
    fn measure(&mut self, text: &str, size: f32, _italic: bool, _bold: bool) -> latex_math::Metrics {
        let width = text.chars().count() as f32 * size * 0.5;
        latex_math::Metrics {
            width,
            ascent: size * 0.8,
            descent: size * 0.2,
        }
    }
}

/// Layout sin egui (para `examples/verificar_latex.rs` y tests).
pub fn layout_only(latex: &str, inline: bool, font_size: f32) -> Option<MathBox> {
    let node = latex_math::parse_latex(latex)?;
    let mut measurer = SimpleMeasurer;
    Some(latex_math::layout(&node, font_size, !inline, &mut measurer))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn context_with_math_font() -> Option<egui::Context> {
        if !has_math_font() {
            return None;
        }
        let ctx = egui::Context::default();
        let mut fonts = egui::FontDefinitions::default();
        crate::math_font::install_in_font_definitions(&mut fonts);
        ctx.set_fonts(fonts);
        Some(ctx)
    }

    #[test]
    fn renders_inline_and_display_math_in_egui() {
        let Some(ctx) = context_with_math_font() else {
            return;
        };
        let mut cache = CommonMarkCache::default();
        for (tex, inline) in [
            (r"\frac{a}{b}", true),
            (r"\sqrt{x^2 + y^2}", true),
            (r"\begin{pmatrix} a & b \\ c & d \end{pmatrix}", false),
            (r"\sum_{i=0}^{n} i^2", false),
        ] {
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                render(ui, &mut cache, tex, inline, 16.0);
            });
            output.textures_delta.clear();
        }
        assert_eq!(cache.latex_cache.len(), 4);
    }

    #[test]
    fn unsupported_latex_falls_back_to_source_text() {
        let Some(ctx) = context_with_math_font() else {
            return;
        };
        let mut cache = CommonMarkCache::default();
        let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
            render(ui, &mut cache, r"\comandoquenoexiste", true, 16.0);
        });
        output.textures_delta.clear();
        assert!(cache.latex_cache.is_empty());
        assert_eq!(cache.latex_error_cache.len(), 1);
    }

    #[test]
    fn reported_formulas_render_in_egui() {
        let Some(ctx) = context_with_math_font() else {
            return;
        };
        let formulas = [
            r"v - (e - 1) + (r - 1) = 2 \implies v - e + 1 + r - 1 = 2 \implies v - e + r = 2 \quad \blacksquare",
            r"v - e + r = v - (v - 1) + 1 = v - v + 1 + 1 = 2 \quad \checkmark",
            r"\mathbb{P}(E_1 \cup \dots \cup E_n) = \sum_{i=1}^{n} \mathbb{P}(E_i) - \sum_{i_1 < i_2} \mathbb{P}(E_{i_1} \cap E_{i_2}) \\
\\ + \dots + (-1)^{1+r} \sum_{i_1 < i_2 < \dots < i_r} \mathbb{P}(E_{i_1} \cap E_{i_2} \cap \dots \cap E_{i_r}) \\
\\ + \dots + (-1)^{n+1} \mathbb{P}(E_1 \cap \dots \cap E_n)",
            r"\begin{aligned}
\sum_{i} g(a_i) \, p(a_i) &= \sum_{j} \sum_{\{i \mid g(a_i) = y_j\}} g(a_i) \, p(a_i) \\[6pt]
&= \sum_{j} y_j \left( \sum_{\{i \mid g(a_i) = y_j\}} p(a_i) \right) \\[6pt]
&= \sum_{j} y_j \cdot \mathbb{P}\big(g(X) = y_j\big) \\[6pt]
&= \sum_{j} y_j \cdot \mathbb{P}(Y = y_j) \\[6pt]
&= \mathbb{E}[Y] = \mathbb{E}[g(X)] \quad \blacksquare
\end{aligned}",
        ];
        let mut cache = CommonMarkCache::default();
        for tex in formulas {
            let mut output = ctx.run_ui(egui::RawInput::default(), |ui| {
                render(ui, &mut cache, tex, false, 16.0);
            });
            output.textures_delta.clear();
        }
        assert_eq!(cache.latex_cache.len(), formulas.len());
        assert!(cache.latex_error_cache.is_empty());
    }
}
