//! The Nocturne theme (spec §8.1): colours, visuals and fonts. Colours given
//! in the design as oklch are converted to sRGB here once.

use egui::{Color32, CornerRadius, FontData, FontDefinitions, FontFamily, Stroke, Theme};
use fp_model::CartColor;

pub const BG: Color32 = Color32::from_rgb(0x16, 0x18, 0x26);
pub const SURFACE: Color32 = Color32::from_rgb(0x23, 0x25, 0x32);
pub const TEXT: Color32 = Color32::from_rgb(0xe9, 0xe9, 0xed);
pub const ACCENT: Color32 = Color32::from_rgb(0x91, 0x84, 0xd9);
pub const ACCENT_400: Color32 = Color32::from_rgb(0xb5, 0xab, 0xfc);

pub const NEUTRAL_100: Color32 = Color32::from_rgb(0xf3, 0xf5, 0xfe);
pub const NEUTRAL_200: Color32 = Color32::from_rgb(0xe4, 0xe7, 0xf5);
pub const NEUTRAL_300: Color32 = Color32::from_rgb(0xcf, 0xd3, 0xe5);
pub const NEUTRAL_400: Color32 = Color32::from_rgb(0xb2, 0xb6, 0xca);
pub const NEUTRAL_500: Color32 = Color32::from_rgb(0x93, 0x97, 0xab);
pub const NEUTRAL_600: Color32 = Color32::from_rgb(0x75, 0x79, 0x8c);
pub const NEUTRAL_700: Color32 = Color32::from_rgb(0x59, 0x5d, 0x6c);
pub const NEUTRAL_800: Color32 = Color32::from_rgb(0x3f, 0x42, 0x4d);
pub const NEUTRAL_900: Color32 = Color32::from_rgb(0x29, 0x2b, 0x31);

/// Row and state colours.
pub const ON_AIR_ROW: Color32 = Color32::from_rgb(0xb0, 0x2a, 0x2d);
pub const ON_AIR_TEXT: Color32 = Color32::from_rgb(0xff, 0x64, 0x5f);
pub const NEXT_ROW: Color32 = Color32::from_rgb(0x14, 0x6d, 0x34);
pub const NEXT_TEXT: Color32 = Color32::from_rgb(0x3e, 0xab, 0x5e);
pub const AMBER: Color32 = Color32::from_rgb(0xf0, 0xb1, 0x35);
pub const CUE: Color32 = Color32::from_rgb(0x6b, 0xcb, 0xf7);
pub const PLAY: Color32 = Color32::from_rgb(0x3f, 0xc1, 0x68);

/// Meter zones: muted traffic-light tones that sit with the Nocturne
/// neutrals (feedback spec F9).
pub const METER_NORMAL: Color32 = Color32::from_rgb(0x7f, 0xb0, 0x8a);
pub const METER_WARNING: Color32 = Color32::from_rgb(0xd9, 0xb4, 0x5a);
pub const METER_DANGER: Color32 = Color32::from_rgb(0xd8, 0x64, 0x6a);

/// Meter rulers (operator feedback 4, Q11): a major tick for every label,
/// a fainter minor tick between them, and the alignment level in white,
/// all outside the bars.
pub const METER_TICK: Color32 = NEUTRAL_400;
pub const METER_TICK_MINOR_ALPHA: f32 = 0.60;
pub const METER_ALIGNMENT_TICK: Color32 = Color32::WHITE;

/// Waveform markers.
pub const INTRO: Color32 = Color32::from_rgb(0x43, 0xb2, 0xe1);
pub const OUTRO_SHADE: Color32 = Color32::from_rgb(0xf4, 0xa2, 0x5c);
pub const OUTRO_LINE: Color32 = Color32::from_rgb(0xfb, 0xa9, 0x62);

/// Font family names registered by [`fonts`].
pub const MEDIUM: &str = "inter-500";
pub const SEMIBOLD: &str = "inter-600";
pub const ICONS_FILL: &str = "phosphor-fill";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct WaveColors {
    pub played: Color32,
    pub unplayed: Color32,
}

