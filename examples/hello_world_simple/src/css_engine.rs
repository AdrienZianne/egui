use std::collections::{HashMap, HashSet};

use eframe::egui::{
    self, Color32, Frame, TextFormat,
    style_trait::{
        Modifiers, Modifiers, Property, StyleContext, StyleEngine, StyleModifier, StyleSheet,
    },
};

type Properties = HashMap<Modifiers, HashSet<Property>>;

#[derive(Default)]
pub struct CssEngine {
    style: HashMap<StyleModifier, Properties>,
}

impl StyleEngine for CssEngine {
    fn get(&self, ctx: &StyleContext<'_>) -> StyleSheet {
        let properties = self.get_property(ctx.classes, ctx.modifier);
        let mut ss = StyleSheet::default();
        for property in properties {
            match property {
                Property::Margin(margin) => ss.margin = margin,
                Property::Padding(margin) => ss.padding = margin,
                Property::Border(stroke) => ss.border = stroke,
                Property::Background(color32) => ss.background = color32,
                Property::Transform(tstransform) => ss.transform = tstransform,
            }
        }
        ss
    }
}

impl CssEngine {
    pub fn add_property(&mut self, classes: &Modifiers, modifier: &Modifiers, property: &Property) {
        for class in classes.0.clone() {
            self.style
                .entry(class)
                .and_modify(|f| {
                    f.entry(*modifier)
                        .and_modify(|p| {
                            p.replace(*property);
                        })
                        .or_insert_with(|| {
                            let mut h = HashSet::new();
                            h.insert(*property);
                            h
                        });
                })
                .or_insert_with(|| {
                    let mut h: Properties = HashMap::new();
                    let mut s: HashSet<Property> = HashSet::new();
                    s.insert(*property);
                    h.insert(*modifier, s);
                    h
                });
        }
    }

    pub fn add_properties(
        &mut self,
        classes: &Modifiers,
        modifier: &Modifiers,
        properties: &Vec<Property>,
    ) {
        for class in classes.0.clone() {
            self.style
                .entry(class)
                .and_modify(|f| {
                    f.entry(*modifier)
                        .and_modify(|p| {
                            p.extend(properties);
                        })
                        .or_insert_with(|| {
                            let mut h = HashSet::new();
                            h.extend(properties);
                            h
                        });
                })
                .or_insert_with(|| {
                    let mut h: Properties = HashMap::new();
                    let mut s: HashSet<Property> = HashSet::new();
                    s.extend(properties);
                    h.insert(*modifier, s);
                    h
                });
        }
    }

    fn get_property(&self, classes: &Modifiers, modifier: &Modifiers) -> HashSet<Property> {
        let mut properties: HashSet<Property> = HashSet::new();
        for classe in &classes.0 {
            if let Some(x) = self.style.get(&classe) {
                if let Some(p) = x.get(modifier) {
                    properties.extend(p);
                }
            }
        }
        properties
    }
}
