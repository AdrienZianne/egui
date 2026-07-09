use std::{collections::HashMap, sync::Arc};

use eframe::egui::{
    Color32, Context, Stroke, Ui, UiStack,
    theme_plugin::ThemeStyle,
    widget_style::{BaseStyle, ButtonStyle, Classes, HasClasses as _, WidgetState},
};
use logos::Logos;
use pest::{
    Parser as _,
    iterators::{Pair, Pairs},
};
use pest_derive::Parser;

#[derive(Debug, Default, Clone)]
pub struct ESSEngine {
    info: HashMap<String, Ess>,
}

/// Example of a ESS file
///
/// .button {
///     fill: white;
///
/// }
///
/// .button:hover {
///     border: 2 red;
/// }
#[derive(Debug, Default, Clone)]
pub struct Ess {
    rules: HashMap<(WidgetState, Vec<(String, WidgetState)>), Vec<Property>>,
}

#[derive(Parser)]
#[grammar = "ess.pest"]
pub struct ESSParser;

impl ESSEngine {
    pub fn try_parse(ess: &str) -> Result<Self, String> {
        let res = ESSParser::parse(Rule::sheet, ess).map_err(|e| e.to_string())?;
        let mut engine = Self::default();
        for rule in res {
            let mut inner = rule.into_inner();
            let selectors = inner.next().expect("selectors should exist");
            let Some(properties) = inner.next() else {
                // Ignore this rule if no property
                continue;
            };

            let mut classes = selectors.into_inner().rev();

            let first_class_state = classes.next().expect("at least one class");
            let first_class_state = parse_class_state(first_class_state)?;

            let mut ancestors: Vec<(String, WidgetState)> = vec![];

            for class_state in classes {
                ancestors.push(parse_class_state(class_state)?);
            }

            let mut class_properties = vec![];

            for property in properties.into_inner() {
                class_properties.push(parse_property(property.into_inner())?);
            }

            engine.insert_rule(first_class_state, ancestors, class_properties);
        }

        // println!("{:?}", engine);

        Ok(engine)
    }

    fn insert_rule(
        &mut self,
        class: (String, WidgetState),
        ancestors: Vec<(String, WidgetState)>,
        properties: Vec<Property>,
    ) {
        self.info
            .entry(class.0)
            .or_default()
            .rules
            .insert((class.1, ancestors), properties);
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
        let base = ui.get_widget_style::<BaseStyle>(classes, state);
        let mut default = ButtonStyle {
            frame: base.frame,
            text_style: base.text,
        };
        for classe in classes.list() {
            if let Some(ess) = self.info.get(&classe.to_string()) {
                // println!("uh");
                let keys = ess.rules.keys().clone().collect::<Vec<_>>();
                // keys.sort();
                for hierarchy in keys {
                    let properties = &ess.rules[hierarchy];

                    if has_ancestors(ui, stack, hierarchy.1.clone()) {
                        for property in properties {
                            match property {
                                Property::Border(width, color) => {
                                    default.frame.stroke = if let Some(color) = color {
                                        Stroke::new(*width, *color)
                                    } else {
                                        Stroke::new(*width, default.frame.stroke.color)
                                    }
                                }
                                Property::BorderColor(color) => default.frame.stroke.color = *color,
                                Property::Fill(color) => default.frame.fill = *color,
                                _ => {}
                            }
                        }
                    }
                }
            }
        }
        default
    }
}

pub fn has_ancestors(
    ctx: &Context,
    stack: &UiStack,
    mut classes: Vec<(String, WidgetState)>,
) -> bool {
    let Some(mut current_ancestor) = classes.pop() else {
        return true;
    };

    let mut ancestors = stack.ancestors();
    ancestors.reverse();

    let ancestor = stack;
    while let Some(ancestor) = &ancestor.parent {
        let state = ctx
            .read_response(ancestor.id)
            .map_or(WidgetState::Inactive, |r| r.widget_state());

        if ancestor.classes.has(&current_ancestor.0) && state == current_ancestor.1 {
            if let Some(next_ancestor) = classes.pop() {
                current_ancestor = next_ancestor;
            } else {
                return true;
            }
        }
    }

    false
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
    #[token(";")]
    End,
    #[regex(r"[a-zA-Z]+")]
    Property,
    #[regex(r"[0-9]+")]
    Number,
    #[regex(r"#(?:[0-9a-fA-F]{3}){1,2}")]
    Color,
    #[regex(r"[ \t\n\f]+", logos::skip)]
    Whitespace,
}

