use epaint::Color32;
use smallvec::SmallVec;

use crate::{Response, Style, Ui, style::WidgetVisuals};

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
}

impl From<Style> for StyleSheet {
    fn from(value: Style) -> Self {
        StyleSheet {
            background: value.visuals.widgets.inactive.bg_fill,
        }
    }
}

pub trait WidgetStyle: From<StyleSheet> {}

impl Ui {
    pub fn widget_style<T: 'static>(&self, response: &Response, classes: &Classes) -> T {}
}
