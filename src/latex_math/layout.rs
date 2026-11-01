//! Layout del motor matemático de Tinta, portado a Rust.
//!
//! Basado en `src/math_render.cpp` de Tinta (MIT, Copyright (c) 2024
//! oipoistar). El original mide con DirectWrite y pinta con Direct2D. Aquí la
//! medición se abstrae con [`Measurer`] y el resultado es un "display list"
//! ([`MathBox`]) que `render.rs` dibuja con egui.
//!
//! El algoritmo de layout es un port directo de `mathParse` /
//! `layoutNodeImpl`: dos pasadas (medición con `record = false` y colocación
//! con `record = true`), `pad = fontSize * (display ? 0.15 : 0.08)` y
//! `baseline = probe.ascent + pad`.
//!
//! Limitaciones conocidas:
//! - La medición es 2D; las itálicas dependen de la fuente y del `Measurer`.
//! - Los tamaños de fuente y los espacios son exactamente los del original,
//!   pero los anchos reales dependen de la fuente que use cada `Measurer`.

use super::ast::{Kind, MNode};

#[derive(Clone, Copy, Debug, Default)]
pub struct Metrics {
    pub width: f32,
    pub ascent: f32,
    pub descent: f32,
}

impl Metrics {
    pub fn height(&self) -> f32 {
        self.ascent + self.descent
    }
}

#[derive(Clone, Debug, Default)]
pub struct MathRun {
    pub text: String,
    pub x: f32,
    pub y: f32,
    pub size: f32,
    pub italic: bool,
    pub bold: bool,
    pub baseline: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MathRule {
    pub x: f32,
    pub y: f32,
    pub w: f32,
    pub h: f32,
}

#[derive(Clone, Copy, Debug, Default)]
pub struct MathLine {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
    pub thickness: f32,
}

/// Resultado del layout: mismo contenido que el `MathBox` de Tinta.
#[derive(Clone, Debug, Default)]
pub struct MathBox {
    pub runs: Vec<MathRun>,
    pub rules: Vec<MathRule>,
    pub lines: Vec<MathLine>,
    pub width: f32,
    pub height: f32,
    /// Distancia desde el borde superior hasta la línea base del texto.
    pub baseline: f32,
}

/// Abstracción de la medición de texto. En producción la implementa
/// `render::EguiMeasurer`; en tests se usa una implementación determinista.
pub trait Measurer {
    fn measure(&mut self, text: &str, size: f32, italic: bool, bold: bool) -> Metrics;
}

// ---------------------------------------------------------------------------
// Contexto y utilidades
// ---------------------------------------------------------------------------

struct LayoutCtx<'a> {
    measurer: &'a mut dyn Measurer,
    out: &'a mut MathBox,
}

fn style_scale(style: i32) -> f32 {
    if style < 2 {
        1.0
    } else if style == 2 {
        0.7
    } else {
        0.5
    }
}

fn is_math_variable_char(c: char) -> bool {
    let c = c as u32;
    (c >= 'a' as u32 && c <= 'z' as u32)
        || (c >= 'A' as u32 && c <= 'Z' as u32)
        || (c >= 0x03B1 && c <= 0x03D6)
        || c == 0x03F0
        || c == 0x03F1
        || c == 0x03F5
}

/// Port de `symbolSpacing` (clases de espaciado TeX aproximadas).
fn symbol_spacing(text: &str, roman: bool) -> f32 {
    let chars: Vec<char> = text.chars().collect();
    if chars.len() != 1 {
        if text == "\u{22EF}" || text == "\u{2026}" {
            return 0.16;
        }
        // Nombres de función upright (sin, log, ...) llevan espacio fino.
        if roman && !chars.is_empty() && chars.iter().all(|c| c.is_alphabetic()) {
            return 0.16;
        }
        return 0.0;
    }
    let c = chars[0] as u32;
    if (0x2190..=0x21FF).contains(&c)
        || (0x27F0..=0x27FF).contains(&c)
        || (0x223C..=0x223D).contains(&c)
        || (0x2243..=0x228D).contains(&c)
        || (0x22A2..=0x22AF).contains(&c)
        || c == 0x2A7D
        || c == 0x2A7E
        || c == 0x2AAF
        || c == 0x2AB0
    {
        return 0.26;
    }
    match c {
        0x3D | 0x3C | 0x3E // = < >
        | 0x2264 | 0x2265 | 0x2260 | 0x2261 | 0x2248
        | 0x2208 | 0x2209 | 0x2282 | 0x2283 | 0x2286
        | 0x2287 | 0x2192 | 0x21D2 | 0x2194 | 0x21D4
        | 0x223C | 0x221D | 0x2223 | 0x2225
        | 0x2190 | 0x21D0 | 0x21A6
        | 0x27F5 | 0x27F6 | 0x27F7 | 0x27F8
        | 0x27F9 | 0x27FA | 0x27FC => 0.26, // relaciones
        0x2B | 0x2212 | 0x00B1 | 0x2213 | 0x00D7
        | 0x00F7 | 0x22C5 | 0x222A | 0x2229 | 0x2227
        | 0x2228 | 0x2216 | 0x2295 | 0x2296 | 0x2297
        | 0x2298 | 0x2299 | 0x229B | 0x228E | 0x2293
        | 0x2294 | 0x22C9 | 0x22CA => 0.2, // operadores binarios
        0x2C | 0x3B => 0.12, // puntuación
        _ => 0.0,
    }
}

/// Port de `nodeSpacing`: espaciado explícito o inferido del primer hijo.
fn node_spacing(node: Option<&MNode>) -> f32 {
    let Some(node) = node else {
        return 0.0;
    };
    if node.spacing >= 0.0 {
        return node.spacing;
    }
    if node.literal_text {
        return 0.0;
    }
    if node.kind == Kind::Sym {
        return symbol_spacing(&node.text, node.roman);
    }
    if (node.kind == Kind::Script || node.kind == Kind::Stack) && !node.kids.is_empty() {
        return node_spacing(node.kid(0));
    }
    0.0
}