const fn wave(played: [u8; 3], unplayed: [u8; 3]) -> WaveColors {
    WaveColors {
        played: Color32::from_rgb(played[0], played[1], played[2]),
        unplayed: Color32::from_rgb(unplayed[0], unplayed[1], unplayed[2]),
    }
}

/// The waveform colours offered in Settings, by configuration name.
pub const WAVE_PALETTE: &[(&str, WaveColors)] = &[
    ("violet", wave([0xb5, 0xab, 0xfc], [0x59, 0x5d, 0x6c])),
    ("amber", wave([0xf9, 0xb6, 0x4f], [0x6e, 0x52, 0x32])),
    ("cyan", wave([0x58, 0xd1, 0xe5], [0x35, 0x5f, 0x6a])),
    ("white", wave([0xf3, 0xf5, 0xfe], [0x59, 0x5d, 0x6c])),
    ("orange", wave([0xf9, 0x89, 0x42], [0x71, 0x46, 0x2e])),
    ("magenta", wave([0xe5, 0x6b, 0xc1], [0x66, 0x3d, 0x59])),
    ("ice", wave([0xb7, 0xde, 0xf3], [0x4f, 0x60, 0x6d])),
    ("sand", wave([0xe0, 0xcf, 0xac], [0x67, 0x5c, 0x4b])),
    ("slate", wave([0xcf, 0xd3, 0xe5], [0x4a, 0x4e, 0x5c])),
];

const DEFAULT_WAVE: WaveColors = wave([0xcf, 0xd3, 0xe5], [0x4a, 0x4e, 0x5c]);

/// Colours for a configured name; an unknown name falls back to Slate.
pub fn wave_colors(name: &str) -> WaveColors {
    WAVE_PALETTE
        .iter()
        .find(|(n, _)| n.eq_ignore_ascii_case(name))
        .map_or(DEFAULT_WAVE, |(_, c)| *c)
}

/// Inter for text (three weights) and Phosphor icons (regular mixed into
/// ordinary text, fill as its own family).
pub fn fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let inter: [(&str, &'static [u8]); 3] = [
        (
            "inter-400",
            include_bytes!("../../assets/fonts/Inter-400.ttf"),
        ),
        (MEDIUM, include_bytes!("../../assets/fonts/Inter-500.ttf")),
        (SEMIBOLD, include_bytes!("../../assets/fonts/Inter-600.ttf")),
    ];
    for (name, bytes) in inter {
        fonts
            .font_data
            .insert(name.to_owned(), FontData::from_static(bytes).into());
    }
    let fallbacks = fonts
        .families
        .get(&FontFamily::Proportional)
        .cloned()
        .unwrap_or_default();
    fonts
        .families
        .entry(FontFamily::Proportional)
        .or_default()
        .insert(0, "inter-400".to_owned());
    for weight in [MEDIUM, SEMIBOLD] {
        let mut keys = vec![weight.to_owned()];
        keys.extend(fallbacks.iter().cloned());
        fonts.families.insert(FontFamily::Name(weight.into()), keys);
    }
    egui_phosphor::add_to_fonts(&mut fonts, egui_phosphor::Variant::Regular);
    // Icons in the weight families too, so a bold label can carry one.
    for weight in [MEDIUM, SEMIBOLD] {
        if let Some(keys) = fonts.families.get_mut(&FontFamily::Name(weight.into())) {
            keys.insert(1, "phosphor".to_owned());
        }
    }
    egui_phosphor::add_font_bytes_as_family(
        &mut fonts,
        ICONS_FILL,
        egui_phosphor::Variant::Fill.font_bytes(),
    );
    fonts
}

