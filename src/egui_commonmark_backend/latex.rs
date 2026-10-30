use crate::egui_commonmark_backend::misc::CommonMarkCache;
use egui::{Color32, RichText, Ui};
use std::path::PathBuf;
use std::sync::OnceLock;
use time::OffsetDateTime;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

/// Partes del `World` de Typst que son idénticas para todas las fórmulas:
/// la librería estándar, el índice de fuentes y la fuente ya parseada.
///
/// Se construyen una sola vez. La fuente se busca en el sistema (igual que hace
/// Tinta con Cambria Math); si no existe, `font` es `None` y las fórmulas se
/// muestran como texto.
struct TypstParts {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    font: Option<Font>,
}

fn typst_parts() -> &'static TypstParts {
    static PARTS: OnceLock<TypstParts> = OnceLock::new();
    PARTS.get_or_init(|| {
        let font = crate::math_font::system_math_font().cloned();
        let book = match &font {
            Some(font) => FontBook::from_fonts(std::slice::from_ref(font)),
            None => FontBook::new(),
        };
        TypstParts {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(book),
            font,
        }
    })
}

/// Indica si hay una fuente matemática disponible para renderizar LaTeX.
pub fn has_math_font() -> bool {
    crate::math_font::system_math_font().is_some()
}

/// Muestra el LaTeX sin renderizar cuando no hay fuente matemática.
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

struct MinimalWorld {
    parts: &'static TypstParts,
    source: Source,
    time: time::OffsetDateTime,
}

impl MinimalWorld {
    fn new(source_text: String) -> Self {
        Self {
            parts: typst_parts(),
            source: Source::detached(source_text),
            time: OffsetDateTime::now_utc(),
        }
    }
}

impl World for MinimalWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.parts.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.parts.book
    }
    fn main(&self) -> FileId {
        self.source.id()
    }
    fn source(&self, id: FileId) -> FileResult<Source> {
        if id == self.source.id() {
            Ok(self.source.clone())
        } else {
            Err(FileError::NotFound(PathBuf::new()))
        }
    }
    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(PathBuf::new()))
    }
    fn font(&self, index: usize) -> Option<Font> {
        if index == 0 {
            self.parts.font.clone()
        } else {
            None
        }
    }
    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Some(Datetime::Date(self.time.date()))
    }
}

pub fn render(ui: &mut Ui, cache: &mut CommonMarkCache, latex: &str, inline: bool, font_size: f32) {
    if !has_math_font() {
        render_as_text(ui, latex, inline);
        return;
    }

    let text_color = ui.visuals().text_color();
    let color_hex = format!(
        "#{:02x}{:02x}{:02x}",
        text_color.r(),
        text_color.g(),
        text_color.b()
    );
    let cache_key = format!("{}_{}_{}_{}", latex, inline, color_hex, font_size);

    let svg_content = if let Some(svg) = cache.latex_cache.get(&cache_key) {
        Some(svg.clone())
    } else if let Some(err) = cache.latex_error_cache.get(&cache_key) {
        ui.label(RichText::new(format!("LaTeX error: {}", err)).color(Color32::RED));
        None
    } else {
        match render_to_svg(latex, inline, &color_hex, font_size) {
            Ok(svg) => {
                super::misc::insert_cached(&mut cache.latex_cache, cache_key.clone(), svg.clone());
                Some(svg)
            }
            Err(err) => {
                super::misc::insert_cached(
                    &mut cache.latex_error_cache,
                    cache_key.clone(),
                    err.clone(),
                );
                ui.label(RichText::new(format!("LaTeX error: {}", err)).color(Color32::RED));
                None
            }
        }
    };

    if let Some(svg) = svg_content {
        let uri = format!("bytes://latex_{}.svg", egui::Id::new(&cache_key).value());
        let mut img = egui::Image::from_bytes(uri, svg.as_bytes().to_vec());

        if inline || (latex.len() < 50 && !latex.contains('\n')) {
            // No need to fit to original size 1.3 if we are already using the correct font size
            img = img.fit_to_original_size(1.0); 

            ui.vertical(|ui| {
                ui.add_space(1.0);
                ui.add(img);
            });
        } else {
            img = img.fit_to_original_size(1.0);
            ui.vertical_centered(|ui| {
                ui.add(img.max_width(ui.available_width()));
            });
        }
    }
}

pub fn render_to_svg(latex_input: &str, inline: bool, color_hex: &str, font_size: f32) -> Result<String, String> {
    // Fuente + librería + índice de fuentes se reutilizan (se construyen una vez).
    let parts = typst_parts();
    let Some(font) = &parts.font else {
        return Err("No hay ninguna fuente matemática instalada".to_string());
    };

    // Convertir LaTeX a Typst Math
    let clean_latex = latex_input.replace('\n', "");
    let typst_math = crate::format::latex_a_typst(&clean_latex);

    let font_family = font.info().family.clone();

    let margin = if inline {
        "0.5pt"
    } else {
        "0pt"
    };

    let typst_code = format!(
        r#"
        #set page(width: auto, height: auto, margin: {}, fill: none)
        #set text(font: "{}", size: {}pt, fill: rgb("{}"))
        #show math.equation: set text(font: "{}", fill: rgb("{}"))
        $ {} $
        "#,
        margin, font_family, font_size, color_hex, font_family, color_hex, typst_math
    );

    let world = MinimalWorld::new(typst_code);

    match typst::compile(&world).output {
        Ok(document) => {
            if document.pages.is_empty() {
                return Err("No se generaron páginas".to_string());
            }
            Ok(typst_svg::svg(&document.pages[0]))
        }
        Err(errors) => {
            let mut msg = String::new();
            for err in errors {
                msg.push_str(&err.message);
                msg.push('\n');
            }
            Err(msg)
        }
    }
}
