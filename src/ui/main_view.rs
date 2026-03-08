use crate::app::{App, Message};
use crate::file_ops::format_size;
use crate::ui::style;

use iced::widget::{
    button, checkbox, column, container, row, scrollable, text, Column, Space,
};
use iced::{alignment, Element, Font, Length};

pub fn view(app: &App) -> Element<'_, Message> {
    let theme = app.theme();
    let tc = style::text_color(&theme);
    let mc = style::text_muted_color(&theme);
    let t = app.t();

    // ── Header ───────────────────────────────────────────────────────
    let title = text(t.app_title)
        .size(22)
        .font(Font::with_name("Segoe UI"))
        .color(tc);

    let config_btn = button(
        text("\u{2699}")
            .font(Font::with_name("Segoe UI Emoji"))
            .size(20)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::OpenConfig)
    .padding([6, 12])
    .style(style::btn_ghost);

    let header = row![Space::new().width(Length::Fill), title, Space::new().width(Length::Fill), config_btn]
        .align_y(alignment::Vertical::Center)
        .padding([16, 24]);

    // ── Table header ─────────────────────────────────────────────────
    let table_header = container(
        row![
            text("").width(36),
            container(text(t.col_path).size(13).color(mc))
                .width(Length::FillPortion(3)),
            container(text(t.col_description).size(13).color(mc))
                .width(Length::FillPortion(2)),
            text(t.col_size)
                .size(13)
                .color(mc)
                .width(90),
            text(t.col_status)
                .size(13)
                .color(mc)
                .width(110),
        ]
        .spacing(10)
        .padding([10, 20]),
    )
    .style(style::table_header);

    // ── Table rows ───────────────────────────────────────────────────
    let rows: Vec<Element<Message>> = app
        .data
        .folders
        .iter()
        .enumerate()
        .map(|(i, folder)| {
            let size_text = match app.folder_sizes.get(i) {
                Some(Some(size)) => format_size(*size),
                _ => {
                    if app.scanning {
                        t.scanning_text.to_string()
                    } else {
                        "-".to_string()
                    }
                }
            };

            let status_widget: Element<Message> = match app.folder_statuses.get(i) {
                Some(Some(0)) => text(t.status_succeed)
                    .font(Font::with_name("Segoe UI Emoji"))
                    .size(13)
                    .color(style::SUCCESS_GREEN)
                    .into(),
                Some(Some(1)) => text(t.status_failed)
                    .font(Font::with_name("Segoe UI Emoji"))
                    .size(13)
                    .color(style::DANGER)
                    .into(),
                Some(Some(2)) => text(t.status_skipped)
                    .font(Font::with_name("Segoe UI Emoji"))
                    .size(13)
                    .color(style::SUCCESS_GREEN)
                    .into(),
                _ => text("").into(),
            };

            let row_style = if i % 2 == 0 {
                style::row_even
            } else {
                style::row_odd
            };

            container(
                row![
                    checkbox(folder.selected)
                        .on_toggle(move |_| Message::ToggleSelected(i))
                        .width(36),
                    container(
                        text(&folder.path)
                            .size(13)
                            .color(tc),
                    )
                    .width(Length::FillPortion(3))
                    .clip(true),
                    container(
                        text(&folder.description)
                            .size(13)
                            .color(mc),
                    )
                    .width(Length::FillPortion(2))
                    .clip(true),
                    text(size_text).size(13).color(tc).width(90),
                    container(status_widget).width(110),
                ]
                .spacing(10)
                .padding([8, 20])
                .align_y(alignment::Vertical::Center),
            )
            .style(row_style)
            .into()
        })
        .collect();

    let table_body = scrollable(Column::with_children(rows));

    // ── Bottom action bar ────────────────────────────────────────────
    let refresh_btn = button(
        text(t.btn_refresh)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::Refresh)
    .padding([10, 24])
    .style(style::btn_primary);

    let delete_btn = button(
        text(t.btn_delete)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::DeleteClicked)
    .padding([10, 20])
    .style(style::btn_danger);

    let perm_delete_btn = button(
        text(t.btn_permanently_delete)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::PermanentDeleteClicked)
    .padding([10, 20])
    .style(style::btn_danger);

    let bottom_bar = container(
        row![refresh_btn, Space::new().width(Length::Fill), delete_btn, perm_delete_btn]
            .spacing(12)
            .padding([14, 24])
            .align_y(alignment::Vertical::Center),
    )
    .style(style::bottom_bar);

    // ── Assemble everything inside a card ─────────────────────────────
    let table_card = container(column![table_header, table_body].spacing(0))
        .width(Length::Fill)
        .height(Length::Fill)
        .style(style::card);

    let content = column![header, table_card, bottom_bar].spacing(0);

    container(content)
        .width(Length::Fill)
        .height(Length::Fill)
        .padding(iced::Padding { top: 12.0, right: 16.0, bottom: 16.0, left: 16.0 })
        .style(style::app_background)
        .into()
}
