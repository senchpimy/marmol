fn main() -> Result<(), eframe::Error> {
    #[cfg(not(target_os = "android"))]
    let _ = env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info"))
        .filter_module("swc_ecma_codegen", log::LevelFilter::Off)
        .filter_module("swc_ecma_transforms_base", log::LevelFilter::Off)
        .filter_module("swc", log::LevelFilter::Off)
        .filter_module("swc_common", log::LevelFilter::Off)
        .filter_module("swc_ecma_parser", log::LevelFilter::Off)
        .filter_module("tracing", log::LevelFilter::Off)
        .filter_module("wgpu_core", log::LevelFilter::Warn)
        .filter_module("wgpu_hal", log::LevelFilter::Warn)
        .filter_module("naga", log::LevelFilter::Warn)
        .try_init();

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