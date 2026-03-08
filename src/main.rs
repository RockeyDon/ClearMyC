#![cfg_attr(not(test), windows_subsystem = "windows")]

mod app;
mod data;
mod file_ops;
mod i18n;
mod llm;
mod ui;

use app::App;
use iced::Font;

/// Load the Microsoft YaHei font from the Windows Fonts directory.
/// Falls back gracefully if the font file is not found.
fn load_system_cjk_font() -> Option<Vec<u8>> {
    let font_path = r"C:\Windows\Fonts\msyh.ttc";
    std::fs::read(font_path).ok()
}

fn main() -> iced::Result {
    let mut app = iced::application(App::new, App::update, App::view)
        .title("ClearMyC")
        .theme(|state: &App| Some(state.theme()))
        .window_size((800.0, 600.0));

    if let Some(font_data) = load_system_cjk_font() {
        app = app
            .font(font_data)
            .default_font(Font::with_name("Microsoft YaHei"));
    }

    app.run()
}
