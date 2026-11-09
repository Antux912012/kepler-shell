use gtk4::gdk::Display;
use gtk4::CssProvider;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct PaletteFile {
    pub palette: HashMap<String, String>,
    pub semantic: HashMap<String, String>,
}

pub struct ThemeManager;

impl ThemeManager {
    pub fn find_palette(name: &str) -> Option<PaletteFile> {
        let filename = format!("{}.toml", name);
        let candidates = [
            if let Ok(c) = std::env::var("XDG_CONFIG_HOME") {
                PathBuf::from(c).join("kepler-shell/palettes").join(&filename)
            } else if let Ok(h) = std::env::var("HOME") {
                PathBuf::from(h).join(".config/kepler-shell/palettes").join(&filename)
            } else {
                PathBuf::from(&filename)
            },
            PathBuf::from("data/palettes").join(&filename),
            PathBuf::from("/usr/share/kepler-shell/palettes").join(&filename),
        ];

        for path in candidates {
            if path.exists() {
                if let Ok(content) = fs::read_to_string(&path) {
                    if let Ok(palette) = toml::from_str::<PaletteFile>(&content) {
                        return Some(palette);
                    }
                }
            }
        }
        None
    }

    pub fn generate_css_vars(palette: &PaletteFile) -> String {
        let mut css = String::new();
        for (k, v) in &palette.palette {
            css.push_str(&format!("@define-color {} {};\n", k, v));
        }
        for (k, v) in &palette.semantic {
            css.push_str(&format!("@define-color {} {};\n", k, v));
        }
        css
    }

    pub fn load_stylesheet() -> String {
        let candidates = [
            if let Ok(h) = std::env::var("HOME") {
                PathBuf::from(h).join(".config/kepler-shell/style/fluent-adwaita.css")
            } else {
                PathBuf::from("style.css")
            },
            PathBuf::from("data/style/fluent-adwaita.css"),
            PathBuf::from("/usr/share/kepler-shell/style/fluent-adwaita.css"),
        ];

        for p in candidates {
            if p.exists() {
                if let Ok(s) = fs::read_to_string(&p) {
                    return s;
                }
            }
        }

        // Builtin fallback minimal css
        include_str!("../../data/style/fluent-adwaita.css").to_string()
    }

    pub fn apply(provider: &CssProvider, palette_name: &str) {
        let mut full_css = String::new();

        if let Some(pal) = Self::find_palette(palette_name) {
            full_css.push_str(&Self::generate_css_vars(&pal));
        }
        full_css.push_str("\n");
        full_css.push_str(&Self::load_stylesheet());

        provider.load_from_string(&full_css);

        if let Some(display) = Display::default() {
            gtk4::style_context_add_provider_for_display(
                &display,
                provider,
                gtk4::STYLE_PROVIDER_PRIORITY_APPLICATION,
            );
        }
    }
}
