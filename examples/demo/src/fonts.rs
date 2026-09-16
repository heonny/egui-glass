//! Platform UI fonts (SF Pro / Apple SD Gothic Neo on macOS, Segoe UI / Malgun
//! Gothic on Windows) in front of egui's bundled fonts, which stay as fallbacks
//! for symbols and emoji. Missing files are skipped silently.

use egui::{FontData, FontDefinitions, FontFamily};

#[cfg(target_os = "macos")]
const CANDIDATES: &[(&str, u32)] = &[
    ("/System/Library/Fonts/SFNS.ttf", 0),
    ("/System/Library/Fonts/AppleSDGothicNeo.ttc", 0),
];

#[cfg(target_os = "windows")]
const CANDIDATES: &[(&str, u32)] = &[
    ("C:\\Windows\\Fonts\\segoeui.ttf", 0),
    ("C:\\Windows\\Fonts\\malgun.ttf", 0),
];

#[cfg(not(any(target_os = "macos", target_os = "windows")))]
const CANDIDATES: &[(&str, u32)] = &[];

pub fn system_fonts() -> FontDefinitions {
    let mut fonts = FontDefinitions::default();
    let mut loaded = Vec::new();
    for (path, index) in CANDIDATES {
        let Ok(bytes) = std::fs::read(path) else { continue };
        let name = path.rsplit(['/', '\\']).next().unwrap_or(path).to_owned();
        let mut data = FontData::from_owned(bytes);
        data.index = *index;
        fonts.font_data.insert(name.clone(), std::sync::Arc::new(data));
        loaded.push(name);
    }
    if let Some(proportional) = fonts.families.get_mut(&FontFamily::Proportional) {
        proportional.splice(0..0, loaded);
    }
    fonts
}
