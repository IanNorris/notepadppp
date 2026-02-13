use notepadppp::tools::markdown_viewer;

// Since render_markdown requires an egui::Ui (which needs a full egui context),
// we test the module compiles and the public API is accessible.
// Full rendering tests require an egui test harness.

#[test]
fn test_markdown_module_compiles() {
    // Verify the module and function exist and are accessible
    let _f: fn(&mut egui::Ui, &str) = markdown_viewer::render_markdown;
}
