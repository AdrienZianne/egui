use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

use emath::TSTransform;
use epaint::{Color32, FontId, Margin, Shadow, Stroke, text::TextFormat};
use smallvec::SmallVec;
use strum_macros::{AsRefStr, EnumString};

use crate::{Context, Frame, Id, Response, Style, Ui, style::WidgetVisuals};

pub(crate) const CLASSES_SMALL_VEC_SIZE: usize = 5;

/// Small list of classes
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

/// Classes are divided in 2 : The built-in classes and the custom one
#[derive(Debug, Clone)]
pub enum Classe {
    Button,
    Custom(String),
}

/// Widget implementing this trait have classes, allowing better customization
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

/// Allow the use of the str instead of enum
impl From<&str> for Classe {
    #[inline]
    fn from(class: &str) -> Self {
        match class {
            "button" => Self::Button,
            _ => Self::Custom(class.to_owned()),
        }
    }
}

/// Modifier is the state of the widget, and the style that need to be used consequently
#[derive(Default, PartialEq, Eq, Debug)]
pub enum Modifiers {
    #[default]
    Default,
    Selected,
    Enabled,
    Disabled,
    Hover,
    Active,
    Open,
    Focus,
}

pub struct Properties {
    properties: HashMap<Classe, HashSet<Property>>,
}

impl Properties {
    fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }
}

// Enum of the properties available
#[derive(AsRefStr, EnumString)]
pub enum Property {
    Margin(Margin),
    Padding(Margin),
    Border(Stroke),
    Background(Color32),
    Transform(TSTransform),
}

pub struct StyleSheet {
    pub margin: Margin,
    pub padding: Margin,
    pub border: Stroke,
    pub background: Color32,
    pub transform: TSTransform,
}

pub trait StyleEngine<T>: Send + Sync {
    fn get(&self, ctx: &StyleContext<'_>) -> T;
}

fn style_id() -> Id {
    Id::new("style-id")
}

pub struct StyleContext<'b> {
    pub ui: &'b Ui,
    pub classes: &'b Classes,
    pub modifier: Modifiers,
}

impl Context {
    pub fn set_style_engine<T: 'static>(&self, engine: impl StyleEngine<T> + 'static) {
        self.data_mut(|d| {
            d.insert_temp(style_id(), StyleEngineContainer(Arc::new(engine)));
        });
    }
}

impl Ui {
    pub fn widget_style<T: 'static + From<StyleSheet>>(
        &self,
        ui: &Self,
        response: &Response,
        classes: &Classes,
    ) -> T {
        let engine: Option<StyleEngineContainer<T>> = self.data_mut(|d| d.get_temp(style_id()));

        let modifier = if !response.sense.interactive() {
            Modifiers::Disabled
        } else if response.is_pointer_button_down_on() || response.clicked() {
            Modifiers::Active
        } else if response.has_focus() {
            Modifiers::Focus
        } else if response.hovered() {
            Modifiers::Hover
        } else if response.highlighted() {
            Modifiers::Selected
        } else if response.opened() {
            Modifiers::Open
        } else {
            Modifiers::Default
        };

        if let Some(engine) = engine {
            println!("Engine found");
            engine.0.get(&StyleContext {
                ui,
                classes,
                modifier,
            })
        } else {
            println!("Engine not found");
            ui.style()
                .get(&StyleContext {
                    ui,
                    classes,
                    modifier,
                })
                .into()
        }
    }
}

/// Convert from Style to `StyleEngine`
impl StyleEngine<StyleSheet> for Style {
    fn get(&self, ctx: &StyleContext<'_>) -> StyleSheet {
        let visuals = ctx.ui.ctx().style();
        let modifiers_visuals = match ctx.modifier {
            Modifiers::Disabled => visuals.visuals.widgets.noninteractive,
            Modifiers::Hover => visuals.visuals.widgets.hovered,
            Modifiers::Active => visuals.visuals.widgets.active,
            Modifiers::Enabled | Modifiers::Open => visuals.visuals.widgets.open,
            Modifiers::Selected | Modifiers::Focus => WidgetVisuals {
                bg_fill: visuals.visuals.selection.bg_fill,
                bg_stroke: visuals.visuals.selection.stroke,
                weak_bg_fill: visuals.visuals.selection.bg_fill,
                ..visuals.visuals.widgets.inactive
            },
            Modifiers::Default => visuals.visuals.widgets.inactive,
        };
        StyleSheet {
            frame: Frame {
                corner_radius: modifiers_visuals.corner_radius,
                fill: modifiers_visuals.bg_fill,
                inner_margin: visuals.spacing.button_padding.into(),
                outer_margin: 0.into(),
                stroke: modifiers_visuals.bg_stroke,
                shadow: Shadow::NONE,
            },
            text: TextFormat::simple(
                ctx.ui
                    .style()
                    .override_font_id
                    .clone()
                    .unwrap_or(FontId::new(12.0, epaint::FontFamily::Proportional)),
                ctx.ui.visuals().text_color(),
            ),
        }
    }
}

struct StyleEngineContainer<T>(Arc<dyn StyleEngine<T>>);

impl<T> Clone for StyleEngineContainer<T> {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub trait StyleSheetT {
    fn set(&mut self, engine: impl StyleEngine);
}
