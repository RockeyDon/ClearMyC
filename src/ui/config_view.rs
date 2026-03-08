use crate::app::{App, ConfigTab, Message};
use crate::i18n::Locale;
use crate::ui::style;

use iced::widget::{button, column, container, radio, row, rule, scrollable, text, text_input, Column};
use iced::{alignment, Element, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let theme = app.theme();
    let tc = style::text_color(&theme);
    let mc = style::text_muted_color(&theme);
    let t = app.t();

    // ── Sidebar tabs ─────────────────────────────────────────────────
    let folder_tab = button(
        text(t.tab_folders)
            .size(14)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Left),
    )
    .on_press(Message::ConfigTabChanged(ConfigTab::Folder))
    .padding([12, 20])
    .width(Length::Fill)
    .style(if app.config_tab == ConfigTab::Folder {
        style::btn_tab_active as fn(&iced::Theme, button::Status) -> button::Style
    } else {
        style::btn_tab_inactive
    });

    let strategy_tab = button(
        text(t.tab_strategy)
            .size(14)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Left),
    )
    .on_press(Message::ConfigTabChanged(ConfigTab::Strategy))
    .padding([12, 20])
    .width(Length::Fill)
    .style(if app.config_tab == ConfigTab::Strategy {
        style::btn_tab_active as fn(&iced::Theme, button::Status) -> button::Style
    } else {
        style::btn_tab_inactive
    });

    let theme_tab = button(
        text(t.tab_theme)
            .size(14)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Left),
    )
    .on_press(Message::ConfigTabChanged(ConfigTab::Theme))
    .padding([12, 20])
    .width(Length::Fill)
    .style(if app.config_tab == ConfigTab::Theme {
        style::btn_tab_active as fn(&iced::Theme, button::Status) -> button::Style
    } else {
        style::btn_tab_inactive
    });

    let about_tab = button(
        text(t.tab_about)
            .size(14)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Left),
    )
    .on_press(Message::ConfigTabChanged(ConfigTab::About))
    .padding([12, 20])
    .width(Length::Fill)
    .style(if app.config_tab == ConfigTab::About {
        style::btn_tab_active as fn(&iced::Theme, button::Status) -> button::Style
    } else {
        style::btn_tab_inactive
    });

    let sidebar = container(
        column![folder_tab, strategy_tab, theme_tab, about_tab]
        .spacing(4)
        .padding([20, 14]),
    )
    .width(170)
    .height(Length::Fill)
    .style(style::sidebar);

    // ── Right content ────────────────────────────────────────────────
    let right_content: Element<Message> = match app.config_tab {
        ConfigTab::Folder => view_folder_tab(app, tc, mc),
        ConfigTab::Strategy => view_strategy_tab(app, tc, mc),
        ConfigTab::Theme => view_theme_tab(app, tc, mc),
        ConfigTab::About => view_about_tab(app, tc, mc),
    };

    // ── Close button ─────────────────────────────────────────────────
    let close_btn = button(
        text(t.btn_save_and_close)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::CloseConfig)
    .padding([10, 24])
    .style(style::btn_primary);

    let right_panel = column![
        container(right_content)
            .width(Length::Fill)
            .height(Length::Fill)
            .padding([20, 24]),
        container(close_btn)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Right)
            .padding([12, 24]),
    ];

    // ── Assemble inside a card ───────────────────────────────────────
    let layout = container(row![sidebar, right_panel].spacing(0))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::card);

    container(layout)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(16)
        .style(style::app_background)
        .into()
}

