use iced::widget::{button, checkbox, column, container, pick_list, row, scrollable, text, text_input, Space};
use iced::{Element, Length, Theme};

use super::app::{Message, NotepadIced};
use super::theme::{AppColors, AppTheme};

const FIELD_LABEL_SIZE: u16 = 13;

fn action_button<'a>(label: &'a str, msg: Message, theme: &AppTheme) -> Element<'a, Message> {
    let t_text = theme.text;
    let t_accent = theme.accent;
    let t_button_bg = theme.button_bg;
    button(text(label).size(13))
        .on_press(msg)
        .padding([4, 16])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => t_accent,
                _ => t_button_bg,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: t_text,
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

fn close_button<'a>(label: &'a str, msg: Message, theme: &AppTheme) -> Element<'a, Message> {
    let t_text = theme.text;
    let t_button_bg = theme.button_bg;
    let t_menu_hover = theme.menu_hover;
    button(text(label).size(13))
        .on_press(msg)
        .padding([4, 12])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => t_menu_hover,
                _ => t_button_bg,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: t_text,
                border: iced::Border {
                    radius: 3.0.into(),
                    ..Default::default()
                },
                ..Default::default()
            }
        })
        .into()
}

fn setting_row<'a>(label: &'a str, control: Element<'a, Message>, theme: &AppTheme) -> Element<'a, Message> {
    row![
        container(text(label).size(FIELD_LABEL_SIZE).color(theme.text))
            .width(Length::Fixed(140.0)),
        control,
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center)
    .into()
}

pub fn view_preferences_dialog<'a>(state: &NotepadIced, theme: &AppTheme) -> Element<'a, Message> {
    let t_text = theme.text;
    let t_accent = theme.accent;
    let t_button_bg = theme.button_bg;
    let t_close_hover = theme.close_hover;
    let t_dialog_bg = theme.dialog_bg;
    let t_border = theme.border;

    let title = text("Preferences").size(18).color(t_text);

    let close_x = button(text("x").size(14))
        .on_press(Message::CancelPreferences)
        .padding([2, 6])
        .style(move |_theme: &Theme, status| {
            let bg = match status {
                button::Status::Hovered | button::Status::Pressed => t_close_hover,
                _ => iced::Color::TRANSPARENT,
            };
            button::Style {
                background: Some(iced::Background::Color(bg)),
                text_color: t_text,
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
        button(text("-").size(14))
            .on_press(Message::PrefFontSizeDecrease)
            .padding([2, 8])
            .style(move |_t: &Theme, s| {
                let bg = match s {
                    button::Status::Hovered | button::Status::Pressed => t_accent,
                    _ => t_button_bg,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: t_text,
                    border: iced::Border { radius: 3.0.into(), ..Default::default() },
                    ..Default::default()
                }
            }),
        text(format!("{:.0}", state.pref_font_size))
            .size(13)
            .color(t_text),
        button(text("+").size(14))
            .on_press(Message::PrefFontSizeIncrease)
            .padding([2, 8])
            .style(move |_t: &Theme, s| {
                let bg = match s {
                    button::Status::Hovered | button::Status::Pressed => t_accent,
                    _ => t_button_bg,
                };
                button::Style {
                    background: Some(iced::Background::Color(bg)),
                    text_color: t_text,
                    border: iced::Border { radius: 3.0.into(), ..Default::default() },
                    ..Default::default()
                }
            }),
    ]
    .spacing(8)
    .align_y(iced::Alignment::Center);

    // Font family selector
    let font_families: Vec<String> = vec![
        "monospace".into(),
        "Cascadia Code".into(),
        "Consolas".into(),
        "JetBrains Mono".into(),
        "Fira Code".into(),
        "Source Code Pro".into(),
        "Courier New".into(),
    ];
    let selected_font = Some(state.pref_font_family.clone());
    let font_picker: Element<'a, Message> = pick_list(
        font_families,
        selected_font,
        Message::PrefFontFamilyChanged,
    )
    .text_size(13)
    .width(Length::Fixed(200.0))
    .into();

    // Tab size input
    let tab_size_control = text_input("4", &state.pref_tab_size)
        .on_input(Message::PrefTabSizeChanged)
        .size(13)
        .width(Length::Fixed(60.0));

    // Line spacing input
    let line_spacing_control = text_input("1.3", &state.pref_line_spacing)
        .on_input(Message::PrefLineSpacingChanged)
        .size(13)
        .width(Length::Fixed(60.0));

    // Theme selector: pick_list dropdown
    let available = AppColors::available_themes();
    let theme_names: Vec<String> = available.iter().map(|t| t.name.clone()).collect();
    let selected_theme = Some(state.pref_theme.clone());
    let theme_picker: Element<'a, Message> = pick_list(
        theme_names,
        selected_theme,
        Message::PrefSetTheme,
    )
    .text_size(13)
    .width(Length::Fixed(200.0))
    .into();

    // Theme import/export buttons
    let theme_buttons = row![
        action_button("Import Theme...", Message::ImportTheme, theme),
        action_button("Export Theme...", Message::ExportTheme, theme),
    ]
    .spacing(8);

    // Color customization inputs
    let color_bg_input = text_input("#RRGGBB", &state.pref_color_bg)
        .on_input(Message::PrefColorBgChanged)
        .size(13)
        .width(Length::Fixed(100.0));

    let color_fg_input = text_input("#RRGGBB", &state.pref_color_fg)
        .on_input(Message::PrefColorFgChanged)
        .size(13)
        .width(Length::Fixed(100.0));

    let color_sel_input = text_input("#RRGGBB", &state.pref_color_sel)
        .on_input(Message::PrefColorSelChanged)
        .size(13)
        .width(Length::Fixed(100.0));

    let color_caret_input = text_input("#RRGGBB", &state.pref_color_caret)
        .on_input(Message::PrefColorCaretChanged)
        .size(13)
        .width(Length::Fixed(100.0));

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
        action_button("Save", Message::SavePreferences, theme),
        close_button("Cancel", Message::CancelPreferences, theme),
    ]
    .spacing(8);

    let content = column![
        header,
        Space::with_height(8),
        text("Editor").size(14).color(t_accent),
        setting_row("Font Family", font_picker, theme),
        setting_row("Font Size", font_size_control.into(), theme),
        setting_row("Tab Size", tab_size_control.into(), theme),
        setting_row("Line Spacing", line_spacing_control.into(), theme),
        Space::with_height(4),
        auto_save,
        word_wrap,
        line_numbers,
        whitespace,
        Space::with_height(8),
        text("Appearance").size(14).color(t_accent),
        setting_row("Theme", theme_picker, theme),
        theme_buttons,
        Space::with_height(4),
        text("Color Overrides (hex)").size(14).color(t_accent),
        setting_row("Background", color_bg_input.into(), theme),
        setting_row("Text", color_fg_input.into(), theme),
        setting_row("Selection", color_sel_input.into(), theme),
        setting_row("Caret", color_caret_input.into(), theme),
        Space::with_height(8),
        buttons,
    ]
    .spacing(6)
    .padding(16)
    .width(Length::Fixed(480.0));

    container(
        scrollable(content).height(Length::Fill)
    )
    .max_height(600)
    .style(move |_theme: &Theme| container::Style {
        background: Some(iced::Background::Color(t_dialog_bg)),
        border: iced::Border {
            color: t_border,
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
