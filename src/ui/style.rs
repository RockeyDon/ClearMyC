// ─── Centralized style definitions for the entire application ─────────
//
// Design language: clean, modern, spacious.
//   - Rounded corners everywhere (6–10 px)
//   - Soft shadows instead of hard borders
//   - Accent blue for primary actions
//   - Danger red for destructive actions
//   - Full light / dark theme support via Palette

use iced::border::Radius;
use iced::widget::{button, container, text_input};
use iced::{Background, Border, Color, Shadow, Theme, Vector};

// ─── Dual-mode colour palette ────────────────────────────────────────
// Every colour that changes between light/dark is accessed through a
// `Palette` struct.  Functions that need colours call `palette(theme)`.

#[derive(Debug, Clone, Copy)]
pub struct Palette {
    pub bg:           Color,
    pub surface:      Color,
    pub surface_alt:  Color, // row stripe / sidebar
    pub border:       Color,
    pub text:         Color,
    pub text_muted:   Color,
    pub shadow:       Color,
}

const LIGHT: Palette = Palette {
    bg:          Color::from_rgb(0.976, 0.980, 0.988),  // #F9FAFB
    surface:     Color::from_rgb(1.0, 1.0, 1.0),        // #FFFFFF
    surface_alt: Color::from_rgb(0.953, 0.957, 0.965),  // #F3F4F6
    border:      Color::from_rgb(0.910, 0.918, 0.929),  // #E8EAED
    text:        Color::from_rgb(0.129, 0.145, 0.176),  // #21252D
    text_muted:  Color::from_rgb(0.439, 0.467, 0.529),  // #707787
    shadow:      Color::from_rgba(0.0, 0.0, 0.0, 0.08),
};

const DARK: Palette = Palette {
    bg:          Color::from_rgb(0.098, 0.106, 0.122),  // #191B1F
    surface:     Color::from_rgb(0.149, 0.161, 0.184),  // #26292F
    surface_alt: Color::from_rgb(0.118, 0.129, 0.149),  // #1E2126
    border:      Color::from_rgb(0.220, 0.235, 0.263),  // #383C43
    text:        Color::from_rgb(0.906, 0.914, 0.929),  // #E7E9ED
    text_muted:  Color::from_rgb(0.580, 0.604, 0.651),  // #949AA6
    shadow:      Color::from_rgba(0.0, 0.0, 0.0, 0.25),
};

fn palette(theme: &Theme) -> &'static Palette {
    match theme {
        Theme::Dark => &DARK,
        _ => &LIGHT,
    }
}

// ─── Theme-independent accent colours ────────────────────────────────

pub const ACCENT: Color       = Color::from_rgb(0.231, 0.510, 0.965);  // #3B82F6
pub const ACCENT_HOVER: Color = Color::from_rgb(0.369, 0.608, 1.0);    // #5E9BFF
pub const DANGER: Color       = Color::from_rgb(0.937, 0.267, 0.267);  // #EF4444
pub const DANGER_HOVER: Color = Color::from_rgb(0.960, 0.400, 0.400);  // #F56565
pub const SUCCESS_GREEN: Color= Color::from_rgb(0.133, 0.773, 0.369);  // #22C55E
pub const WHITE: Color        = Color::from_rgb(1.0, 1.0, 1.0);
pub const OVERLAY_BG: Color   = Color::from_rgba(0.0, 0.0, 0.0, 0.35);

// ─── Spacing / sizing constants ──────────────────────────────────────

pub const RADIUS_SM: f32 = 6.0;
pub const RADIUS_MD: f32 = 10.0;
pub const RADIUS_LG: f32 = 14.0;

// ─── Helper: build a Shadow from a palette ───────────────────────────

fn card_shadow(p: &Palette) -> Shadow {
    Shadow {
        color: p.shadow,
        offset: Vector { x: 0.0, y: 4.0 },
        blur_radius: 16.0,
    }
}

