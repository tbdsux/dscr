//! Shared look: icons and the Liquid Glass palette both windows draw with.

use rune::{Color, El, Icon, SystemMaterial, TextStyle, col, el, text};
use rune_kit::Kit;

pub(crate) const ICONS: [Icon; 3] = [
    Icon { name: "dscr-screen", view: 24.0, body: r#"<rect x="3" y="4" width="18" height="12" rx="2" stroke="currentColor" stroke-width="1.6" fill="none"/><path d="M8 20h8M12 16v4" stroke="currentColor" stroke-width="1.6" stroke-linecap="round"/>"# },
    Icon { name: "dscr-window", view: 24.0, body: r#"<rect x="3" y="5" width="18" height="14" rx="2" stroke="currentColor" stroke-width="1.6" fill="none"/><path d="M3 9h18" stroke="currentColor" stroke-width="1.6"/>"# },
    Icon { name: "dscr-selection", view: 24.0, body: r#"<path d="M4 9V4h5M15 4h5v5M20 15v5h-5M9 20H4v-5" stroke="currentColor" stroke-width="1.6" stroke-linecap="round" stroke-linejoin="round" fill="none"/>"# },
];

/// The window's corner radius, so the glass follows the frame.
pub(crate) const WINDOW_RADIUS: f32 = 16.0;

/// Translucent colours that read on Liquid Glass in either appearance.
pub(crate) struct Glass { pub(crate) dark: bool, pub(crate) card: Color, pub(crate) edge: Color, pub(crate) hair: Color, pub(crate) well: Color, pub(crate) accent: Color }

impl Glass {
    pub(crate) fn new(k: &Kit, accent: Color) -> Glass {
        let dark = k.dark;
        Glass {
            dark,
            card: if dark { Color::WHITE.alpha(0.06) } else { Color::WHITE.alpha(0.55) },
            edge: if dark { Color::WHITE.alpha(0.10) } else { Color::BLACK.alpha(0.07) },
            hair: if dark { Color::WHITE.alpha(0.07) } else { Color::BLACK.alpha(0.06) },
            well: if dark { Color::WHITE.alpha(0.08) } else { Color::BLACK.alpha(0.05) },
            accent,
        }
    }
    /// The whole window: native glass, with a light tint so text stays legible over busy desktops.
    pub(crate) fn root(&self, k: &Kit) -> El {
        col().w_full().h_full().radius(WINDOW_RADIUS).continuous_corners()
            .system_material(SystemMaterial::Glass).fill(k.c.page)
            .gradient(k.c.page.alpha(if self.dark { 0.35 } else { 0.25 }), k.c.page.alpha(if self.dark { 0.55 } else { 0.4 }))
    }
    pub(crate) fn card(&self) -> El {
        col().w_full().radius(14.0).continuous_corners().fill(self.card).border(1.0, self.edge)
            .shadow(1.0, 3.0, Color::BLACK.alpha(if self.dark { 0.25 } else { 0.05 }))
    }
    /// Accent fill with a soft top highlight and a glow beneath.
    pub(crate) fn accent_fill(&self, e: El) -> El {
        e.gradient(self.accent.mix(Color::WHITE, 0.18), self.accent)
            .border(1.0, Color::WHITE.alpha(0.18))
            .shadow(4.0, 14.0, self.accent.alpha(0.35))
    }
}

/// A group of settings rows with hairlines between them, as in System Settings.
pub(crate) fn group(g: &Glass, rows: Vec<El>) -> El {
    let mut card = g.card();
    for (i, r) in rows.into_iter().enumerate() {
        if i > 0 { card = card.child(el().w_full().h(1.0).px(14.0).child(el().w_full().h_full().fill(g.hair))); }
        card = card.child(r.w_full().h(46.0).px(14.0).items_center());
    }
    card
}

pub(crate) fn caption(k: &Kit, t: &str) -> El {
    text(t.to_uppercase()).font(TextStyle::new(10.5, 650.0)).color(k.c.text_muted).px(6.0)
}
