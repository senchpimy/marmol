use std::{
    collections::HashMap,
    sync::{
        Arc,
        atomic::{AtomicU64, Ordering::Relaxed},
    },
};

use egui::{
    ColorImage,
    load::{BytesPoll, ImageLoadResult, ImageLoader, ImagePoll, LoadError, SizeHint},
    mutex::Mutex,
};
use resvg::{
    tiny_skia::Pixmap,
    usvg::{Options, Transform, Tree},
};

/// Candidatos de fuentes instaladas para cada familia genérica CSS.
///
/// `usvg`/`resvg` no resuelven las familias genéricas (`sans-serif`, `serif`, …)
/// solas: su `fontdb` las traduce a un nombre vacío salvo que se configuren con
/// `set_*_family`. Aquí las mapeamos a la primera fuente real instalada que exista.
const SANS_SERIF_CANDIDATES: &[&str] = &[
    "Noto Sans",
    "DejaVu Sans",
    "Liberation Sans",
    "Roboto",
    "Arial",
    "Helvetica",
    "Segoe UI",
    "Open Sans",
    "Noto Sans CJK SC",
    "Inter",
];
const SERIF_CANDIDATES: &[&str] = &[
    "Noto Serif",
    "DejaVu Serif",
    "Liberation Serif",
    "Times New Roman",
    "Noto Serif CJK SC",
];
const MONOSPACE_CANDIDATES: &[&str] = &[
    "Noto Sans Mono",
    "DejaVu Sans Mono",
    "Liberation Mono",
    "Consolas",
    "Courier New",
    "Roboto Mono",
];
const CURSIVE_CANDIDATES: &[&str] = &["Comic Sans MS", "Comic Neue", "Segoe Script"];
const FANTASY_CANDIDATES: &[&str] = &["Impact", "Copperplate", "Papyrus"];

/// Configura las opciones de renderizado SVG con un `fontdb` que sí resuelve las
/// familias genéricas de CSS, de forma que el texto de los diagramas (mermaid, vega,
/// TikZ, latex) se dibuje aunque la fuente del SVG no esté instalada.
fn build_options() -> Options<'static> {
    use resvg::usvg::fontdb;

    let mut options = Options::default();
    {
        let db = options.fontdb_mut();
        db.load_system_fonts();

        fn first_installed(db: &fontdb::Database, candidates: &[&str]) -> Option<String> {
            candidates
                .iter()
                .copied()
                .find(|name| {
                    db.query(&fontdb::Query {
                        families: &[fontdb::Family::Name(name)],
                        weight: fontdb::Weight(400),
                        stretch: fontdb::Stretch::Normal,
                        style: fontdb::Style::Normal,
                    })
                    .is_some()
                })
                .map(str::to_owned)
        }

        if let Some(f) = first_installed(db, SANS_SERIF_CANDIDATES) {
            db.set_sans_serif_family(f);
        }
        if let Some(f) = first_installed(db, SERIF_CANDIDATES) {
            db.set_serif_family(f);
        }
        if let Some(f) = first_installed(db, MONOSPACE_CANDIDATES) {
            db.set_monospace_family(f);
        }
        if let Some(f) = first_installed(db, CURSIVE_CANDIDATES) {
            db.set_cursive_family(f);
        }
        if let Some(f) = first_installed(db, FANTASY_CANDIDATES) {
            db.set_fantasy_family(f);
        }
    }
    options
}

struct Entry {
    last_used: AtomicU64,
    result: Result<Arc<ColorImage>, String>,
}

/// `ImageLoader` propio para SVGs en memoria (`bytes://…svg`).
///
/// Reemplaza al `SvgLoader` de `egui_extras`, cuyo `fontdb` interno no resuelve las
/// familias genéricas CSS y por tanto **no dibuja el texto** de los diagramas
/// generados por mermaid/vega/TikZ.
pub struct SvgLoader {
    pass_index: AtomicU64,
    cache: Mutex<HashMap<String, HashMap<SizeHint, Entry>>>,
    options: Options<'static>,
}