/// Port de `emitRun`: mide el texto y, si `record`, lo añade al display list.
///
/// La posición se da por la línea base, igual que el original. `run.y` es el
/// borde superior de la caja del texto; `run.baseline` es la línea base dentro
/// de esa caja.
fn emit_run(
    ctx: &mut LayoutCtx,
    text: &str,
    size: f32,
    italic: bool,
    bold: bool,
    x: f32,
    baseline_y: f32,
    record: bool,
) -> Metrics {
    let m = ctx.measurer.measure(text, size, italic, bold);
    if record {
        ctx.out.runs.push(MathRun {
            text: text.to_owned(),
            x,
            y: baseline_y - m.ascent,
            size,
            italic,
            bold,
            baseline: m.ascent,
        });
    }
    m
}

// ---------------------------------------------------------------------------
// Layout recursivo
// ---------------------------------------------------------------------------

fn layout_node(
    ctx: &mut LayoutCtx,
    node: Option<&MNode>,
    size: f32,
    x: f32,
    baseline_y: f32,
    style: i32,
    delimiter_height: f32,
    record: bool,
) -> Metrics {
    layout_node_impl(ctx, node, size, x, baseline_y, style, delimiter_height, record)
}

fn measure_node(
    ctx: &mut LayoutCtx,
    node: Option<&MNode>,
    size: f32,
    style: i32,
    delimiter_height: f32,
) -> Metrics {
    layout_node_impl(ctx, node, size, 0.0, 0.0, style, delimiter_height, false)
}

