use eframe::egui::{
    Color32, Frame, Style,
    theme_plugin::{ThemePlugin, ThemeStyle},
    widget_style::{ButtonStyle, Classes, HasClasses as _, WidgetState, WidgetStyle},
};

#[derive(Default)]
pub struct CustomThemePluginA {
    pub color: Option<Color32>,
}

impl ThemePlugin for CustomThemePluginA {
    fn debug_name(&self) -> &'static str {
        "Engine A"
    }
}

impl ThemeStyle<WidgetStyle> for CustomThemePluginA {
    fn style(&self, classes: &Classes, state: WidgetState, style: &Style) -> WidgetStyle {
        style.widget_style(classes, state)
    }
}

impl ThemeStyle<ButtonStyle> for CustomThemePluginA {
    fn style(&self, classes: &Classes, state: WidgetState, style: &Style) -> ButtonStyle {
        let default = style.button_style(classes, state);
        let mut button_color = default.frame.fill;
        if classes.has("red") {
            button_color = Color32::RED;
        } else if classes.has("blue") {
            button_color = Color32::BLUE;
        } else if classes.has("dynamic")
            && let Some(color) = self.color
        {
            button_color = color;
        }
        ButtonStyle {
            frame: Frame {
                fill: button_color,
                ..default.frame
            },
            text_style: default.text_style,
        }
    }
}

pub struct CustomThemePluginB;

impl ThemePlugin for CustomThemePluginB {
    fn debug_name(&self) -> &'static str {
        "Engine B"
    }
}

impl ThemeStyle<WidgetStyle> for CustomThemePluginB {
    fn style(&self, classes: &Classes, state: WidgetState, style: &Style) -> WidgetStyle {
        style.widget_style(classes, state)
    }
}

impl ThemeStyle<ButtonStyle> for CustomThemePluginB {
    fn style(&self, classes: &Classes, state: WidgetState, style: &Style) -> ButtonStyle {
        let default = style.button_style(classes, state);
        let mut button_color = default.frame.fill;
        if classes.has("red") {
            button_color = Color32::BLUE;
        } else if classes.has("blue") {
            button_color = Color32::RED;
        }
        ButtonStyle {
            frame: Frame {
                fill: button_color,
                ..default.frame
            },
            text_style: default.text_style,
        }
    }
}
