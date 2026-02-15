use iced::Color;

pub struct AppColors;

impl AppColors {
    pub const BACKGROUND: Color = Color::from_rgb(0.12, 0.12, 0.14);
    pub const TAB_BAR_BG: Color = Color::from_rgb(0.15, 0.15, 0.18);
    pub const TAB_ACTIVE_BG: Color = Color::from_rgb(0.22, 0.22, 0.26);
    pub const TAB_INACTIVE_BG: Color = Color::from_rgb(0.15, 0.15, 0.18);
    pub const STATUS_BAR_BG: Color = Color::from_rgb(0.0, 0.47, 0.84);
    pub const TEXT: Color = Color::from_rgb(0.85, 0.85, 0.85);
    pub const TEXT_DIM: Color = Color::from_rgb(0.55, 0.55, 0.55);
    pub const ACCENT: Color = Color::from_rgb(0.0, 0.47, 0.84);
    pub const CLOSE_HOVER: Color = Color::from_rgb(0.9, 0.2, 0.2);
    pub const BORDER: Color = Color::from_rgb(0.3, 0.3, 0.35);
}
