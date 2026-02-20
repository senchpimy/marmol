use egui::{Color32, Pos2, Rect, Vec2};
use super::data::ExcalidrawElement;

pub fn hex_to_color(hex: &str) -> Color32 {
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

pub fn color_to_hex(c: Color32) -> String {
    if c == Color32::TRANSPARENT {
        return "transparent".into();
    }
    format!("#{:02x}{:02x}{:02x}", c.r(), c.g(), c.b())
}

pub fn move_element_group(elements: &mut Vec<ExcalidrawElement>, root_idx: usize, delta: Vec2) {
    let mut indices = vec![root_idx];
    let rid = &elements[root_idx].id;
    let cid = elements[root_idx].container_id.clone();
    if let Some(c) = cid {
        if let Some(p) = elements.iter().position(|e| e.id == *c) {
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
            if c == rid && !indices.contains(&i) {
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

pub fn is_point_inside(el: &ExcalidrawElement, p: Pos2) -> bool {
    // Transformamos el punto al espacio local del elemento (sin rotación, relativo al centro)
    let center = Pos2::new(el.x + el.width / 2.0, el.y + el.height / 2.0);
    let cl = Vec2::new(el.width / 2.0, el.height / 2.0);
    let rot_inv = egui::emath::Rot2::from_angle(-el.angle);
    let p_rel = (rot_inv * (p - center)).to_pos2();

    match el.element_type.as_str() {
        "line" | "arrow" | "freedraw" | "draw" => {
            if el.points.len() < 2 {
                return false;
            }
            let margin = (el.stroke_width as f32).max(10.0);
            for i in 0..el.points.len() - 1 {
                // Los puntos en el.points son relativos a x,y.
                // En nuestro espacio local (relativo al centro), p_local = p_in_points - cl
                let p1 = Pos2::new(el.points[i][0], el.points[i][1]) - cl;
                let p2 = Pos2::new(el.points[i + 1][0], el.points[i + 1][1]) - cl;
                if dist_to_segment(p_rel, p1, p2) < margin {
                    return true;
                }
            }
            false
        }
        _ => {
            // El rectángulo local va de [-w/2, -h/2] a [w/2, h/2]
            Rect::from_center_size(Pos2::ZERO, Vec2::new(el.width, el.height))
                .expand(10.0)
                .contains(p_rel)
        }
    }
}

pub fn normalize_element(el: &mut ExcalidrawElement) {
    if el.points.is_empty() {
        return;
    }

    let mut min_x = el.points[0][0];
    let mut min_y = el.points[0][1];
    let mut max_x = el.points[0][0];
    let mut max_y = el.points[0][1];

    for p in &el.points {
        min_x = min_x.min(p[0]);
        min_y = min_y.min(p[1]);
        max_x = max_x.max(p[0]);
        max_y = max_y.max(p[1]);
    }

    if min_x != 0.0 || min_y != 0.0 {
        el.x += min_x;
        el.y += min_y;
        for p in &mut el.points {
            p[0] -= min_x;
            p[1] -= min_y;
        }
    }

    el.width = max_x - min_x;
    el.height = max_y - min_y;
}

fn dist_to_segment(p: Pos2, a: Pos2, b: Pos2) -> f32 {
    let l2 = a.distance_sq(b);
    if l2 == 0.0 {
        return p.distance(a);
    }
    let t = ((p.x - a.x) * (b.x - a.x) + (p.y - a.y) * (b.y - a.y)) / l2;
    let t = t.clamp(0.0, 1.0);
    p.distance(Pos2::new(a.x + t * (b.x - a.x), a.y + t * (b.y - a.y)))
}
