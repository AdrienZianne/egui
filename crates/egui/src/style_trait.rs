use std::sync::Arc;

use epaint::{Color32, FontId, text::TextFormat};
use smallvec::SmallVec;

use crate::{style::WidgetVisuals, Context, Id, Response, Style, Ui};

pub(crate) const CLASSES_SMALL_VEC_SIZE: usize = 5;

#[derive(Debug, Clone, Default)]
pub struct Classes(SmallVec<[Classe; CLASSES_SMALL_VEC_SIZE]>);

impl Classes {
    pub fn with_if(mut self, class: Classe, condition: bool) -> Self {
        if condition {
            self.0.push(class);
        }
        self
    }

    pub fn add_if(&mut self, class: Classe, condition: bool) {
        if condition {
            self.0.push(class);
        }
    }
}

#[derive(Debug, Clone)]
pub enum Classe {
    Button,
    Custom(String),
}

pub trait HasClasses {
    fn classes(&self) -> &Classes;

    fn classes_mut(&mut self) -> &mut Classes;

    fn add_class<T>(&mut self, class: T) -> &Self
    where
        T: Into<Classe>,
    {
        self.classes_mut().add_if(class.into(), true);
        self
    }

    fn with_class<T>(mut self, class: T) -> Self
    where
        Self: Sized,
        T: Into<Classe>,
    {
        self.classes_mut().add_if(class.into(), true);
        self
    }

    fn with_class_if<T>(mut self, class: T, condition: bool) -> Self
    where
        Self: Sized,
        T: Into<Classe>,
    {
        self.classes_mut().add_if(class.into(), condition);
        self
    }
}

impl From<&str> for Classe {
    #[inline]
    fn from(class: &str) -> Self {
        Self::Custom(class.to_owned())
    }
}

pub struct StyleSheet {
    pub background: Color32,
    pub text: TextFormat,
}

impl From<Style> for StyleSheet {
    fn from(value: Style) -> Self {
        StyleSheet {
            background: value.visuals.widgets.inactive.bg_fill,
            text: TextFormat::simple(
                value
                    .override_font_id
                    .unwrap_or(FontId::new(12.0, epaint::FontFamily::Proportional)),
                value.visuals.text_color(),
            ),
        }
    }
}

impl From<&Arc<Style>> for StyleSheet {
    fn from(value: &Arc<Style>) -> Self {
        StyleSheet {
            background: value.visuals.widgets.inactive.bg_fill,
            text: TextFormat::simple(
                value
                    .override_font_id.clone()
                    .unwrap_or(FontId::new(12.0, epaint::FontFamily::Proportional)),
                value.visuals.text_color(),
            ),
        }
    }
}

pub trait StyleEngine<T>: Send + Sync {
    fn get(&self, ctx: &StyleContext) -> T;
}

fn style_id() -> Id {
    Id::new("style-id")
}

pub struct StyleContext<'b> {
    pub ui: &'b Ui,
    pub classes: &'b Classes,
    pub response: &'b Response,
}

impl Context {
    pub fn set_style_engine<T: 'static>(&self, engine: impl StyleEngine<T> + 'static) {
        self.data_mut(|d| {
            d.insert_temp(style_id(), StyleEngineContainer(Arc::new(engine)));
        });
    }
}

impl Ui {
    pub fn widget_style<T: 'static>(&self, ui: &Ui, response: &Response, classes: &Classes) -> T {
        let engine: StyleEngineContainer<T> = self.data_mut(|d| {
            d.get_temp(style_id()).expect("Uh")
        });

        engine.0.get(&StyleContext {
            ui, classes, response
        })
    }
}

impl StyleEngine<StyleSheet> for Style {
    fn get(&self, ctx: &StyleContext) -> StyleSheet {
        ctx.ui.style().into()
    }
}

impl StyleEngine<StyleSheet> for Arc<Style> {
    fn get(&self, ctx: &StyleContext) -> StyleSheet {
        ctx.ui.style().into()
    }
}

struct StyleEngineContainer<T>(Arc<dyn StyleEngine<T>>);

impl<T> Clone for StyleEngineContainer<T> {
    fn clone(&self) -> Self {
        StyleEngineContainer(self.0.clone())
    }
}
