use eframe::egui::{
    Color32, Frame, Plugin, Style,
    widget_style::{ButtonStyle, Classes, HasClasses as _, ThemePlugin, WidgetState, WidgetStyle},
};

pub struct CustomThemePlugin;

impl Plugin for CustomThemePlugin {
    fn debug_name(&self) -> &'static str {
        "test"
    }
}

impl ThemePlugin<WidgetStyle> for CustomThemePlugin {
    fn style(&self, classes: &Classes, state: WidgetState, style: &Style) -> WidgetStyle {
        style.widget_style(classes, state)
    }
}

impl ThemePlugin<ButtonStyle> for CustomThemePlugin {
    fn style(&self, classes: &Classes, state: WidgetState, style: &Style) -> ButtonStyle {
        let default = style.button_style(classes, state);
        let mut button_color = default.frame.fill;
        if classes.has("red") {
            button_color = Color32::RED;
        } else if classes.has("blue") {
            button_color = Color32::BLUE;
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