impl SvgLoader {
    pub const ID: &'static str = concat!(module_path!(), "::SvgLoader");
}

impl Default for SvgLoader {
    fn default() -> Self {
        Self {
            pass_index: AtomicU64::new(0),
            cache: Mutex::new(HashMap::default()),
            options: build_options(),
        }
    }
}

pub fn install_svg_loader(ctx: &egui::Context) {
    if !ctx.is_loader_installed(SvgLoader::ID) {
        ctx.add_image_loader(Arc::new(SvgLoader::default()));
    }
}

fn is_supported(uri: &str) -> bool {
    uri.ends_with(".svg")
}

fn load_svg_bytes_with_size(
    svg_bytes: &[u8],
    size_hint: SizeHint,
    options: &Options<'_>,
) -> Result<ColorImage, String> {
    use egui::Vec2;

    let rtree = Tree::from_data(svg_bytes, options).map_err(|err| err.to_string())?;

    let source_size = Vec2::new(rtree.size().width(), rtree.size().height());

    let scaled_size = match size_hint {
        SizeHint::Size {
            width,
            height,
            maintain_aspect_ratio,
        } => {
            if maintain_aspect_ratio {
                let mut size = source_size;
                size *= width as f32 / source_size.x;
                if size.y > height as f32 {
                    size *= height as f32 / size.y;
                }
                size
            } else {
                Vec2::new(width as _, height as _)
            }
        }
        SizeHint::Height(h) => source_size * (h as f32 / source_size.y),
        SizeHint::Width(w) => source_size * (w as f32 / source_size.x),
        SizeHint::Scale(scale) => scale.into_inner() * source_size,
    };

    let scaled_size = scaled_size.round();
    let (w, h) = (scaled_size.x as u32, scaled_size.y as u32);

    let mut pixmap =
        Pixmap::new(w, h).ok_or_else(|| format!("Failed to create SVG Pixmap of size {w}x{h}"))?;

    resvg::render(
        &rtree,
        Transform::from_scale(w as f32 / source_size.x, h as f32 / source_size.y),
        &mut pixmap.as_mut(),
    );

    Ok(ColorImage::from_rgba_premultiplied([w as _, h as _], pixmap.data())
        .with_source_size(source_size))
}

impl ImageLoader for SvgLoader {
    fn id(&self) -> &str {
        Self::ID
    }

    fn load(&self, ctx: &egui::Context, uri: &str, size_hint: SizeHint) -> ImageLoadResult {
        if !is_supported(uri) {
            return Err(LoadError::NotSupported);
        }

        let mut cache = self.cache.lock();
        let bucket = cache.entry(uri.to_owned()).or_default();

        if let Some(entry) = bucket.get(&size_hint) {
            entry
                .last_used
                .store(self.pass_index.load(Relaxed), Relaxed);
            match entry.result.clone() {
                Ok(image) => Ok(ImagePoll::Ready { image }),
                Err(err) => Err(LoadError::Loading(err)),
            }
        } else {
            match ctx.try_load_bytes(uri) {
                Ok(BytesPoll::Ready { bytes, .. }) => {
                    let result = load_svg_bytes_with_size(&bytes, size_hint, &self.options)
                        .map(Arc::new);
                    bucket.insert(
                        size_hint,
                        Entry {
                            last_used: AtomicU64::new(self.pass_index.load(Relaxed)),
                            result: result.clone(),
                        },
                    );
                    match result {
                        Ok(image) => Ok(ImagePoll::Ready { image }),
                        Err(err) => Err(LoadError::Loading(err)),
                    }
                }
                Ok(BytesPoll::Pending { size }) => Ok(ImagePoll::Pending { size }),
                Err(err) => Err(err),
            }
        }
    }

    fn forget(&self, uri: &str) {
        self.cache.lock().retain(|key, _| key != uri);
    }

    fn forget_all(&self) {
        self.cache.lock().clear();
    }

