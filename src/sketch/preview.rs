use egui::{Color32, Ui};
use rust_embed::RustEmbed;
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
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

struct MiniWorld {
    library: LazyHash<Library>,
    book: LazyHash<FontBook>,
    fonts: Vec<Font>,
    source: Source,
    time: OffsetDateTime,
}

impl MiniWorld {
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

impl World for MiniWorld {
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
            Err(FileError::NotFound(std::path::PathBuf::new()))
        }
    }

    fn file(&self, _id: FileId) -> FileResult<Bytes> {
        Err(FileError::NotFound(std::path::PathBuf::new()))
    }

    fn font(&self, index: usize) -> Option<Font> {
        self.fonts.get(index).cloned()
    }

    fn today(&self, _offset: Option<i64>) -> Option<Datetime> {
        Some(Datetime::Date(self.time.date()))
    }
}

pub fn latex_a_svg(latex: &str, color: Color32) -> Result<String, String> {
    let font_file =
        Asset::get("NotoSansMath-Regular.ttf").ok_or("No se encontró la fuente NotoSansMath")?;
    let font_data = font_file.data.to_vec();

    let clean = latex.replace('\n', " ");
    let typst_math = tex2typst_rs::tex2typst(&clean).unwrap_or_else(|_| clean.clone());

    let hex = format!("#{:02x}{:02x}{:02x}", color.r(), color.g(), color.b());
    let font = Font::new(Bytes::from(font_data.clone()), 0).ok_or("Fuente inválida")?;
    let familia = font.info().family.clone();

    let typst_code = format!(
        r#"
        #set page(width: auto, height: auto, margin: 4pt, fill: none)
        #set text(font: "{}", size: 22pt, fill: rgb("{}"))
        #show math.equation: set text(font: "{}", fill: rgb("{}"))
        $ {} $
        "#,
        familia, hex, familia, hex, typst_math
    );

    let world = MiniWorld::new(typst_code, font_data);

    match typst::compile(&world).output {
        Ok(document) => document
            .pages
            .first()
            .map(typst_svg::svg)
            .ok_or_else(|| "El documento no generó páginas".to_string()),
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

pub fn mostrar_preview(
    ui: &mut Ui,
    cache: &mut Option<(String, Result<String, String>)>,
    latex: &str,
) {
    if latex.trim().is_empty() {
        return;
    }

    let cambiar = cache.as_ref().map(|(k, _)| k != latex).unwrap_or(true);

    if cambiar {
        *cache = Some((
            latex.to_string(),
            latex_a_svg(latex, ui.visuals().text_color()),
        ));
    }

    if let Some((_, resultado)) = cache.as_ref() {
        match resultado {
            Ok(svg) => {
                let mut hasher = DefaultHasher::new();
                latex.hash(&mut hasher);
                let uri = format!("bytes://sketch_preview_{:016x}.svg", hasher.finish());
                let img =
                    egui::Image::from_bytes(uri, svg.as_bytes().to_vec()).fit_to_original_size(1.0);
                ui.vertical_centered(|ui| {
                    ui.add(img.max_width(ui.available_width()));
                });
            }
            Err(e) => {
                ui.colored_label(Color32::RED, format!("Error en el preview: {}", e));
            }
        }
    }
}
