use notepadppp::ui::NotepadApp;

fn main() -> eframe::Result<()> {
    env_logger::init();
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([1200.0, 800.0])
            .with_title("Notepad+++"),
        ..Default::default()
    };
    eframe::run_native(
        "Notepad+++",
        options,
        Box::new(|cc| Ok(Box::new(NotepadApp::new(cc)))),
    )
}
