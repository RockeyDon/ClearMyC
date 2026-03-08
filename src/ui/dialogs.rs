use crate::app::{App, Message};
use crate::i18n::Texts;
use crate::ui::style;

use iced::widget::{button, column, container, row, text, Space};
use iced::{alignment, Color, Element, Font, Length};

pub fn view_delete_confirm(t: &'static Texts, tc: Color, mc: Color) -> Element<'static, Message> {
    let icon = text(t.delete_confirm_icon)
        .font(Font::with_name("Segoe UI Emoji"))
        .size(36)
        .align_x(alignment::Horizontal::Center);

    let title = text(t.delete_confirm_title)
        .size(18)
        .color(tc);

    let msg = text(t.delete_confirm_msg)
        .size(13)
        .color(mc);

    let yes_btn = button(
        text(t.btn_yes_delete)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::ConfirmDelete)
    .padding([10, 28])
    .style(style::btn_danger);

    let no_btn = button(
        text(t.btn_cancel)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::CancelDelete)
    .padding([10, 28])
    .style(style::btn_outlined);

    let dialog = container(
        column![
            icon,
            title,
            msg,
            Space::new().height(8),
            row![yes_btn, no_btn]
                .spacing(12)
                .align_y(alignment::Vertical::Center),
        ]
        .spacing(12)
        .padding([32, 40])
        .align_x(alignment::Horizontal::Center)
        .max_width(420),
    )
    .style(style::dialog_card);

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(style::overlay)
        .into()
}

pub fn view_permanent_confirm(t: &'static Texts, _tc: Color, mc: Color) -> Element<'static, Message> {
    let icon = text(t.perm_confirm_icon)
        .font(Font::with_name("Segoe UI Emoji"))
        .size(36)
        .align_x(alignment::Horizontal::Center);

    let title = text(t.perm_confirm_title)
        .size(18)
        .color(style::DANGER);

    let msg = text(t.perm_confirm_msg)
        .size(13)
        .color(mc);

    let yes_btn = button(
        text(t.btn_yes_perm_delete)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::ConfirmPermanentDelete)
    .padding([10, 28])
    .style(style::btn_danger);

    let no_btn = button(
        text(t.btn_cancel)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::CancelPermanentDelete)
    .padding([10, 28])
    .style(style::btn_outlined);

    let dialog = container(
        column![
            icon,
            title,
            msg,
            Space::new().height(8),
            row![yes_btn, no_btn]
                .spacing(12)
                .align_y(alignment::Vertical::Center),
        ]
        .spacing(12)
        .padding([32, 40])
        .align_x(alignment::Horizontal::Center)
        .max_width(420),
    )
    .style(style::dialog_card);

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(style::overlay)
        .into()
}

pub fn view_connect_dialog(app: &App) -> Element<'_, Message> {
    let theme = app.theme();
    let tc = style::text_color(&theme);
    let t = app.t();

    let msg = text(&app.connect_dialog_text)
        .size(14)
        .color(tc);

    let close_btn = button(
        text(t.btn_close)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::CloseConnectDialog)
    .padding([10, 28])
    .style(style::btn_outlined);

    let dialog = container(
        column![
            msg,
            Space::new().height(12),
            close_btn,
        ]
        .spacing(8)
        .padding([32, 40])
        .align_x(alignment::Horizontal::Center)
        .max_width(420),
    )
    .style(style::dialog_card);

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(style::overlay)
        .into()
}

pub fn view_auto_discover_dialog(app: &App) -> Element<'_, Message> {
    let theme = app.theme();
    let tc = style::text_color(&theme);
    let t = app.t();

    let msg = text(&app.auto_discover_dialog_text)
        .size(14)
        .color(tc);

    let close_btn = button(
        text(t.btn_close)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::CloseAutoDiscoverDialog)
    .padding([10, 28])
    .style(style::btn_outlined);

    let dialog = container(
        column![
            msg,
            Space::new().height(12),
            close_btn,
        ]
        .spacing(8)
        .padding([32, 40])
        .align_x(alignment::Horizontal::Center)
        .max_width(420),
    )
    .style(style::dialog_card);

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(style::overlay)
        .into()
}

pub fn view_clear_confirm(app: &App) -> Element<'_, Message> {
    let theme = app.theme();
    let tc = style::text_color(&theme);
    let mc = style::text_muted_color(&theme);
    let t = app.t();

    let icon = text("\u{1F5D1}")
        .font(Font::with_name("Segoe UI Emoji"))
        .size(36)
        .align_x(alignment::Horizontal::Center);

    let title = text(t.clear_confirm_title)
        .size(18)
        .color(tc);

    let msg = text(t.clear_confirm_msg)
        .size(13)
        .color(mc);

    let yes_btn = button(
        text(t.btn_yes_clear)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::ConfirmClearFolders)
    .padding([10, 28])
    .style(style::btn_danger);

    let no_btn = button(
        text(t.btn_cancel)
            .size(14)
            .align_x(alignment::Horizontal::Center),
    )
    .on_press(Message::CancelClearFolders)
    .padding([10, 28])
    .style(style::btn_outlined);

    let dialog = container(
        column![
            icon,
            title,
            msg,
            Space::new().height(8),
            row![yes_btn, no_btn]
                .spacing(12)
                .align_y(alignment::Vertical::Center),
        ]
        .spacing(12)
        .padding([32, 40])
        .align_x(alignment::Horizontal::Center)
        .max_width(420),
    )
    .style(style::dialog_card);

    container(dialog)
        .width(Length::Fill)
        .height(Length::Fill)
        .align_x(alignment::Horizontal::Center)
        .align_y(alignment::Vertical::Center)
        .style(style::overlay)
        .into()
}