fn subtle_shadow(p: &Palette) -> Shadow {
    Shadow {
        color: p.shadow,
        offset: Vector { x: 0.0, y: 2.0 },
        blur_radius: 8.0,
    }
}

// ─── Public helpers for views to get theme-aware text colours ────────

pub fn text_color(theme: &Theme) -> Color {
    palette(theme).text
}

pub fn text_muted_color(theme: &Theme) -> Color {
    palette(theme).text_muted
}

// ═══════════════════════════════════════════════════════════════════════
//  Button styles
// ═══════════════════════════════════════════════════════════════════════

/// Primary action button — filled accent blue, white text, rounded.
pub fn btn_primary(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(ACCENT)),
        text_color: WHITE,
        border: Border {
            radius: RADIUS_SM.into(),
            ..Border::default()
        },
        shadow: subtle_shadow(&LIGHT),
        snap: true,
    };
    match status {
        button::Status::Active | button::Status::Pressed => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(ACCENT_HOVER)),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.6, 0.6, 0.6))),
            text_color: Color { a: 0.5, ..WHITE },
            ..base
        },
    }
}

/// Danger / destructive button — filled red.
pub fn btn_danger(_theme: &Theme, status: button::Status) -> button::Style {
    let base = button::Style {
        background: Some(Background::Color(DANGER)),
        text_color: WHITE,
        border: Border {
            radius: RADIUS_SM.into(),
            ..Border::default()
        },
        shadow: subtle_shadow(&LIGHT),
        snap: true,
    };
    match status {
        button::Status::Active | button::Status::Pressed => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(DANGER_HOVER)),
            ..base
        },
        button::Status::Disabled => button::Style {
            background: Some(Background::Color(Color::from_rgb(0.6, 0.6, 0.6))),
            text_color: Color { a: 0.5, ..WHITE },
            ..base
        },
    }
}

/// Ghost / subtle button — transparent bg, shows bg on hover.
pub fn btn_ghost(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let base = button::Style {
        background: None,
        text_color: p.text_muted,
        border: Border {
            radius: RADIUS_SM.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
        snap: true,
    };
    match status {
        button::Status::Active | button::Status::Pressed => base,
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.surface_alt)),
            text_color: p.text,
            ..base
        },
        button::Status::Disabled => base,
    }
}

/// Sidebar tab — unselected.
pub fn btn_tab_inactive(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let base = button::Style {
        background: None,
        text_color: p.text_muted,
        border: Border {
            radius: RADIUS_SM.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
        snap: true
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.border)),
            text_color: p.text,
            ..base
        },
        _ => base,
    }
}

/// Sidebar tab — selected.
pub fn btn_tab_active(_theme: &Theme, _status: button::Status) -> button::Style {
    button::Style {
        background: Some(Background::Color(ACCENT)),
        text_color: WHITE,
        border: Border {
            radius: RADIUS_SM.into(),
            ..Border::default()
        },
        shadow: subtle_shadow(&LIGHT),
        snap: true
    }
}

/// Small row-delete button — subtle, turns red on hover.
pub fn btn_row_delete(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let base = button::Style {
        background: None,
        text_color: p.text_muted,
        border: Border {
            radius: RADIUS_SM.into(),
            ..Border::default()
        },
        shadow: Shadow::default(),
        snap: true
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(Color {
                a: 0.12,
                ..DANGER
            })),
            text_color: DANGER,
            ..base
        },
        button::Status::Pressed => button::Style {
            background: Some(Background::Color(Color {
                a: 0.20,
                ..DANGER
            })),
            text_color: DANGER,
            ..base
        },
        _ => base,
    }
}