/// Port de `layoutNodeImpl`. `ctx.style` y `ctx.delimiterHeight` del original
/// se pasan aquí como parámetros para reproducir la semántica de paso por
/// valor del C++.
#[allow(clippy::too_many_arguments)]
fn layout_node_impl(
    ctx: &mut LayoutCtx,
    node: Option<&MNode>,
    mut size: f32,
    x: f32,
    baseline_y: f32,
    mut style: i32,
    delimiter_height: f32,
    record: bool,
) -> Metrics {
    let mut m = Metrics::default();
    let Some(node) = node else {
        return m;
    };

    if node.style >= 0 {
        size *= style_scale(node.style) / style_scale(style);
        style = node.style;
    }
    let em = size;

    match node.kind {
        Kind::Space => {
            m.width = node.space
                * if node.text == "pt" {
                    96.0 / 72.0
                } else if node.text == "px" {
                    1.0
                } else {
                    em
                };
            m.ascent = 0.0;
            m.descent = 0.0;
            m
        }
        Kind::Sym => {
            let italic = node.force_italic
                || (!node.roman
                    && !node.text.is_empty()
                    && node.text.chars().next().is_some_and(is_math_variable_char));
            let run_size = size * if node.large_op && style == 0 { 1.45 } else { 1.0 };
            emit_run(
                ctx,
                &node.text,
                run_size,
                italic,
                node.bold,
                x,
                baseline_y,
                record,
            )
        }
        Kind::Row => {
            let mut cx = x;
            let mut prev_space = 0.0f32;
            let mut first = true;
            for kid in node.kids.iter().flatten() {
                let kid_node = kid.as_ref();
                let mut kid_space = node_spacing(Some(kid_node)) * em;
                // Un signo inicial (o tras otro operador) es unario.
                if node_spacing(Some(kid_node)) == 0.2 && (first || prev_space >= em * 0.2) {
                    kid_space = 0.0;
                }
                if !first {
                    cx += prev_space.max(kid_space);
                }
                let km = layout_node_impl(
                    ctx,
                    Some(kid_node),
                    size,
                    cx,
                    baseline_y,
                    style,
                    delimiter_height,
                    record,
                );
                cx += km.width;
                m.ascent = m.ascent.max(km.ascent);
                m.descent = m.descent.max(km.descent);
                prev_space = kid_space;
                first = false;
            }
            m.width = cx - x;
            m
        }
        Kind::Middle => {
            if node.open == '\0' {
                return m;
            }
            let height = if node.delimiter_size > 0.0 {
                node.delimiter_size * em
            } else {
                em.max(delimiter_height)
            };
            let center = baseline_y - em * 0.26;
            if matches!(node.open, '|' | '\u{2016}' | '\u{2223}' | '\u{2225}') {
                m.width = em * 0.3;
                m.ascent = height / 2.0 + em * 0.26;
                m.descent = height / 2.0 - em * 0.26;
                if record {
                    let stroke = (em * 0.055).max(1.0);
                    ctx.out.lines.push(MathLine {
                        x1: x + em * 0.1,
                        y1: center - height / 2.0,
                        x2: x + em * 0.1,
                        y2: center + height / 2.0,
                        thickness: stroke,
                    });
                    if node.open != '|' && node.open != '\u{2223}' {
                        ctx.out.lines.push(MathLine {
                            x1: x + em * 0.25,
                            y1: center - height / 2.0,
                            x2: x + em * 0.25,
                            y2: center + height / 2.0,
                            thickness: stroke,
                        });
                    }
                }
                return m;
            }
            let text = node.open.to_string();
            let glyph = emit_run(ctx, &text, height, false, false, 0.0, 0.0, false);
            m.width = glyph.width;
            m.ascent = glyph.height() / 2.0 + em * 0.26;
            m.descent = (glyph.height() / 2.0 - em * 0.26).max(0.0);
            if record {
                emit_run(
                    ctx,
                    &text,
                    height,
                    false,
                    false,
                    x,
                    center + (glyph.ascent - glyph.descent) / 2.0,
                    true,
                );
            }
            m
        }
        Kind::Grid => {
            let cell_style = if node.compact {
                style.max(2)
            } else {
                style.max(1)
            };
            let cell_size = size * style_scale(cell_style) / style_scale(style);
            let mut columns = node.alignment.chars().count();
            for row in &node.cells {
                columns = columns.max(row.len());
            }
            if columns == 0 || node.cells.is_empty() {
                return m;
            }

            let mut widths = vec![0.0f32; columns];
            let mut ascents: Vec<f32> = Vec::new();
            let mut descents: Vec<f32> = Vec::new();
            let mut cells: Vec<Vec<Metrics>> = Vec::new();
            let mut total_height = 0.0f32;
            let row_gap = cell_size * 0.35;

            for row in &node.cells {
                let mut ascent = cell_size * 0.8;
                let mut descent = cell_size * 0.2;
                let mut measured = Vec::with_capacity(row.len());
                for (col, cell) in row.iter().enumerate() {
                    let cm = measure_node(
                        ctx,
                        Some(cell),
                        cell_size,
                        cell_style,
                        delimiter_height,
                    );
                    widths[col] = widths[col].max(cm.width);
                    ascent = ascent.max(cm.ascent);
                    descent = descent.max(cm.descent);
                    measured.push(cm);
                }
                ascents.push(ascent);
                descents.push(descent);
                total_height += ascent + descent;
                cells.push(measured);
            }
            total_height += row_gap * (cells.len() as f32 - 1.0);

            let cell_padding = if node.vertical_rules.is_empty() {
                0.0
            } else {
                cell_size * 0.2
            };
            for width in &mut widths {
                *width += cell_padding * 2.0;
            }

            m.ascent = total_height / 2.0 + em * 0.26;
            m.descent = total_height - m.ascent;
            let mut top = baseline_y - m.ascent;

            for row in 0..cells.len() {
                let mut cx = x;
                for col in 0..columns {
                    let mut align = if node.aligned {
                        if col % 2 == 1 {
                            'l'
                        } else {
                            'r'
                        }
                    } else {
                        'c'
                    };
                    if node.uniform_alignment {
                        align = node.alignment.chars().next().unwrap_or('c');
                    } else if col < node.alignment.chars().count() {
                        align = node.alignment.chars().nth(col).unwrap_or('c');
                    }
                    if col < cells[row].len() && record {
                        let cw = cells[row][col].width;
                        let extra = widths[col] - cw - 2.0 * cell_padding;
                        let offset = cell_padding
                            + if align == 'r' {
                                extra
                            } else if align == 'c' {
                                extra / 2.0
                            } else {
                                0.0
                            };
                        layout_node_impl(
                            ctx,
                            Some(&node.cells[row][col]),
                            cell_size,
                            cx + offset,
                            top + ascents[row],
                            cell_style,
                            delimiter_height,
                            true,
                        );
                    }
                    cx += widths[col];
                    if col + 1 < columns {
                        cx += cell_size
                            * if node.aligned && col % 2 == 0 {
                                0.26
                            } else {
                                1.0
                            };
                    }
                }
                m.width = m.width.max(cx - x);
                top += ascents[row] + descents[row] + row_gap;
            }

            if record {
                let stroke = (em * 0.045).max(1.0);
                let y1 = baseline_y - m.ascent;
                let y2 = baseline_y + m.descent;
                for &boundary in &node.vertical_rules {
                    let mut dx = x;
                    for col in 0..boundary.min(columns) {
                        dx += widths[col];
                        if col + 1 < columns {
                            dx += cell_size * if col + 1 == boundary { 0.5 } else { 1.0 };
                        }
                    }
                    ctx.out.lines.push(MathLine {
                        x1: dx,
                        y1,
                        x2: dx,
                        y2,
                        thickness: stroke,
                    });
                }
                for &boundary in &node.horizontal_rules {
                    let mut dy = y1;
                    for row in 0..boundary.min(cells.len()) {
                        dy += ascents[row] + descents[row];
                        if row + 1 < cells.len() {
                            dy += row_gap * if row + 1 == boundary { 0.5 } else { 1.0 };
                        }
                    }
                    ctx.out.lines.push(MathLine {
                        x1: x,
                        y1: dy,
                        x2: x + m.width,
                        y2: dy,
                        thickness: stroke,
                    });
                }
            }
            m
        }
        Kind::Phantom => {
            m = layout_node_impl(
                ctx,
                node.kid(0),
                size,
                x,
                baseline_y,
                style,
                delimiter_height,
                record && node.deco_kind == 3,
            );
            if node.deco_kind == 1 || node.deco_kind == 3 {
                m.ascent = 0.0;
                m.descent = 0.0;
            }
            if node.deco_kind == 2 {
                m.width = 0.0;
            }
            m
        }
        Kind::Stack => {
            let child_style = (style + 1).clamp(2, 3);
            let annotation_size = size * style_scale(child_style) / style_scale(style);
            let mut base = measure_node(ctx, node.kid(0), size, style, delimiter_height);
            let below = measure_node(
                ctx,
                node.kid(1),
                annotation_size,
                child_style,
                delimiter_height,
            );
            let above = measure_node(
                ctx,
                node.kid(2),
                annotation_size,
                child_style,
                delimiter_height,
            );
            if node.deco_kind != 0 {
                base.width = (em * 1.6).max(below.width.max(above.width) + em * 0.6);
                base.ascent = em * 0.45;
                base.descent = 0.0;
            }
            m.width = base.width.max(below.width.max(above.width));
            let gap = em * 0.12;
            m.ascent = base.ascent
                + if node.kid(2).is_some() {
                    gap + above.height()
                } else {
                    0.0
                };
            m.descent = base.descent
                + if node.kid(1).is_some() {
                    gap + below.height()
                } else {
                    0.0
                };
            if record {
                if node.deco_kind != 0 {
                    let ay = baseline_y - em * 0.26;
                    let head = em * 0.22;
                    let tip = if node.deco_kind == 1 {
                        x + m.width
                    } else {
                        x
                    };
                    let tail = tip + if node.deco_kind == 1 { -head } else { head };
                    let stroke = (em * 0.055).max(1.0);
                    ctx.out.lines.push(MathLine {
                        x1: x,
                        y1: ay,
                        x2: x + m.width,
                        y2: ay,
                        thickness: stroke,
                    });
                    ctx.out.lines.push(MathLine {
                        x1: tail,
                        y1: ay - head * 0.6,
                        x2: tip,
                        y2: ay,
                        thickness: stroke,
                    });
                    ctx.out.lines.push(MathLine {
                        x1: tail,
                        y1: ay + head * 0.6,
                        x2: tip,
                        y2: ay,
                        thickness: stroke,
                    });
                } else {
                    layout_node_impl(
                        ctx,
                        node.kid(0),
                        size,
                        x + (m.width - base.width) / 2.0,
                        baseline_y,
                        style,
                        delimiter_height,
                        true,
                    );
                }
                if node.kid(2).is_some() {
                    layout_node_impl(
                        ctx,
                        node.kid(2),
                        annotation_size,
                        x + (m.width - above.width) / 2.0,
                        baseline_y - base.ascent - gap - above.descent,
                        child_style,
                        delimiter_height,
                        true,
                    );
                }
                if node.kid(1).is_some() {
                    layout_node_impl(
                        ctx,
                        node.kid(1),
                        annotation_size,
                        x + (m.width - below.width) / 2.0,
                        baseline_y + base.descent + gap + below.ascent,
                        child_style,
                        delimiter_height,
                        true,
                    );
                }
            }
            m
        }
        Kind::Script => {
            let Some(base) = node.kid(0) else {
                return m;
            };
            let sub = node.kid(1);
            let sup = node.kid(2);
            let child_style = (style + 1).clamp(2, 3);
            let script_size = size * style_scale(child_style) / style_scale(style);

            let centered = base.limits == 1
                || (base.limits < 0
                    && base.limit_op
                    && (style == 0 || base.kind == Kind::Deco));
            if centered {
                let bm = measure_node(ctx, Some(base), size, style, delimiter_height);
                let sm = measure_node(ctx, sub, script_size, child_style, delimiter_height);
                let tm = measure_node(ctx, sup, script_size, child_style, delimiter_height);
                m.width = bm.width.max(sm.width.max(tm.width));
                let gap = em * 0.16;
                m.ascent = bm.ascent
                    + if sup.is_some() {
                        gap + tm.height()
                    } else {
                        0.0
                    };
                m.descent = bm.descent
                    + if sub.is_some() {
                        gap + sm.height()
                    } else {
                        0.0
                    };
                if record {
                    layout_node_impl(
                        ctx,
                        Some(base),
                        size,
                        x + (m.width - bm.width) / 2.0,
                        baseline_y,
                        style,
                        delimiter_height,
                        true,
                    );
                    if sup.is_some() {
                        layout_node_impl(
                            ctx,
                            sup,
                            script_size,
                            x + (m.width - tm.width) / 2.0,
                            baseline_y - bm.ascent - gap - tm.descent,
                            child_style,
                            delimiter_height,
                            true,
                        );
                    }
                    if sub.is_some() {
                        layout_node_impl(
                            ctx,
                            sub,
                            script_size,
                            x + (m.width - sm.width) / 2.0,
                            baseline_y + bm.descent + gap + sm.ascent,
                            child_style,
                            delimiter_height,
                            true,
                        );
                    }
                }
                return m;
            }

            let bm = layout_node_impl(
                ctx,
                Some(base),
                size,
                x,
                baseline_y,
                style,
                delimiter_height,
                record,
            );
            let sx = x + bm.width + em * 0.03;
            let sup_shift = (bm.ascent * 0.55).max(em * 0.38);
            let sub_shift = (bm.descent + em * 0.05).max(em * 0.16);

            let mut sup_m = Metrics::default();
            let mut sub_m = Metrics::default();
            if sup.is_some() {
                sup_m = layout_node_impl(
                    ctx,
                    sup,
                    script_size,
                    sx,
                    baseline_y - sup_shift,
                    child_style,
                    delimiter_height,
                    record,
                );
            }
            if sub.is_some() {
                sub_m = layout_node_impl(
                    ctx,
                    sub,
                    script_size,
                    sx,
                    baseline_y + sub_shift,
                    child_style,
                    delimiter_height,
                    record,
                );
            }
            m.width = bm.width + em * 0.03 + sup_m.width.max(sub_m.width);
            m.ascent = bm.ascent.max(if sup.is_some() {
                sup_shift + sup_m.ascent
            } else {
                0.0
            });
            m.descent = bm.descent.max(if sub.is_some() {
                sub_shift + sub_m.descent
            } else {
                0.0
            });
            m
        }
        Kind::Frac => {
            let child_style = (style + 1).min(3);
            let inner = size * style_scale(child_style) / style_scale(style);
            let num = measure_node(ctx, node.kid(0), inner, child_style, delimiter_height);
            let den = measure_node(ctx, node.kid(1), inner, child_style, delimiter_height);
            let pad = em * 0.12;
            let rule_w = num.width.max(den.width) + pad * 2.0;
            let rule_h = if node.no_bar {
                0.0
            } else {
                (em * 0.055).max(1.0)
            };
            let axis = em * 0.26;
            let gap = em * if style == 0 { 0.14 } else { 0.1 };

            let rule_y = baseline_y - axis - rule_h / 2.0;
            if record {
                layout_node_impl(
                    ctx,
                    node.kid(0),
                    inner,
                    x + pad + (rule_w - 2.0 * pad - num.width) / 2.0,
                    rule_y - gap - num.descent,
                    child_style,
                    delimiter_height,
                    true,
                );
                layout_node_impl(
                    ctx,
                    node.kid(1),
                    inner,
                    x + pad + (rule_w - 2.0 * pad - den.width) / 2.0,
                    rule_y + rule_h + gap + den.ascent,
                    child_style,
                    delimiter_height,
                    true,
                );
                if !node.no_bar {
                    ctx.out.rules.push(MathRule {
                        x,
                        y: rule_y,
                        w: rule_w,
                        h: rule_h,
                    });
                }
            }
            m.width = rule_w;
            m.ascent = axis + rule_h / 2.0 + gap + num.height();
            m.descent = (-axis + rule_h / 2.0 + gap + den.height()).max(0.0);
            m
        }
        Kind::Delim => {
            // El original pone ctx.delimiterHeight = 0 para medir el contenido.
            let cm = measure_node(ctx, node.kid(0), size, style, 0.0);
            let common = |c: char| c == '\0' || "()[]{}|\u{2016}".contains(c);
            if cm.height() > em * 1.4 && common(node.open) && common(node.close) {
                let top = baseline_y - cm.ascent - em * 0.08;
                let height = cm.height() + em * 0.16;
                let width = em * 0.38;
                let pad = em * 0.12;
                let stroke = (em * 0.055).max(1.0);

                let mut new_lines: Vec<MathLine> = Vec::new();
                let mut line = |x1: f32, y1: f32, x2: f32, y2: f32| {
                    if record {
                        new_lines.push(MathLine {
                            x1,
                            y1,
                            x2,
                            y2,
                            thickness: stroke,
                        });
                    }
                };
                let mut draw = |d: char, dx: f32| {
                    if d == '\0' {
                        return;
                    }
                    let bottom = top + height;
                    let right = matches!(d, ')' | ']' | '}');
                    if d == '[' || d == ']' {
                        let edge = dx + if right { width } else { 0.0 };
                        line(edge, top, edge, bottom);
                        line(dx, top, dx + width, top);
                        line(dx, bottom, dx + width, bottom);
                    } else if d == '|' || d == '\u{2016}' {
                        line(dx + width * 0.4, top, dx + width * 0.4, bottom);
                        if d == '\u{2016}' {
                            line(dx + width * 0.8, top, dx + width * 0.8, bottom);
                        }
                    } else {
                        let point = |t: f32| {
                            let u = if d == '{' || d == '}' {
                                // Hombro, cintura y hombro de una llave.
                                let v = (t - 0.5).abs() * 2.0;
                                0.5
                                    + 0.5
                                        * (2.0 * v - 1.0)
                                        * (2.0 * v - 1.0)
                                        * (2.0 * v - 1.0)
                            } else {
                                1.0 - (t * std::f32::consts::PI).sin()
                            };
                            (
                                dx + width * if right { 1.0 - u } else { u },
                                top + t * height,
                            )
                        };
                        let mut prev = point(0.0);
                        for step in 1..=32 {
                            let next = point(step as f32 / 32.0);
                            line(prev.0, prev.1, next.0, next.1);
                            prev = next;
                        }
                    }
                };

                let left_width = if node.open != '\0' { width + pad } else { 0.0 };
                let right_width = if node.close != '\0' { width + pad } else { 0.0 };
                draw(node.open, x);
                layout_node_impl(
                    ctx,
                    node.kid(0),
                    size,
                    x + left_width,
                    baseline_y,
                    style,
                    cm.height(),
                    record,
                );
                draw(node.close, x + left_width + cm.width + pad);
                if record {
                    ctx.out.lines.extend(new_lines);
                }
                m.width = left_width + cm.width + right_width;
                m.ascent = cm.ascent + em * 0.08 + stroke / 2.0;
                m.descent = cm.descent + em * 0.08 + stroke / 2.0;
                return m;
            }

            // Los delimitadores se estiran con el contenido escalando el
            // tamaño del glifo.
            let content_h = cm.height().max(em);
            let glyph_size = if content_h > em * 1.15 {
                content_h
            } else {
                size
            };
            let mut cx = x;
            let center_y = baseline_y - (cm.ascent - cm.descent) / 2.0;
            let emit_delim = |ctx: &mut LayoutCtx, d: char, cx: f32| -> f32 {
                if d == '\0' {
                    return 0.0;
                }
                let dt = d.to_string();
                let dm = emit_run(ctx, &dt, glyph_size, false, false, 0.0, 0.0, false);
                if record {
                    // Centra el glifo en el centro vertical del contenido.
                    emit_run(
                        ctx,
                        &dt,
                        glyph_size,
                        false,
                        false,
                        cx,
                        center_y + (dm.ascent - dm.descent) / 2.0,
                        true,
                    );
                }
                dm.width
            };
            let ow = emit_delim(ctx, node.open, cx);
            cx += ow;
            let inner = layout_node_impl(
                ctx,
                node.kid(0),
                size,
                cx,
                baseline_y,
                style,
                cm.height(),
                record,
            );
            cx += inner.width;
            cx += emit_delim(ctx, node.close, cx);
            m.width = cx - x;
            let glyph_half = (glyph_size * 1.1) / 2.0;
            let center_off = (cm.ascent - cm.descent) / 2.0;
            m.ascent = cm.ascent.max(center_off + glyph_half);
            m.descent = cm.descent.max(glyph_half - center_off);
            m
        }
        Kind::Deco => {
            if node.deco_kind == 3 {
                // sqrt
                let cm = measure_node(ctx, node.kid(0), size, style, delimiter_height);
                let child_style = 3;
                let index_size = size * style_scale(3) / style_scale(style);
                let index = node.kid(1);
                let im = measure_node(ctx, index, index_size, child_style, delimiter_height);
                let index_width = if index.is_some() {
                    (im.width - em * 0.15).max(0.0)
                } else {
                    0.0
                };
                let start = x + index_width;
                let cx = start + em * 0.65;
                let top = baseline_y - cm.ascent - em * 0.14;
                let bottom = baseline_y + cm.descent;
                let stroke = (em * 0.055).max(1.0);
                let index_baseline = top + em * 0.25;
                if record {
                    layout_node_impl(
                        ctx,
                        node.kid(0),
                        size,
                        cx,
                        baseline_y,
                        style,
                        delimiter_height,
                        true,
                    );
                    if index.is_some() {
                        layout_node_impl(
                            ctx,
                            index,
                            index_size,
                            x,
                            index_baseline,
                            child_style,
                            delimiter_height,
                            true,
                        );
                    }
                    let shoulder = bottom - (cm.height() * 0.4).min(em * 0.45);
                    ctx.out.lines.push(MathLine {
                        x1: start,
                        y1: shoulder + em * 0.06,
                        x2: start + em * 0.16,
                        y2: shoulder,
                        thickness: stroke,
                    });
                    ctx.out.lines.push(MathLine {
                        x1: start + em * 0.16,
                        y1: shoulder,
                        x2: start + em * 0.33,
                        y2: bottom,
                        thickness: stroke,
                    });
                    ctx.out.lines.push(MathLine {
                        x1: start + em * 0.33,
                        y1: bottom,
                        x2: cx - em * 0.06,
                        y2: top,
                        thickness: stroke,
                    });
                    ctx.out.lines.push(MathLine {
                        x1: cx - em * 0.06,
                        y1: top,
                        x2: cx + cm.width + em * 0.1,
                        y2: top,
                        thickness: stroke,
                    });
                }
                m.width = cx - x + cm.width + em * 0.1;
                m.ascent = (baseline_y - top + stroke / 2.0).max(if index.is_some() {
                    baseline_y - index_baseline + im.ascent
                } else {
                    0.0
                });
                m.descent = cm.descent + stroke / 2.0;
                return m;
            }

            if node.deco_kind >= 5 {
                let cm = measure_node(ctx, node.kid(0), size, style, delimiter_height);
                let kind = node.deco_kind;
                let pad = if kind == 9 { em * 0.2 } else { 0.0 };
                m = cm;
                m.width += 2.0 * pad;
                let stroke = (em * 0.055).max(1.0);
                let top = baseline_y - cm.ascent;
                let bottom = baseline_y + cm.descent;
                let under = kind == 8 || kind == 13;
                let overlay = matches!(kind, 10 | 11 | 18 | 19);
                let extra = if overlay {
                    0.0
                } else if kind == 12 || kind == 13 {
                    em * 0.45
                } else {
                    em * 0.3
                };
                if under {
                    m.descent += extra;
                } else {
                    m.ascent += extra;
                }
                if kind == 9 {
                    m.descent += extra;
                }
                if record {
                    layout_node_impl(
                        ctx,
                        node.kid(0),
                        size,
                        x + pad,
                        baseline_y,
                        style,
                        delimiter_height,
                        true,
                    );
                    let mut new_lines: Vec<MathLine> = Vec::new();
                    let mut line = |x1: f32, y1: f32, x2: f32, y2: f32| {
                        new_lines.push(MathLine {
                            x1,
                            y1,
                            x2,
                            y2,
                            thickness: stroke,
                        });
                    };
                    let center = x + m.width / 2.0;
                    if kind == 5 || kind == 6 {
                        let dot = em * 0.1;
                        let y = top - em * 0.2;
                        if kind == 5 {
                            ctx.out.rules.push(MathRule {
                                x: center - dot / 2.0,
                                y,
                                w: dot,
                                h: dot,
                            });
                        } else {
                            ctx.out.rules.push(MathRule {
                                x: center - em * 0.16,
                                y,
                                w: dot,
                                h: dot,
                            });
                            ctx.out.rules.push(MathRule {
                                x: center + em * 0.06,
                                y,
                                w: dot,
                                h: dot,
                            });
                        }
                    } else if kind == 7 || kind == 16 {
                        let w = if kind == 16 {
                            cm.width.min(em * 0.55)
                        } else {
                            cm.width
                        };
                        let mut prev_x = center - w / 2.0;
                        let mut prev_y = top - em * 0.17;
                        for i in 1..=16 {
                            let t = i as f32 / 16.0;
                            let nx = center - w / 2.0 + w * t;
                            let ny = top - em * 0.17
                                + em * 0.07
                                    * (t * if kind == 7 {
                                        -std::f32::consts::TAU
                                    } else {
                                        std::f32::consts::PI
                                    })
                                    .sin();
                            line(prev_x, prev_y, nx, ny);
                            prev_x = nx;
                            prev_y = ny;
                        }
                    } else if kind == 8 {
                        line(x, bottom + em * 0.12, x + cm.width, bottom + em * 0.12);
                    } else if kind == 9 {
                        let y1 = top - pad;
                        let y2 = bottom + pad;
                        line(x, y1, x + m.width, y1);
                        line(x, y2, x + m.width, y2);
                        line(x, y1, x, y2);
                        line(x + m.width, y1, x + m.width, y2);
                    } else if overlay {
                        if kind != 11 {
                            line(x, bottom, x + cm.width, top);
                        }
                        if kind == 11 || kind == 19 {
                            line(x, top, x + cm.width, bottom);
                        }
                    } else if kind == 12 || kind == 13 {
                        let sign = if under { 1.0 } else { -1.0 };
                        let y = if under {
                            bottom + em * 0.12
                        } else {
                            top - em * 0.12
                        };
                        let h = em * 0.18;
                        line(x, y, x + em * 0.15, y + sign * h);
                        line(
                            x + em * 0.15,
                            y + sign * h,
                            center - em * 0.15,
                            y + sign * h,
                        );
                        line(
                            center - em * 0.15,
                            y + sign * h,
                            center,
                            y + sign * h * 1.5,
                        );
                        line(
                            center,
                            y + sign * h * 1.5,
                            center + em * 0.15,
                            y + sign * h,
                        );
                        line(
                            center + em * 0.15,
                            y + sign * h,
                            x + cm.width - em * 0.15,
                            y + sign * h,
                        );
                        line(x + cm.width - em * 0.15, y + sign * h, x + cm.width, y);
                    } else if kind == 17 {
                        line(
                            center - em * 0.18,
                            top - em * 0.25,
                            center,
                            top - em * 0.1,
                        );
                        line(
                            center,
                            top - em * 0.1,
                            center + em * 0.18,
                            top - em * 0.25,
                        );
                    } else {
                        let sign = if kind == 14 { 1.0 } else { -1.0 };
                        line(
                            center - sign * em * 0.1,
                            top - em * 0.1,
                            center + sign * em * 0.1,
                            top - em * 0.27,
                        );
                    }
                    ctx.out.lines.extend(new_lines);
                }
                return m;
            }

            // overline / overrightarrow / hat
            let cm = measure_node(ctx, node.kid(0), size, style, delimiter_height);
            let gap = em * 0.12;
            let rule_h = (em * 0.05).max(1.0);
            let inner = layout_node_impl(
                ctx,
                node.kid(0),
                size,
                x,
                baseline_y,
                style,
                delimiter_height,
                record,
            );
            let top_y = baseline_y - cm.ascent - gap - rule_h;
            if record {
                match node.deco_kind {
                    1 => {
                        ctx.out.rules.push(MathRule {
                            x,
                            y: top_y,
                            w: cm.width,
                            h: rule_h,
                        });
                    }
                    2 => {
                        let aw = em * 0.22;
                        let ay = top_y + rule_h / 2.0;
                        ctx.out.rules.push(MathRule {
                            x,
                            y: top_y,
                            w: cm.width,
                            h: rule_h,
                        });
                        ctx.out.lines.push(MathLine {
                            x1: x + cm.width - aw,
                            y1: ay - aw * 0.6,
                            x2: x + cm.width,
                            y2: ay,
                            thickness: rule_h,
                        });
                        ctx.out.lines.push(MathLine {
                            x1: x + cm.width - aw,
                            y1: ay + aw * 0.6,
                            x2: x + cm.width,
                            y2: ay,
                            thickness: rule_h,
                        });
                    }
                    4 => {
                        let cxm = x + cm.width / 2.0;
                        let hw = (cm.width / 2.0).min(em * 0.28);
                        ctx.out.lines.push(MathLine {
                            x1: cxm - hw,
                            y1: top_y + rule_h + em * 0.08,
                            x2: cxm,
                            y2: top_y,
                            thickness: rule_h,
                        });
                        ctx.out.lines.push(MathLine {
                            x1: cxm,
                            y1: top_y,
                            x2: cxm + hw,
                            y2: top_y + rule_h + em * 0.08,
                            thickness: rule_h,
                        });
                    }
                    _ => {}
                }
            }
            m.width = inner.width.max(cm.width);
            m.ascent = cm.ascent + gap + rule_h + em * 0.08;
            m.descent = cm.descent;
            m
        }
    }
}

