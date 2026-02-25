use crate::egui_commonmark_backend::misc::CommonMarkCache;
use egui::{Color32, RichText, Ui};
use rust_embed::RustEmbed;
use std::path::PathBuf;
use time::OffsetDateTime;
use typst::diag::{FileError, FileResult};
use typst::foundations::{Bytes, Datetime};
use typst::syntax::{FileId, Source};
use typst::text::{Font, FontBook};
use typst::utils::LazyHash;
use typst::{Library, World};

#[derive(RustEmbed)]
#[folder = "assets/fonts/"]
struct Asset;

struct MinimalWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    source: Source,
    time: time::OffsetDateTime,
}

impl MinimalWorld {
    fn new(source_text: String, font_data: Vec<u8>) -> Self {
        let font = Font::new(Bytes::from(font_data), 0).expect("Fuente inválida");
        let fonts = vec![font];
        let book = FontBook::from_fonts(&fonts);
        let source = Source::detached(source_text);

        Self {
            library: LazyHash::new(Library::default()),
            book: LazyHash::new(book),
            fonts,
            source,
            time: OffsetDateTime::now_utc(),
        }
    }
}

impl World for MinimalWorld {
    fn library(&self) -> &LazyHash<Library> {
        &self.library
    }
    fn book(&self) -> &LazyHash<FontBook> {
        &self.book
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
        self.fonts.get(index).cloned()
    }
    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Some(Datetime::Date(self.time.date()))
    }
}

pub fn render(ui: &mut Ui, cache: &mut CommonMarkCache, latex: &str, inline: bool) {
    let text_color = ui.visuals().text_color();
    let color_hex = format!(
        "#{:02x}{:02x}{:02x}",
        text_color.r(),
        text_color.g(),
        text_color.b()
    );
    let cache_key = format!("{}_{}_{}", latex, inline, color_hex);

    let svg_content = if let Some(svg) = cache.latex_cache.get(&cache_key) {
        Some(svg.clone())
    } else {
        match render_to_svg(latex, inline, &color_hex) {
            Ok(svg) => {
                cache.latex_cache.insert(cache_key.clone(), svg.clone());
                Some(svg)
            }
            Err(err) => {
                ui.label(RichText::new(format!("LaTeX error: {}", err)).color(Color32::RED));
                None
            }
        }
    };

    if let Some(svg) = svg_content {
        let uri = format!("bytes://latex_{}.svg", egui::Id::new(&cache_key).value());
        let mut img = egui::Image::from_bytes(uri, svg.as_bytes().to_vec());

        if inline || (latex.len() < 50 && !latex.contains('\n')) {
            img = img.fit_to_original_size(1.3);

            ui.vertical(|ui| {
                ui.add_space(1.0);
                ui.add(img);
            });
        } else {
            img = img.fit_to_original_size(2.0);
            ui.vertical_centered(|ui| {
                ui.add(img.max_width(ui.available_width()));
            });
        }
    }
}

fn render_to_svg(latex_input: &str, inline: bool, color_hex: &str) -> Result<String, String> {
    let font_file = Asset::get("NotoSansMath-Regular.ttf")
        .ok_or("No se encontró la fuente NotoSansMath en el binario")?;
    let font_data = font_file.data.to_vec();

    // Convertir LaTeX a Typst Math
    let clean_latex = latex_input.replace('\n', "");
    let typst_math =
        tex2typst_rs::tex2typst(&clean_latex).unwrap_or_else(|_| latex_input.to_string());

    let font = Font::new(Bytes::from(font_data.clone()), 0).ok_or("Fuente inválida")?;
    let font_family = font.info().family.clone();

    let (margin, size) = if inline {
        ("0.5pt", "10pt")
    } else {
        ("0pt", "10pt")
    };

    let typst_code = format!(
        r#"
        #set page(width: auto, height: auto, margin: {}, fill: none)
        #set text(font: "{}", size: {}, fill: rgb("{}"))
        #show math.equation: set text(font: "{}", fill: rgb("{}"))
        $ {} $
        "#,
        margin, font_family, size, color_hex, font_family, color_hex, typst_math
    );

    let world = MinimalWorld::new(typst_code, font_data);

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