/// Outlined cancel / secondary button.
pub fn btn_outlined(theme: &Theme, status: button::Status) -> button::Style {
    let p = palette(theme);
    let base = button::Style {
        background: Some(Background::Color(p.surface)),
        text_color: p.text,
        border: Border {
            radius: RADIUS_SM.into(),
            width: 1.0,
            color: p.border,
        },
        shadow: Shadow::default(),
        snap: true
    };
    match status {
        button::Status::Hovered => button::Style {
            background: Some(Background::Color(p.surface_alt)),
            border: Border {
                color: ACCENT,
                ..base.border
            },
            text_color: ACCENT,
            ..base
        },
        _ => base,
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  Container styles
// ═══════════════════════════════════════════════════════════════════════

/// The main application background.
pub fn app_background(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(Background::Color(p.bg)),
        ..container::Style::default()
    }
}

/// A card / panel with rounded corners, soft shadow.
pub fn card(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(Background::Color(p.surface)),
        border: Border {
            radius: RADIUS_MD.into(),
            width: 1.0,
            color: p.border,
        },
        shadow: card_shadow(p),
        ..container::Style::default()
    }
}

/// Table header row.
pub fn table_header(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(Background::Color(p.surface_alt)),
        border: Border {
            radius: Radius {
                top_left: RADIUS_SM,
                top_right: RADIUS_SM,
                bottom_right: 0.0,
                bottom_left: 0.0,
            },
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// Even row stripe.
pub fn row_even(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(Background::Color(p.surface)),
        ..container::Style::default()
    }
}

/// Odd row stripe.
pub fn row_odd(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(Background::Color(p.surface_alt)),
        ..container::Style::default()
    }
}

/// Config sidebar panel.
pub fn sidebar(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(Background::Color(p.surface_alt)),
        border: Border {
            radius: Radius {
                top_left: RADIUS_MD,
                top_right: 0.0,
                bottom_right: 0.0,
                bottom_left: RADIUS_MD,
            },
            ..Border::default()
        },
        ..container::Style::default()
    }
}

/// Semi-transparent overlay behind a dialog.
pub fn overlay(_theme: &Theme) -> container::Style {
    container::Style {
        background: Some(Background::Color(OVERLAY_BG)),
        ..container::Style::default()
    }
}

/// Dialog card — prominent shadow.
pub fn dialog_card(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(Background::Color(p.surface)),
        border: Border {
            radius: RADIUS_LG.into(),
            ..Border::default()
        },
        shadow: Shadow {
            color: Color::from_rgba(0.0, 0.0, 0.0, 0.15),
            offset: Vector { x: 0.0, y: 8.0 },
            blur_radius: 30.0,
        },
        ..container::Style::default()
    }
}

/// Bottom action bar.
pub fn bottom_bar(theme: &Theme) -> container::Style {
    let p = palette(theme);
    container::Style {
        background: Some(Background::Color(p.surface)),
        border: Border {
            radius: Radius {
                top_left: 0.0,
                top_right: 0.0,
                bottom_right: RADIUS_MD,
                bottom_left: RADIUS_MD,
            },
            ..Border::default()
        },
        ..container::Style::default()
    }
}

// ═══════════════════════════════════════════════════════════════════════
//  Text input styles
// ═══════════════════════════════════════════════════════════════════════

pub fn input_style(theme: &Theme, status: text_input::Status) -> text_input::Style {
    let p = palette(theme);
    let base = text_input::Style {
        background: Background::Color(p.surface),
        border: Border {
            radius: RADIUS_SM.into(),
            width: 1.0,
            color: p.border,
        },
        icon: p.text_muted,
        placeholder: p.text_muted,
        value: p.text,
        selection: Color { a: 0.2, ..ACCENT },
    };
    match status {
        text_input::Status::Focused { is_hovered: _ } => text_input::Style {
            border: Border {
                color: ACCENT,
                width: 2.0,
                ..base.border
            },
            ..base
        },
        text_input::Status::Hovered => text_input::Style {
            border: Border {
                color: ACCENT_HOVER,
                ..base.border
            },
            ..base
        },
        _ => base,
    }
}