fn view_folder_tab<'a>(
    app: &'a App,
    tc: iced::Color,
    mc: iced::Color,
) -> Element<'a, Message> {
    let t = app.t();
    let mut rows = Column::new().spacing(10);

    for (i, folder_row) in app.config_folders.iter().enumerate() {
        let idx_label = container(
            text(format!("{}", i + 1))
                .size(12)
                .color(mc)
                .align_x(alignment::Horizontal::Center),
        )
        .width(24)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center);

        let path_input = text_input(t.folder_path_placeholder, &folder_row.path)
            .on_input(move |val| Message::ConfigFolderPathChanged(i, val))
            .padding([10, 12])
            .size(13)
            .width(Length::FillPortion(1))
            .style(style::input_style);

        let desc_input = text_input(t.folder_desc_placeholder, &folder_row.description)
            .on_input(move |val| Message::ConfigFolderDescChanged(i, val))
            .padding([10, 12])
            .size(13)
            .width(Length::FillPortion(1))
            .style(style::input_style);

        let remove_btn = button(
            text("\u{00D7}") // × symbol
                .size(16)
                .align_x(alignment::Horizontal::Center),
        )
        .on_press(Message::RemoveFolderRow(i))
        .padding([4, 8])
        .style(style::btn_row_delete);

        rows = rows.push(
            row![idx_label, path_input, desc_input, remove_btn]
                .spacing(10)
                .padding(iced::Padding { top: 0.0, right: 12.0, bottom: 0.0, left: 0.0 })
                .align_y(alignment::Vertical::Center),
        );
    }

    // Add Folder and Clear buttons share the original Add Folder button width
    let add_btn = button(
        text(t.btn_add_folder)
            .size(14)
            .color(style::ACCENT)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AddFolderRow)
    .padding([12, 0])
    .width(Length::FillPortion(1))
    .style(style::btn_ghost);

    let clear_btn = button(
        text(t.btn_clear)
            .size(14)
            .color(style::DANGER)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::ClearFoldersClicked)
    .padding([12, 0])
    .width(Length::FillPortion(1))
    .style(style::btn_ghost);

    let add_clear_row = row![add_btn, clear_btn]
        .spacing(12);

    let auto_discover_btn = button(
        text(t.btn_auto_discovery)
            .size(14)
            .color(style::ACCENT)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::AutoDiscoverClicked)
    .padding([12, 0])
    .width(Length::Fill)
    .style(style::btn_ghost);

    column![
        text(t.folder_title).size(16).color(tc),
        text(t.folder_subtitle)
            .size(12)
            .color(mc),
        scrollable(rows).height(Length::Fill),
        add_clear_row,
        auto_discover_btn,
    ]
    .spacing(12)
    .into()
}

fn view_strategy_tab<'a>(
    app: &'a App,
    tc: iced::Color,
    mc: iced::Color,
) -> Element<'a, Message> {
    let t = app.t();

    // ── LLM Configuration section ────────────────────────────────────
    let base_url_row = row![
        text(t.strategy_base_url).size(13).color(mc).width(90),
        text_input("https://api.openai.com/v1", &app.strategy_base_url)
            .on_input(Message::StrategyBaseUrlChanged)
            .padding([10, 12])
            .size(13)
            .width(Length::Fill)
            .style(style::input_style),
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center);

    let model_row = row![
        text(t.strategy_model).size(13).color(mc).width(90),
        text_input("e.g. gpt-4", &app.strategy_model)
            .on_input(Message::StrategyModelChanged)
            .padding([10, 12])
            .size(13)
            .width(Length::Fill)
            .style(style::input_style),
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center);

    let api_key_row = row![
        text(t.strategy_api_key).size(13).color(mc).width(90),
        text_input("sk-xxxx", &app.strategy_api_key)
            .on_input(Message::StrategyApiKeyChanged)
            .padding([10, 12])
            .size(13)
            .width(Length::Fill)
            .secure(true)
            .style(style::input_style),
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center);

    let connect_btn = button(
        text(t.btn_connect)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::ConnectClicked)
    .padding([10, 24])
    .style(style::btn_outlined);

    let llm_section = column![
        base_url_row,
        model_row,
        api_key_row,
        container(connect_btn)
            .width(Length::Fill)
            .align_x(alignment::Horizontal::Right),
    ]
    .spacing(12);

    // ── Strategy rules section ───────────────────────────────────────
    let skip_list_row = row![
        text(t.strategy_skip_list).size(13).color(mc).width(90),
        text_input("C:\\path1;C:\\path2", &app.strategy_skip_list)
            .on_input(Message::StrategySkipListChanged)
            .padding([10, 12])
            .size(13)
            .width(Length::Fill)
            .style(style::input_style),
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center);

    let min_size_row = row![
        text(t.strategy_min_size).size(13).color(mc).width(90),
        text_input("1GB", &app.strategy_min_size)
            .on_input(Message::StrategyMinSizeChanged)
            .padding([10, 12])
            .size(13)
            .width(180)
            .style(style::input_style),
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center);

    let skip_recent_row = row![
        text(t.strategy_skip_recent).size(13).color(mc).width(90),
        text_input("7", &app.strategy_skip_recent)
            .on_input(Message::StrategySkipRecentChanged)
            .padding([10, 12])
            .size(13)
            .width(80)
            .style(style::input_style),
        text(t.strategy_skip_recent_suffix).size(13).color(mc),
    ]
    .spacing(12)
    .align_y(alignment::Vertical::Center);

    let strategy_section = column![
        skip_list_row,
        min_size_row,
        skip_recent_row,
    ]
    .spacing(12);

    column![
        text(t.strategy_llm_title).size(16).color(tc),
        text(t.strategy_llm_description)
            .size(12)
            .color(mc),
        llm_section,
        rule::horizontal(1),
        text(t.strategy_rules_title).size(15).color(tc),
        strategy_section,
    ]
    .spacing(16)
    .into()
}

