fn main() -> Result<(), eframe::Error> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default().with_inner_size([1280.0, 720.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Marmol",
        options,
        Box::new(|cc| {
            egui_extras::install_image_loaders(&cc.egui_ctx);
            Ok(Box::new(marmol::Marmol::new(cc)))
        }),
    )
}