use std::sync::Arc;

use epaint::Stroke;

use crate::{
    Frame, Id, Style, TextStyle, Ui,
    util::IdTypeMap,
    widget_style::{Classes, StyleStruct, TextVisuals, WidgetState, WidgetStyle},
};

impl ThemeStyle<WidgetStyle> for Style {
    fn style(&self, _classes: &Classes, state: WidgetState, base: &Self) -> WidgetStyle {
        let visuals = base.visuals.widgets.state(state);
        let font_id = base.override_font_id.clone();
        WidgetStyle {
            frame: Frame {
                fill: visuals.bg_fill,
                stroke: visuals.bg_stroke,
                corner_radius: visuals.corner_radius,
                inner_margin: base.spacing.button_padding.into(),
                ..Default::default()
            },
            stroke: visuals.fg_stroke,
            text: TextVisuals {
                color: base
                    .visuals
                    .override_text_color
                    .unwrap_or_else(|| visuals.text_color()),
                font_id: font_id.unwrap_or_else(|| TextStyle::Body.resolve(base)),
                strikethrough: Stroke::NONE,
                underline: Stroke::NONE,
            },
        }
    }
}

/// A Theme plugin that implement a style computation for a defined `StyleStruct`
pub trait ThemeStyle<S> {
    /// The style according to the classes and state of the widget
    fn style(&self, classes: &Classes, state: WidgetState, base: &Style) -> S;
}

impl Ui {
    /// Access the installed theme plugin if there is one and fetch the requested widget style if it exist.
    /// Fallback to the default style if not found.
    ///
    /// Requested widget style must implement [`StyleStruct`].
    pub fn widget_style<S: StyleStruct + Clone + 'static>(
        &self,
        id: crate::Id,
        classes: &Classes,
    ) -> S {
        // If the requested `StyleStruct` is cached, return it without computing.
        // Otherwise proceed to compute the style from the widget information.

        // Fetch the current state of the widget
        let state = self
            .ctx()
            .read_response(id)
            .map(|r| r.widget_state())
            .unwrap_or_default();

        if let Some(style) = self.get_style::<S>(classes, state, self.style()) {
            style
        } else {
            S::default_style(classes, state, self.style())
        }
    }
}

#[derive(Default)]
pub(crate) struct Themes {
    themes: IdTypeMap,
    /// The current theme cache
    cache: IdTypeMap,
}

impl Themes {
    /// Register a theme and the style associated
    pub(crate) fn register<S: StyleStruct + 'static>(
        &mut self,
        theme: impl ThemeStyle<S> + Send + Sync + 'static,
    ) {
        self.themes
            .insert_temp::<Arc<dyn ThemeStyle<S> + Send + Sync>>(Id::NULL, Arc::new(theme));
        self.cache.remove_by_type::<S>();
    }

    /// Fetch the style of the current theme
    pub(crate) fn get<S: StyleStruct + Clone + 'static>(
        &mut self,
        classes: &Classes,
        state: WidgetState,
        base: &Style,
    ) -> Option<S> {
        let style_id = Id::new(classes).with(state);
        self.cache.get_temp::<S>(style_id);

        if let Some(cached_style) = self.cache.get_temp::<S>(style_id) {
            return Some(cached_style);
        }

        let style = self
            .themes
            .get_temp::<Arc<dyn ThemeStyle<S> + Send + Sync>>(Id::NULL)
            .map(|engine| engine.style(classes, state, base))?;

        self.cache.insert_temp::<S>(style_id, style.clone());
        Some(style)
    }
}