/// Port de `mathParse`: mide y coloca el árbol completo.
pub fn layout(root: &MNode, font_size: f32, display: bool, measurer: &mut dyn Measurer) -> MathBox {
    let mut out = MathBox::default();
    let mut ctx = LayoutCtx {
        measurer,
        out: &mut out,
    };
    let style = if display { 0 } else { 1 };
    let probe = measure_node(&mut ctx, Some(root), font_size, style, 0.0);
    let pad = font_size * if display { 0.15 } else { 0.08 };
    let baseline = probe.ascent + pad;
    let final_metrics = layout_node(&mut ctx, Some(root), font_size, pad, baseline, style, 0.0, true);
    out.width = final_metrics.width + pad * 2.0;
    out.height = final_metrics.ascent + final_metrics.descent + pad * 2.0;
    out.baseline = baseline;
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Medidor determinista: ancho proporcional al número de caracteres.
    struct FakeMeasurer;

    impl Measurer for FakeMeasurer {
        fn measure(&mut self, text: &str, size: f32, _italic: bool, _bold: bool) -> Metrics {
            Metrics {
                width: 0.5 * size * text.chars().count() as f32,
                ascent: 0.8 * size,
                descent: 0.2 * size,
            }
        }
    }

    fn sym(text: &str) -> MNode {
        MNode::sym(text)
    }

    fn row(kids: Vec<MNode>) -> MNode {
        let mut node = MNode::new(Kind::Row);
        node.kids = kids.into_iter().map(|k| Some(Box::new(k))).collect();
        node
    }

    fn frac(num: MNode, den: MNode) -> MNode {
        let mut node = MNode::new(Kind::Frac);
        node.kids = vec![Some(Box::new(num)), Some(Box::new(den))];
        node
    }

    fn script(base: MNode, sub: Option<MNode>, sup: Option<MNode>) -> MNode {
        let mut node = MNode::new(Kind::Script);
        node.kids = vec![
            Some(Box::new(base)),
            sub.map(|n| Box::new(n)),
            sup.map(|n| Box::new(n)),
        ];
        node
    }

    fn delim(open: char, close: char, content: MNode) -> MNode {
        let mut node = MNode::new(Kind::Delim);
        node.open = open;
        node.close = close;
        node.kids = vec![Some(Box::new(content))];
        node
    }

    fn grid(cells: Vec<Vec<MNode>>, alignment: &str) -> MNode {
        let mut node = MNode::new(Kind::Grid);
        node.cells = cells;
        node.alignment = alignment.to_owned();
        node
    }

    fn deco(kind: i32, content: MNode) -> MNode {
        let mut node = MNode::new(Kind::Deco);
        node.deco_kind = kind;
        node.kids = vec![Some(Box::new(content))];
        node
    }

    fn run_layout(node: &MNode, display: bool) -> MathBox {
        layout(node, 16.0, display, &mut FakeMeasurer)
    }

    #[test]
    fn frac_generates_rule_and_positive_size() {
        let node = frac(sym("a"), sym("b"));
        let boxed = run_layout(&node, true);
        assert!(boxed.width > 0.0);
        assert!(boxed.height > 0.0);
        assert!(!boxed.rules.is_empty(), "una fracción debe tener barra");
        assert!(!boxed.runs.is_empty());
    }

    #[test]
    fn script_uses_smaller_size() {
        let node = script(sym("x"), None, Some(sym("2")));
        let boxed = run_layout(&node, false);
        assert!(boxed.width > 0.0 && boxed.height > 0.0);
        let base = boxed
            .runs
            .iter()
            .find(|r| r.text == "x")
            .expect("run base");
        let sup = boxed
            .runs
            .iter()
            .find(|r| r.text == "2")
            .expect("run superíndice");
        assert!(
            sup.size < base.size,
            "el script {} debería ser menor que la base {}",
            sup.size,
            base.size
        );
    }

    #[test]
    fn sub_and_sup_both_placed() {
        let node = script(sym("x"), Some(sym("i")), Some(sym("2")));
        let boxed = run_layout(&node, false);
        assert!(boxed.width > 0.0 && boxed.height > 0.0);
        assert!(boxed.runs.iter().any(|r| r.text == "i"));
        assert!(boxed.runs.iter().any(|r| r.text == "2"));
    }

    #[test]
    fn sqrt_has_lines_and_positive_size() {
        let node = deco(3, sym("x"));
        let boxed = run_layout(&node, true);
        assert!(boxed.width > 0.0 && boxed.height > 0.0);
        assert!(!boxed.lines.is_empty(), "sqrt dibuja su signo con líneas");
    }

    #[test]
    fn delimited_frac_positive_size() {
        let node = delim('(', ')', frac(sym("a"), sym("b")));
        let boxed = run_layout(&node, true);
        assert!(boxed.width > 0.0 && boxed.height > 0.0);
    }

    #[test]
    fn matrix_2x2_positive_size() {
        let node = grid(
            vec![
                vec![sym("a"), sym("b")],
                vec![sym("c"), sym("d")],
            ],
            "cc",
        );
        let boxed = run_layout(&node, true);
        assert!(boxed.width > 0.0 && boxed.height > 0.0);
        for text in ["a", "b", "c", "d"] {
            assert!(boxed.runs.iter().any(|r| r.text == text), "falta {text}");
        }
    }

    #[test]
    fn cases_grid_positive_size() {
        let mut node = grid(
            vec![
                vec![row(vec![sym("x"), sym(">"), sym("0")])],
                vec![row(vec![sym("x"), sym("<"), sym("0")])],
            ],
            "l",
        );
        node.vertical_rules = vec![1];
        let boxed = run_layout(&node, true);
        assert!(boxed.width > 0.0 && boxed.height > 0.0);
        assert!(!boxed.lines.is_empty(), "cases dibuja la llave/regla vertical");
    }

    #[test]
    fn aligned_grid_positive_size() {
        let mut node = grid(
            vec![
                vec![sym("a"), sym("="), sym("b")],
                vec![sym("c"), sym("="), sym("d")],
            ],
            "rl",
        );
        node.aligned = true;
        let boxed = run_layout(&node, true);
        assert!(boxed.width > 0.0 && boxed.height > 0.0);
    }

    #[test]
    fn sum_with_limits_uses_smaller_scripts() {
        let mut base = sym("\u{2211}");
        base.limit_op = true;
        let node = script(base, Some(sym("i=1")), Some(sym("n")));
        let boxed = run_layout(&node, true); // display centra los límites
        assert!(boxed.width > 0.0 && boxed.height > 0.0);
        let sum = boxed
            .runs
            .iter()
            .find(|r| r.text == "\u{2211}")
            .expect("run suma");
        let sub = boxed
            .runs
            .iter()
            .find(|r| r.text == "i=1")
            .expect("run límite inferior");
        assert!(sub.size < sum.size);
    }

    #[test]
    fn overline_and_vec_positive_size() {
        for kind in [1, 2] {
            let node = deco(kind, sym("x"));
            let boxed = run_layout(&node, true);
            assert!(boxed.width > 0.0 && boxed.height > 0.0);
        }
    }

    #[test]
    fn hspace_has_width_and_no_height() {
        let mut node = MNode::new(Kind::Space);
        node.space = 1.0;
        let boxed = run_layout(&node, true);
        assert!(boxed.width > 0.0);
        // Nodo sin texto: solo el padding vertical del layout.
        assert!(boxed.height > 0.0);
        assert!(boxed.runs.is_empty());
    }

    #[test]
    fn phantom_keeps_metrics_but_draws_nothing() {
        let mut node = MNode::new(Kind::Phantom);
        node.deco_kind = 0;
        node.kids = vec![Some(Box::new(sym("x")))];
        let boxed = run_layout(&node, true);
        assert!(boxed.width > 0.0 && boxed.height > 0.0);
        assert!(boxed.runs.is_empty(), "phantom no debe registrar runs");
    }

    #[test]
    fn empty_nodes_do_not_panic() {
        let empty = MNode::new(Kind::Row);
        let boxed = run_layout(&empty, true);
        assert!(boxed.height > 0.0);
        assert!(boxed.runs.is_empty());

        let mut empty_frac = MNode::new(Kind::Frac);
        empty_frac.kids = vec![None, None];
        let _ = run_layout(&empty_frac, true);

        let empty_script = MNode::new(Kind::Script);
        let _ = run_layout(&empty_script, true);

        let empty_grid = MNode::new(Kind::Grid);
        let _ = run_layout(&empty_grid, true);

        let empty_delim = MNode::new(Kind::Delim);
        let _ = run_layout(&empty_delim, true);

        let empty_phantom = MNode::new(Kind::Phantom);
        let _ = run_layout(&empty_phantom, true);

        let empty_deco = MNode::new(Kind::Deco);
        let _ = run_layout(&empty_deco, true);

        let empty_middle = MNode::new(Kind::Middle);
        let _ = run_layout(&empty_middle, true);
    }

    #[test]
    fn display_and_inline_are_coherent() {
        let node = frac(sym("a"), sym("b"));
        let display = run_layout(&node, true);
        let inline = run_layout(&node, false);
        assert!(display.width > 0.0 && inline.width > 0.0);
        assert!(display.height > 0.0 && inline.height > 0.0);
        // En display la fracción usa el tamaño completo; en inline, script.
        assert!(
            display.width >= inline.width,
            "display {} debería ser al menos tan ancho como inline {}",
            display.width,
            inline.width
        );
        assert!(
            display.height >= inline.height,
            "display {} debería ser al menos tan alto como inline {}",
            display.height,
            inline.height
        );
        assert!(display.baseline > 0.0 && inline.baseline > 0.0);
    }

    #[test]
    fn middle_delimiter_has_lines() {
        let mut node = MNode::new(Kind::Middle);
        node.open = '|';
        node.delimiter_size = 2.0;
        let boxed = run_layout(&node, true);
        assert!(boxed.width > 0.0 && boxed.height > 0.0);
        assert!(!boxed.lines.is_empty());
    }

    #[test]
    fn reported_formulas_layout_without_panicking() {
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
        for formula in formulas {
            let node = crate::latex_math::parse_latex(formula)
                .unwrap_or_else(|| panic!("no parsea: {formula}"));
            let boxed = run_layout(&node, true);
            assert!(boxed.width.is_finite() && boxed.width > 0.0, "{formula}");
            assert!(boxed.height.is_finite() && boxed.height > 0.0, "{formula}");
            assert!(!boxed.runs.is_empty(), "{formula}");
            assert!(boxed.baseline.is_finite(), "{formula}");
        }
    }
}
