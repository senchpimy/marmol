use egui::{
    emath::Rot2, Color32, FontFamily, FontId, Pos2, Rect, Shape, Stroke, TextureHandle,
    Vec2,
};
use super::data::ExcalidrawElement;
use super::utils::hex_to_color;

pub fn draw_selection_border<F>(painter: &egui::Painter, el: &ExcalidrawElement, to_screen: &F, _s: f32, ui: &egui::Ui)
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
        Stroke::new(1.0, ui.ctx().style().visuals.selection.stroke.color),
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

pub fn draw_element<F>(
    painter: &egui::Painter,
    el: &ExcalidrawElement,
    tex: Option<TextureHandle>,
    to_screen: &F,
    sc: f32,
) where
    F: Fn(Pos2) -> Pos2,
{
    if el.opacity == 0 {
        return;
    }
    let a = ((el.opacity as f32 / 100.0) * 255.0) as u8;
    let sc_col = hex_to_color(&el.stroke_color).linear_multiply(a as f32 / 255.0); // Simple alpha fix
    let bg_col = hex_to_color(&el.background_color).linear_multiply(a as f32 / 255.0);
    let s = Stroke::new(el.stroke_width as f32 * sc, sc_col);
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
                FontId::new(el.font_size.unwrap_or(20.0) * sc, FontFamily::Proportional),
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
