use egui::{self, Ui};

/// Orientation of the split view.
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum SplitOrientation {
    /// Side by side (left/right)
    Horizontal,
    /// Top and bottom
    Vertical,
}

/// Split view state for showing two editor panels.
pub struct SplitView {
    pub enabled: bool,
    pub orientation: SplitOrientation,
    /// Position of divider, 0.0 to 1.0
    pub ratio: f32,
    /// Which tab index is shown in the second panel
    pub second_tab_index: Option<usize>,
}

impl Default for SplitView {
    fn default() -> Self {
        Self {
            enabled: false,
            orientation: SplitOrientation::Horizontal,
            ratio: 0.5,
            second_tab_index: None,
        }
    }
}

impl SplitView {
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable split view with the given orientation.
    pub fn enable(&mut self, orientation: SplitOrientation) {
        self.enabled = true;
        self.orientation = orientation;
        self.ratio = 0.5;
    }

    /// Disable split view.
    pub fn disable(&mut self) {
        self.enabled = false;
        self.second_tab_index = None;
    }

    /// Render the split view with a draggable divider.
    /// Calls `render_panel` for each side with the panel index (0 = primary, 1 = secondary).
    pub fn render(
        &mut self,
        ui: &mut Ui,
        mut render_panel: impl FnMut(&mut Ui, usize),
    ) {
        if !self.enabled {
            render_panel(ui, 0);
            return;
        }

        let available = ui.available_rect_before_wrap();
        let divider_width = 6.0;

        match self.orientation {
            SplitOrientation::Horizontal => {
                let total_width = available.width();
                let left_width = (total_width * self.ratio - divider_width / 2.0).max(50.0);
                let right_width = (total_width - left_width - divider_width).max(50.0);

                ui.horizontal(|ui| {
                    ui.allocate_ui(egui::vec2(left_width, available.height()), |ui| {
                        render_panel(ui, 0);
                    });

                    // Draggable divider
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(divider_width, available.height()),
                        egui::Sense::drag(),
                    );
                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        if response.hovered() || response.dragged() {
                            egui::Color32::from_gray(120)
                        } else {
                            egui::Color32::from_gray(80)
                        },
                    );
                    if response.dragged() {
                        let delta = response.drag_delta().x;
                        self.ratio = ((self.ratio * total_width + delta) / total_width)
                            .clamp(0.1, 0.9);
                    }

                    ui.allocate_ui(egui::vec2(right_width, available.height()), |ui| {
                        render_panel(ui, 1);
                    });
                });
            }
            SplitOrientation::Vertical => {
                let total_height = available.height();
                let top_height = (total_height * self.ratio - divider_width / 2.0).max(50.0);
                let bottom_height = (total_height - top_height - divider_width).max(50.0);

                ui.vertical(|ui| {
                    ui.allocate_ui(egui::vec2(available.width(), top_height), |ui| {
                        render_panel(ui, 0);
                    });

                    // Draggable divider
                    let (rect, response) = ui.allocate_exact_size(
                        egui::vec2(available.width(), divider_width),
                        egui::Sense::drag(),
                    );
                    ui.painter().rect_filled(
                        rect,
                        0.0,
                        if response.hovered() || response.dragged() {
                            egui::Color32::from_gray(120)
                        } else {
                            egui::Color32::from_gray(80)
                        },
                    );
                    if response.dragged() {
                        let delta = response.drag_delta().y;
                        self.ratio = ((self.ratio * total_height + delta) / total_height)
                            .clamp(0.1, 0.9);
                    }

                    ui.allocate_ui(egui::vec2(available.width(), bottom_height), |ui| {
                        render_panel(ui, 1);
                    });
                });
            }
        }
    }
}