#[derive(Debug, Clone)]
enum Property {
    Border(f32, Option<Color32>),
    BorderColor(Color32),
    Fill(Color32),
    Margin(i8, Option<i8>, Option<(i8, i8)>),
    Padding(i8, Option<i8>, Option<(i8, i8)>),
    CornerRadius(usize),
    Font(String, Option<f32>),
    FontSize(f32),
}

fn parse_class_state(pair: Pair<'_, Rule>) -> Result<(String, WidgetState), String> {
    if pair.as_rule() != Rule::class_state {
        return Err("Not a class_state".to_owned());
    }
    let mut class_state = pair.into_inner();
    let class = class_state
        .next()
        .expect("At least the class name")
        .as_str()[1..]
        .to_owned();
    let state = if let Some(state) = class_state.next() {
        match state.as_str() {
            "hover" => WidgetState::Hovered,
            "active" => WidgetState::Active,
            "disable" => WidgetState::Noninteractive,
            _ => WidgetState::Inactive,
        }
    } else {
        WidgetState::Inactive
    };
    Ok((class, state))
}

fn parse_property(mut property: Pairs<'_, Rule>) -> Result<Property, String> {
    let name = if let Some(name) = property.next() {
        name.as_str()
    } else {
        return Err("Missing name".to_owned());
    };

    let Some(value) = property.next() else {
        return Err("Missing value".to_owned());
    };

    match name {
        "border" => parse_border(value.into_inner()),
        "border_color" => parse_border_color(value.into_inner()),
        "fill" => parse_fill(value.into_inner()),
        _ => Err(format!("Property [{name}] not found")),
    }
}

fn parse_border(mut border: Pairs<'_, Rule>) -> Result<Property, String> {
    let width = if let Some(width) = border.next() {
        parse_float(&width)?
    } else {
        return Err("Missing thickness".to_owned());
    };

    let color = if let Some(color) = border.next() {
        Some(parse_color(&color)?)
    } else {
        None
    };

    Ok(Property::Border(width, color))
}

fn parse_border_color(mut border_color: Pairs<'_, Rule>) -> Result<Property, String> {
    let color = if let Some(color) = border_color.next() {
        parse_color(&color)?
    } else {
        return Err("Missing color".to_owned());
    };

    Ok(Property::BorderColor(color))
}

fn parse_fill(mut fill: Pairs<'_, Rule>) -> Result<Property, String> {
    let color = if let Some(color) = fill.next() {
        parse_color(&color)?
    } else {
        return Err("Missing color".to_owned());
    };

    Ok(Property::Fill(color))
}

// ################### Value ##########################""

fn parse_color(color: &Pair<'_, Rule>) -> Result<Color32, String> {
    if color.as_rule() == Rule::color {
        Ok(Color32::from_hex(&color.as_str()).expect("should be a color"))
    } else {
        Err("Not a color".to_owned())
    }
}

fn parse_number(number: &Pair<'_, Rule>) -> Result<usize, String> {
    if number.as_rule() == Rule::number {
        Ok(number
            .as_str()
            .parse::<usize>()
            .expect("should be a number"))
    } else {
        Err("Not a number".to_owned())
    }
}

fn parse_float(number: &Pair<'_, Rule>) -> Result<f32, String> {
    if number.as_rule() == Rule::number {
        Ok(number.as_str().parse::<f32>().expect("should be a number"))
    } else {
        Err("Not a number".to_owned())
    }
}

fn parse_integer(integer: &Pair<'_, Rule>) -> Result<i8, String> {
    if integer.as_rule() == Rule::number {
        Ok(integer
            .as_str()
            .parse::<i8>()
            .expect("should be a 8 bit integer"))
    } else {
        Err("Not a integer".to_owned())
    }
}

#[derive(Debug, Clone)]
enum Value {
    Number(i32),
    Percentage(usize),
    Text(String),
    Color(Color32),
}