    fn byte_size(&self) -> usize {
        self.cache
            .lock()
            .values()
            .flat_map(|bucket| bucket.values())
            .map(|entry| match &entry.result {
                Ok(image) => image.pixels.len() * size_of::<egui::Color32>(),
                Err(err) => err.len(),
            })
            .sum()
    }

    fn end_pass(&self, pass_index: u64) {
        self.pass_index.store(pass_index, Relaxed);
        let mut cache = self.cache.lock();
        cache.retain(|_key, bucket| {
            if 2 <= bucket.len() {
                bucket.retain(|_, texture| pass_index <= texture.last_used.load(Relaxed) + 1);
            }
            !bucket.is_empty()
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const MERMAID_SVG: &str = r##"<svg xmlns="http://www.w3.org/2000/svg" width="197.6304" height="168" viewBox="0 0 197.6304 168"><rect x="0" y="0" width="197.6304" height="168" fill="#FFFFFF"/><path d="M 98.815,59.000L 98.815,60.666C98.815,62.332,98.815,65.663,98.815,68.580C98.815,71.497,98.815,73.998,98.770,76.499C98.724,79.000,98.633,81.500,98.587,82.750C98.542,84.000,98.542,84.000,98.496,85.250C98.450,86.500,98.359,89.000,98.313,91.501C98.268,94.002,98.268,96.503,98.268,99.420C98.268,102.337,98.268,105.668,98.268,107.334L 98.268,109.000" fill="none" stroke="#64748B" stroke-width="2"    stroke-linecap="round" stroke-linejoin="round" /><rect x="8.00" y="8.00" width="181.63" height="51.00" rx="3" ry="3" fill="#F8FAFC" stroke="#94A3B8" stroke-width="1" stroke-linejoin="round" stroke-linecap="round"/><text x="98.82" y="37.00" text-anchor="middle" font-family="Inter, ui-sans-serif, system-ui, -apple-system, 'Segoe UI', sans-serif" font-size="14" fill="#0F172A"><tspan x="98.82" dy="0.00">Texto de prueba</tspan></text><rect x="31.78" y="109.00" width="132.98" height="51.00" rx="3" ry="3" fill="#F8FAFC" stroke="#94A3B8" stroke-width="1" stroke-linejoin="round" stroke-linecap="round"/><text x="98.27" y="138.00" text-anchor="middle" font-family="Inter, ui-sans-serif, system-ui, -apple-system, 'Segoe UI', sans-serif" font-size="14" fill="#0F172A"><tspan x="98.27" dy="0.00">Otro nodo</tspan></text></svg>"##;

    /// Cuenta píxeles muy oscuros (el color del texto) estrictamente dentro de un rect.
    fn dark_in_rect(img: &ColorImage, x0: usize, x1: usize, y0: usize, y1: usize) -> usize {
        let (w, h) = (img.size[0], img.size[1]);
        let mut n = 0;
        for y in y0..y1.min(h) {
            for x in x0..x1.min(w) {
                let p = img.pixels[y * w + x];
                if p.r() < 60 && p.g() < 60 && p.b() < 80 {
                    n += 1;
                }
            }
        }
        n
    }

    #[test]
    fn mermaid_text_is_rendered() {
        let options = build_options();
        let image = load_svg_bytes_with_size(
            MERMAID_SVG.as_bytes(),
            SizeHint::Scale(egui::emath::OrderedFloat(1.0)),
            &options,
        )
        .expect("svg should load");

        // Rect1: x=8..190, y=8..59 | Rect2: x=31..165, y=109..160 (bordes excluidos)
        let t1 = dark_in_rect(&image, 10, 188, 10, 58);
        let t2 = dark_in_rect(&image, 33, 163, 111, 159);

        assert!(
            t1 > 0,
            "el texto del primer nodo no se dibujó (píxeles oscuros: {t1})"
        );
        assert!(
            t2 > 0,
            "el texto del segundo nodo no se dibujó (píxeles oscuros: {t2})"
        );
    }
}