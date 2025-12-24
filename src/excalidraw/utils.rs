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

pub fn is_point_inside(el: &ExcalidrawElement, p: Pos2) -> bool {
    Rect::from_min_size(Pos2::new(el.x, el.y), Vec2::new(el.width, el.height))
        .expand(10.0)
        .contains(p)
}