fn view_theme_tab<'a>(
    app: &'a App,
    tc: iced::Color,
    mc: iced::Color,
) -> Element<'a, Message> {
    let t = app.t();
    let current_lang = app.locale.as_str().to_string();

    let light = radio(t.theme_light, "light", Some(&app.config_theme), |_| {
        Message::ConfigThemeChanged("light".to_string())
    })
    .size(18)
    .spacing(8);

    let dark = radio(t.theme_dark, "dark", Some(&app.config_theme), |_| {
        Message::ConfigThemeChanged("dark".to_string())
    })
    .size(18)
    .spacing(8);

    let system = radio(t.theme_system, "system", Some(&app.config_theme), |_| {
        Message::ConfigThemeChanged("system".to_string())
    })
    .size(18)
    .spacing(8);

    // ── Language selector ────────────────────────────────────────────
    let en = radio(
        Locale::EnUs.display_name(),
        Locale::EnUs.as_str(),
        Some(current_lang.as_str()),
        |_| Message::LanguageChanged(Locale::EnUs.as_str().to_string()),
    )
    .size(18)
    .spacing(8);

    let zh = radio(
        Locale::ZhCn.display_name(),
        Locale::ZhCn.as_str(),
        Some(current_lang.as_str()),
        |_| Message::LanguageChanged(Locale::ZhCn.as_str().to_string()),
    )
    .size(18)
    .spacing(8);

    column![
        text(t.theme_title).size(16).color(tc),
        text(t.theme_description)
            .size(12)
            .color(mc),
        column![light, dark, system].spacing(14).padding([12, 0]),
        rule::horizontal(1),
        text(t.language_label).size(16).color(tc),
        column![en, zh].spacing(14).padding([12, 0]),
    ]
    .spacing(12)
    .into()
}

fn view_about_tab(app: &App, tc: iced::Color, mc: iced::Color) -> Element<'_, Message> {
    let t = app.t();

    column![
        text(t.about_title).size(16).color(tc),
        column![
            text(t.about_description)
                .size(13)
                .color(mc),
            row![
                text(t.about_homepage_label).size(13).color(mc),
                text("https://github.com/RockeyDon/ClearMyC")
                    .size(13)
                    .color(style::ACCENT),
            ]
            .spacing(6),
            row![
                text(t.about_version_label).size(13).color(mc),
                text("0.3.0").size(13).color(tc),
            ]
            .spacing(6),
        ]
        .spacing(8)
        .padding([8, 0]),
    ]
    .spacing(12)
    .into()
}