/// Installs the fonts and the dark visuals: square corners, Nocturne colours.
pub fn apply(ctx: &egui::Context) {
    ctx.set_fonts(fonts());
    ctx.set_theme(Theme::Dark);
    ctx.style_mut_of(Theme::Dark, |style| {
        let v = &mut style.visuals;
        v.dark_mode = true;
        v.panel_fill = BG;
        v.window_fill = SURFACE;
        v.extreme_bg_color = BG;
        v.faint_bg_color = SURFACE;
        v.override_text_color = Some(TEXT);
        v.hyperlink_color = ACCENT_400;
        v.selection.bg_fill = ACCENT.gamma_multiply(0.45);
        v.selection.stroke = Stroke::new(1.0, ACCENT_400);
        v.window_corner_radius = CornerRadius::ZERO;
        v.menu_corner_radius = CornerRadius::ZERO;
        v.window_stroke = Stroke::new(1.0, NEUTRAL_800);
        let widgets = &mut v.widgets;
        for w in [
            &mut widgets.noninteractive,
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
            &mut widgets.open,
        ] {
            w.corner_radius = CornerRadius::ZERO;
        }
        widgets.noninteractive.bg_fill = SURFACE;
        widgets.noninteractive.bg_stroke = Stroke::new(1.0, NEUTRAL_900);
        widgets.noninteractive.fg_stroke = Stroke::new(1.0, NEUTRAL_400);
        widgets.inactive.bg_fill = NEUTRAL_900;
        widgets.inactive.weak_bg_fill = NEUTRAL_900;
        widgets.inactive.fg_stroke = Stroke::new(1.0, NEUTRAL_200);
        widgets.hovered.bg_fill = NEUTRAL_800;
        widgets.hovered.weak_bg_fill = NEUTRAL_800;
        widgets.hovered.bg_stroke = Stroke::new(1.0, NEUTRAL_700);
        widgets.hovered.fg_stroke = Stroke::new(1.0, NEUTRAL_100);
        widgets.active.bg_fill = NEUTRAL_700;
        widgets.active.weak_bg_fill = NEUTRAL_700;
        widgets.active.fg_stroke = Stroke::new(1.0, NEUTRAL_100);
        style.spacing.item_spacing = egui::vec2(6.0, 4.0);
        style.spacing.button_padding = egui::vec2(8.0, 4.0);
    });
}

/// Player column background: surface 40 % over the page background.
pub const COLUMN_BG: Color32 = Color32::from_rgb(0x1b, 0x1d, 0x2b);
/// Selected row and menu hover.
pub const ACCENT_900: Color32 = Color32::from_rgb(0x2c, 0x28, 0x4a);
pub const ACCENT_300: Color32 = Color32::from_rgb(0xc9, 0xc1, 0xff);
/// The outline of the selected playlist row, drawn on any row colour (the
/// on-air and next fills hide the selection's).
pub const SELECTION_OUTLINE: Color32 = ACCENT_300;
/// Paused / stop-after highlight.
pub const AMBER_BG: Color32 = Color32::from_rgb(0x43, 0x2f, 0x07);
pub const AMBER_TEXT: Color32 = Color32::from_rgb(0xfc, 0xd1, 0x76);
pub const AMBER_DIM: Color32 = Color32::from_rgb(0x9f, 0x79, 0x32);
pub const CUE_BG: Color32 = Color32::from_rgb(0x0d, 0x32, 0x42);
pub const PLAY_BORDER: Color32 = Color32::from_rgb(0x3e, 0xab, 0x5e);
pub const PLAY_HOVER_BG: Color32 = Color32::from_rgb(0x14, 0x36, 0x1d);
pub const PLAY_HOVER: Color32 = Color32::from_rgb(0x93, 0xe4, 0xa4);
pub const PLAY_ACTIVE_BG: Color32 = Color32::from_rgb(0x1d, 0x4e, 0x2b);

// The big Play / Next button is a flat green button with a white icon and
// no outline, in the green of the next row of the playlist (itr07 J4). Every
// fill below keeps the icon at WCAG >= 3:1 (`tests/it/play_button_ui.rs` checks
// it).
/// Idle and on air (the caption tells Play from Next): the next row's green.
pub const PLAY_FILL: Color32 = NEXT_ROW;
/// On air: the button is Next, in the same green as Play.
pub const PLAY_FILL_ON_AIR: Color32 = PLAY_FILL;
/// Pointer over it: a step lighter.
pub const PLAY_FILL_HOVER: Color32 = Color32::from_rgb(0x1b, 0x84, 0x41);
/// Pressed: a step darker.
pub const PLAY_FILL_PRESSED: Color32 = Color32::from_rgb(0x0f, 0x55, 0x27);
/// Disabled: a muted dark green, with the icon at half strength.
pub const PLAY_FILL_DISABLED: Color32 = Color32::from_rgb(0x1a, 0x2f, 0x23);
/// Paused: the dark phase of the blink (lit, it is [`PLAY_FILL`]).
pub const PLAY_FILL_PAUSED_DIM: Color32 = Color32::from_rgb(0x0e, 0x3a, 0x1d);
/// The icon and caption on the fill.
pub const PLAY_ICON: Color32 = Color32::WHITE;
/// The ring of a button that has keyboard focus (all tiles).
pub const FOCUS_RING: Color32 = ACCENT_300;

