use std::{
    collections::{HashMap, HashSet},
    hash::Hash,
    sync::Arc,
};

use emath::TSTransform;
use epaint::{Color32, Margin, Stroke};
use smallvec::SmallVec;
use strum_macros::{AsRefStr, EnumString};

use crate::{Context, Id, Response, Style, Ui, style::WidgetVisuals};

pub(crate) const CLASSES_SMALL_VEC_SIZE: usize = 5;

/// Small list of classes
#[derive(Debug, Clone, Default)]
pub struct Modifiers(pub SmallVec<[StyleModifier; CLASSES_SMALL_VEC_SIZE]>);

impl Modifiers {
    pub fn with_if(mut self, class: StyleModifier, condition: bool) -> Self {
        if condition {
            self.0.push(class);
        }
        self
    }

    pub fn add_if(&mut self, class: StyleModifier, condition: bool) {
        if condition {
            self.0.push(class);
        }
    }
}

/// Similar to selector in CSS
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum StyleModifier {
    // Type selector, for global change to widgets
    Button,
    Checkbox,
    Label,
    // Class selector, for custom change to some widgets
    Class(String),
    // Could add ID ?
}

/// From the proposition of #7586
#[derive(Default, Hash, PartialEq, Eq)]
pub enum WidgetState {
    /// This type of widget cannot be interacted with
    Noninteractive,

    /// An interactive widget that is not being interacted with²
    #[default]
    Inactive,

    /// An interactive widget that is being hovered
    Hovered,

    /// An interactive widget that is being clicked or dragged
    Active,
}

/// Allow the identification of the selectors and modifiers
#[derive(Hash, PartialEq, Eq)]
struct Selector {
    selector: StyleModifier,
    state: WidgetState,
}

/// The central struct
struct StyleSheet {
    styles: HashMap<Selector, HashSet<Property>>,
}

/// Widget implementing this trait have classes, allowing better customization
pub trait HasClasses {
    fn classes(&self) -> &Modifiers;

    fn classes_mut(&mut self) -> &mut Modifiers;

    fn add_class<T>(&mut self, class: T) -> &Self
    where
        T: Into<StyleModifier>,
    {
        self.classes_mut().add_if(class.into(), true);
        self
    }

    fn with_class<T>(mut self, class: T) -> Self
    where
        Self: Sized,
        T: Into<StyleModifier>,
    {
        self.classes_mut().add_if(class.into(), true);
        self
    }

    fn with_class_if<T>(mut self, class: T, condition: bool) -> Self
    where
        Self: Sized,
        T: Into<StyleModifier>,
    {
        self.classes_mut().add_if(class.into(), condition);
        self
    }
}

/// Allow the use of the str instead of enum
impl From<&str> for StyleModifier {
    #[inline]
    fn from(class: &str) -> Self {
        match class {
            "button" => Self::Button,
            "checkbox" => Self::Checkbox,
            "label" => Self::Label,
            _ => Self::Class(class.to_owned()),
        }
    }
}

// Allow the use of the str instead of enum
// impl From<&[str]> for Classes {
//     #[inline]
//     fn from(classes: &[Classe]) -> Self {
//         let mut c = Self::default();
//         for class in classes {
//             c.add_if(class.clone(), true);
//         }
//         c
//     }
//}

pub struct Properties {
    properties: HashMap<StyleModifier, HashSet<Property>>,
}

impl Properties {
    fn new() -> Self {
        Self {
            properties: HashMap::new(),
        }
    }
}

// Enum of the properties available
#[derive(AsRefStr, EnumString, PartialEq, Clone, Copy)]
pub enum Property {
    Margin(Margin),
    Padding(Margin),
    Border(Stroke),
    Background(Color32),
    Transform(TSTransform),
}

impl Eq for Property {}
impl Hash for Property {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
    }
}

pub trait StyleEngine: Send + Sync {
    fn get(&self, ctx: &StyleContext<'_>) -> StyleSheet;
}

fn style_id() -> Id {
    Id::new("style-id")
}

pub struct StyleContext<'b> {
    pub ui: &'b Ui,
    pub classes: &'b Modifiers,
    pub modifier: &'b Modifiers,
}

impl Context {
    pub fn set_style_engine(&self, engine: impl StyleEngine + 'static) {
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
        classes: &Modifiers,
    ) -> T {
        let engine: Option<StyleEngineContainer> = self.data_mut(|d| d.get_temp(style_id()));

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
            engine
                .0
                .get(&StyleContext {
                    ui,
                    classes,
                    modifier: &modifier,
                })
                .into()
        } else {
            ui.style()
                .get(&StyleContext {
                    ui,
                    classes,
                    modifier: &modifier,
                })
                .into()
        }
    }
}

/// Convert from Style to `StyleEngine`
impl StyleEngine for Style {
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
    }
}

struct StyleEngineContainer(Arc<dyn StyleEngine>);

impl Clone for StyleEngineContainer {
    fn clone(&self) -> Self {
        Self(self.0.clone())
    }
}

pub trait StyleSheetT {
    fn set(&mut self, engine: impl StyleEngine);
}
