use std::{collections::HashMap, sync::Arc};

use eframe::egui::{
    Color32, Ui, UiStack,
    theme_plugin::{ThemeCache, ThemeStyle},
    widget_style::{BaseStyle, ButtonStyle, Classes, HasClasses as _, WidgetState},
};
use logos::Logos;

#[derive(Debug, Default, Clone)]
pub struct ESSEngine {
    info: HashMap<String, Rules>,
    cache: ThemeCache,
}

#[derive(Debug, Default, Clone)]
pub struct Rules {
    pub rules: HashMap<Vec<String>, Vec<(String, Value)>>,
}

impl ESSEngine {
    pub fn try_parse(ess: &str) -> Result<Self, String> {
        let mut engine = Self::default();
        let mut lexer = Token::lexer(ess);
        while let Some(token) = lexer.next() {
            let mut hash = HashMap::new();
            if token == Ok(Token::Class) {
                let mut selectors = vec![lexer.slice()[1..].to_owned()];

                loop {
                    let next = lexer.next();
                    if next
                        .as_ref()
                        .is_some_and(|token| token == &Ok(Token::Class))
                    {
                        selectors.push(lexer.slice()[1..].to_owned());
                    } else if next.as_ref().is_some_and(|token| token != &Ok(Token::Open)) {
                        return Err("No opening bracket found !".to_owned());
                    } else {
                        break;
                    }
                }

                let mut rules = vec![];

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

                            rules.push((property, value));
                        }
                        Some(Ok(Token::Close)) => break,
                        Some(Ok(v)) => {
                            return Err(format!("Missing close bracket, found : {v:?}"));
                        }
                        v => return Err(format!("Error : {v:?}")),
                    }
                }

                let selector = selectors.pop().expect("Should have at least one selector");

                engine
                    .info
                    .entry(selector)
                    .and_modify(|e| {
                        e.rules.insert(selectors.clone(), rules.clone());
                    })
                    .or_insert_with(|| {
                        hash.insert(selectors, rules);
                        Rules { rules: hash }
                    });
            }
        }

        Ok(engine)
    }
}

impl ThemeStyle<ButtonStyle> for ESSEngine {
    fn style(
        &mut self,
        ui: &Ui,
        classes: &Classes,
        state: WidgetState,
        stack: &Arc<UiStack>,
    ) -> ButtonStyle {
        self.cache.get(classes, state, || {
            let base = ui.get_widget_style::<BaseStyle>(classes, state);
            let mut default = ButtonStyle {
                frame: base.frame,
                text_style: base.text,
            };
            for classe in classes.list() {
                if let Some(rules) = self.info.get(&classe.to_string()) {
                    let mut keys = rules.rules.keys().clone().collect::<Vec<_>>();
                    keys.sort();
                    for hierarchy in keys {
                        let properties = &rules.rules[hierarchy];

                        println!(
                            "does {} has {:?} in their ancestor ? {:?}",
                            classe,
                            hierarchy,
                            stack.ancestors()
                        );
                        if stack.has_ancestors(hierarchy.clone()) {
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
                }
            }
            default
        })
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

#[derive(Debug, Clone)]
enum Value {
    Number(usize),
    Color(Color32),
}