/// Share of its tone that a fade countdown band paints over a button.
pub const SWEEP_STRENGTH: f32 = 0.35;
/// The countdown band on the Play / Next button: a lighter green on the fill.
pub fn play_sweep() -> Color32 {
    PLAY.gamma_multiply(0.55)
}
/// The countdown band on a button whose content has the tone `content`.
pub fn sweep_tone(content: Color32) -> Color32 {
    content.gamma_multiply(SWEEP_STRENGTH)
}

/// The divider between the level meter and the volume fader: the same
/// hairline as the waveform's border, so the player reads as one piece.
pub const DIVIDER: Color32 = NEUTRAL_800;
pub const INTRO_BADGE_BG: Color32 = Color32::from_rgb(0x00, 0x23, 0x32);
pub const INTRO_BADGE_BLINK: Color32 = Color32::from_rgb(0x00, 0x50, 0x73);
pub const INTRO_BADGE_TEXT: Color32 = Color32::from_rgb(0xb6, 0xe6, 0xff);
pub const OUTRO_BADGE_BG: Color32 = Color32::from_rgb(0x35, 0x1e, 0x08);
pub const OUTRO_BADGE_TEXT: Color32 = Color32::from_rgb(0xff, 0xd5, 0xa4);
/// The active mix (segue) marker on the waveform: violet like the MIXING
/// badge, so it cannot be taken for the orange outro marker.
pub const MIX_LINE: Color32 = ACCENT;
/// The MIX tag's text, dark for contrast on the violet (and on the grey of an
/// inactive marker).
pub const MIX_TEXT: Color32 = Color32::from_rgb(0x14, 0x12, 0x20);

/// Opacity of the cue-in and cue-out lines on the waveform while players
/// ignore cue markers (feedback 2 spec O15).
pub const CUE_EDGE_IGNORED_ALPHA: f32 = 0.35;

/// Cart colours (cart colour spec §3): the tone of each `CartColor`. The
/// state stores the name; only this table knows the tones.
pub fn cart_color(color: CartColor) -> Color32 {
    match color {
        CartColor::Red => Color32::from_rgb(0xd8, 0x64, 0x6a),
        CartColor::Orange => Color32::from_rgb(0xe8, 0x92, 0x5a),
        CartColor::Amber => Color32::from_rgb(0xd9, 0xb4, 0x5a),
        CartColor::Green => Color32::from_rgb(0x7f, 0xb0, 0x8a),
        CartColor::Teal => Color32::from_rgb(0x4f, 0xb3, 0xa9),
        CartColor::Blue => Color32::from_rgb(0x5b, 0x9f, 0xe0),
        CartColor::Violet => Color32::from_rgb(0x91, 0x84, 0xd9),
        CartColor::Pink => Color32::from_rgb(0xd2, 0x7f, 0xb4),
    }
}

/// How strongly a cart's colour washes its button, idle (CC6) and playing
/// (CC7).
pub const CART_WASH_IDLE: f32 = 0.07;
pub const CART_WASH_PLAYING: f32 = 0.16;
/// The elapsed-time fill of a playing cart with a colour (CC7).
pub const CART_PROGRESS_ALPHA: f32 = 0.38;
/// The waveform behind a cart's text, idle and playing (CC8).
pub const CART_WAVE_IDLE_ALPHA: f32 = 0.13;
pub const CART_WAVE_PLAYING_ALPHA: f32 = 0.22;
/// The waveform of a cart without a colour: light grey (CC8).
pub const CART_WAVE_PLAIN: Color32 = NEUTRAL_300;
