use iced::widget::{button, checkbox, column, container, row, text, text_input, Space};
use iced::{Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::AppColors;

const DIALOG_BG: iced::Color = iced::Color::from_rgb(0.18, 0.18, 0.22);
const BUTTON_BG: iced::Color = iced::Color::from_rgb(0.25, 0.25, 0.30);
const FIELD_LABEL_SIZE: u16 = 13;

fn action_button<'a>(label: &'a str, msg: Message) -> Element<'a, Message> {
    button(text(label).size(13))
        .on_press(msg)
        .padding([4, 16])
        .style(|_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => AppColors::ACCENT,
                _ => BUTTON_BG,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: AppColors::TEXT,
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

fn close_button<'a>(label: &'a str, msg: Message) -> Element<'a, Message> {
    button(text(label).size(13))
        .on_press(msg)
        .padding([4, 12])
        .style(|_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => {
                    iced::Color::from_rgb(0.30, 0.30, 0.35)
                }
                _ => BUTTON_BG,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: AppColors::TEXT,
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

fn setting_row<'a>(label: &'a str, control: Element<'a, Message>) -> Element<'a, Message> {
    row![
        container(text(label).size(FIELD_LABEL_SIZE).color(AppColors::TEXT))
            .width(Length::Fixed(140.0)),
        control,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn view_preferences_dialog<'a>(state: &NotepadIced) -> Element<'a, Message> {
    let title = text("Preferences").size(18).color(AppColors::TEXT);

    let close_x = button(text("✕").size(14))
        .on_press(Message::CancelPreferences)
        .padding([2, 6])
        .style(|_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => AppColors::CLOSE_HOVER,
                _ => iced::Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: AppColors::TEXT,
                border: iced::Border::default(),
                ..Default::default()
            }
        });

    let header = row![
        title,
        Space::with_width(Length::Fill),
        close_x,
    ]
    .align_y(iced::Alignment::Center);

    // Font size: -/+ buttons with display
    let font_size_control = row![
        button(text("−").size(14))
            .on_press(Message::PrefFontSizeDecrease)
            .padding([2, 8])
            .style(|_t: &Theme, s| {
                let bg = match s {
                    button::Status::Hovered | button::Status::Pressed => AppColors::ACCENT,
                    _ => BUTTON_BG,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: AppColors::TEXT,
                    border: iced::Border { radius: 3.0.into(), ..Default::default() },
                    ..Default::default()
                }
            }),
        text(format!("{:.0}", state.pref_font_size))
            .size(13)
            .color(AppColors::TEXT),
        button(text("+").size(14))
            .on_press(Message::PrefFontSizeIncrease)
            .padding([2, 8])
            .style(|_t: &Theme, s| {
                let bg = match s {
                    button::Status::Hovered | button::Status::Pressed => AppColors::ACCENT,
                    _ => BUTTON_BG,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: AppColors::TEXT,
                    border: iced::Border { radius: 3.0.into(), ..Default::default() },
                    ..Default::default()
                }
            }),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Tab size input
    let tab_size_control = text_input("4", &state.pref_tab_size)
        .on_input(Message::PrefTabSizeChanged)
        .size(13)
        .width(Length::Fixed(60.0));

    // Theme toggle
    let theme_label = if state.pref_theme == "light" { "Light" } else { "Dark" };
    let theme_control = button(text(theme_label).size(13))
        .on_press(Message::PrefToggleTheme)
        .padding([4, 16])
        .style(|_t: &Theme, s| {
            let bg = match s {
                button::Status::Hovered | button::Status::Pressed => AppColors::ACCENT,
                _ => BUTTON_BG,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: AppColors::TEXT,
                border: iced::Border { radius: 3.0.into(), ..Default::default() },
                ..Default::default()
            }
        });

    // Checkboxes
    let auto_save = checkbox("Auto Save", state.pref_auto_save)
        .on_toggle(Message::PrefToggleAutoSave)
        .text_size(FIELD_LABEL_SIZE)
        .size(16);

    let word_wrap = checkbox("Word Wrap", state.pref_word_wrap)
        .on_toggle(Message::PrefToggleWordWrap)
        .text_size(FIELD_LABEL_SIZE)
        .size(16);

    let line_numbers = checkbox("Show Line Numbers", state.pref_show_line_numbers)
        .on_toggle(Message::PrefToggleLineNumbers)
        .text_size(FIELD_LABEL_SIZE)
        .size(16);

    let whitespace = checkbox("Show Whitespace", state.pref_show_whitespace)
        .on_toggle(Message::PrefToggleWhitespace)
        .text_size(FIELD_LABEL_SIZE)
        .size(16);

    // Buttons
    let buttons = row![
        action_button("Save", Message::SavePreferences),
        close_button("Cancel", Message::CancelPreferences),
    ]
    .spacing(8);

    let content = column![
        header,
        Space::with_height(8),
        setting_row("Font Size", font_size_control.into()),
        setting_row("Tab Size", tab_size_control.into()),
        setting_row("Theme", theme_control.into()),
        Space::with_height(4),
        auto_save,
        word_wrap,
        line_numbers,
        whitespace,
        Space::with_height(8),
        buttons,
    ]
    .spacing(6)
    .padding(16)
    .width(Length::Fixed(360.0));

    container(content)
        .style(|_theme: &Theme| container::Style {
            background: Some(iced::Background::Color(DIALOG_BG)),
            border: iced::Border {
                color: iced::Color::from_rgb(0.30, 0.30, 0.35),
                width: 1.0,
                radius: 4.0.into(),
            },
            shadow: iced::Shadow {
                color: iced::Color::from_rgba(0.0, 0.0, 0.0, 0.5),
                offset: iced::Vector::new(0.0, 4.0),
                blur_radius: 16.0,
            },
            ..Default::default()
        })
        .into()
}
