use eframe::egui::{
    self, Frame, TextFormat,
    style_trait::{StyleContext, StyleEngine, StyleSheet},
};

pub struct CssEngine {}

impl StyleEngine<StyleSheet> for CssEngine {
    fn get(&self, ctx: &StyleContext<'_>) -> StyleSheet {
        StyleSheet {
            frame: Frame::default(),
            text: TextFormat::default(),
        }
    }
}

impl CssEngine {}
