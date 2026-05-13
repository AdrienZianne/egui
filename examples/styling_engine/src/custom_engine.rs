use std::collections::HashMap;

use eframe::egui::{
    Color32, Frame, Stroke, Style,
    theme_plugin::{ThemePlugin, ThemeStyle},
    widget_style::{
        ButtonStyle, Classes, HasClasses as _, StyleStruct as _, WidgetState, WidgetStyle,
    },
};
use logos::Logos;

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

#[derive(Debug, Default)]
pub struct ESSEngine {
    info: HashMap<String, Vec<(String, Value)>>,
}

impl ESSEngine {
    pub fn try_parse(&mut self, ess: &str) -> Result<(), String> {
        let mut lexer = Token::lexer(ess);
        let mut hash = HashMap::new();
        while let Some(token) = lexer.next() {
            if token == Ok(Token::Class) {
                let selector = lexer.slice()[1..].to_owned();
                if lexer
                    .next()
                    .is_some_and(|token| token.is_ok_and(|token| token != Token::Open))
                {
                    return Err("No opening bracket found !".to_owned());
                }

                let mut declarations = vec![];

                loop {
                    match lexer.next() {
                        Some(Ok(Token::Property)) => {
                            let property = lexer.slice().to_owned();

                            if lexer
                                .next()
                                .is_some_and(|token| token.is_ok_and(|token| token != Token::Is))
                            {
                                return Err("No separator between property and value !".to_owned());
                            }

                            let value = match lexer.next() {
                                Some(Ok(Token::Number)) => Value::Number(
                                    lexer
                                        .slice()
                                        .to_owned()
                                        .parse::<usize>()
                                        .expect("Should be a positive integer"),
                                ),
                                Some(Ok(Token::Color)) => Value::Color(
                                    Color32::from_hex(lexer.slice())
                                        .expect("Should be a valid hex"),
                                ),
                                Some(Ok(v)) => return Err(format!("Invalid value : {v:?}")),
                                _ => return Err("Error".to_owned()),
                            };

                            declarations.push((property, value));
                        }
                        Some(Ok(Token::Close)) => break,
                        Some(Ok(v)) => {
                            return Err(format!("Missing close bracket, found : {v:?}"));
                        }
                        v => return Err(format!("Error : {v:?}")),
                    }
                }

                hash.insert(selector, declarations);
            }
        }
        self.info = hash;
        Ok(())
    }
}

impl ThemePlugin for ESSEngine {
    fn debug_name(&self) -> &'static str {
        "CSS Engine"
    }
}

impl ThemeStyle<WidgetStyle> for ESSEngine {
    fn style(&self, classes: &Classes, state: WidgetState, style: &Style) -> WidgetStyle {
        style.widget_style(classes, state)
    }
}

impl ThemeStyle<ButtonStyle> for ESSEngine {
    fn style(&self, classes: &Classes, state: WidgetState, base: &Style) -> ButtonStyle {
        let mut default = ButtonStyle::default_style(classes, state, base);
        for classe in classes.list() {
            if let Some(properties) = self.info.get(&classe.to_string()) {
                for (property, value) in properties {
                    match property.as_str() {
                        "fill" => {
                            if let Value::Color(color) = value {
                                default.frame.fill = *color;
                            }
                        }
                        "border" => {
                            if let Value::Number(size) = value {
                                default.frame.stroke.width = *size as f32;
                            }
                        }
                        _ => {}
                    }
                }
            }
        }
        default
    }
}

#[derive(Debug, Logos, PartialEq)]
enum Token {
    #[token("{")]
    Open,
    #[token("}")]
    Close,
    #[token(":")]
    Is,
    #[regex(r"\.[a-zA-Z]+")]
    Class,
    #[regex(r"[a-zA-Z]+")]
    Property,
    #[regex(r"[0-9]+")]
    Number,
    #[regex(r"#(?:[0-9a-fA-F]{3}){1,2}")]
    Color,
    #[regex(r"[ \t\n\f;]+", logos::skip)]
    Whitespace,
}

#[derive(Debug)]
enum Value {
    Number(usize),
    String(String),
    Color(Color32),
}
