use std::{
    any::Any,
    hash::{Hash as _, Hasher as _},
    sync::Arc,
};

use ahash::HashMap;
use epaint::mutex::Mutex;

use crate::{
    Style, Ui,
    widget_style::{Classes, StyleStruct, WidgetState},
};

/// A theme plugin that extend the egui style customization.
///
/// Theme plugins should not hold a reference to the [`Context`], since this would create a cycle
/// (which would prevent the [`Context`] from being dropped).
pub trait ThemePlugin: Send + Sync + std::any::Any + 'static {
    /// Theme name.
    fn debug_name(&self) -> &'static str;
}

/// A Theme plugin that implement a style computation for a defined `StyleStruct`
pub trait ThemeStyle<S>: ThemePlugin {
    /// The style according to the classes and state of the widget
    fn style(&self, classes: &Classes, state: WidgetState, base: &Style) -> S;
}

impl Ui {
    /// Access the installed theme plugin if there is one and fetch the requested widget style if it exist.
    /// Fallback to the default style if not found.
    ///
    /// Requested widget style must implement [`StyleStruct`].
    pub fn widget_style<S: StyleStruct + Clone + 'static>(
        &self,
        id: crate::Id,
        classes: &Classes,
    ) -> S {
        // If the requested `StyleStruct` is cached, return it without computing.
        // Otherwise proceed to compute the style from the widget information.

        // Fetch the current state of the widget
        let state = self
            .ctx()
            .read_response(id)
            .map(|r| r.widget_state())
            .unwrap_or_default();

        if let Some(style) = self.get_style::<S>(classes, state, self.style()) {
            style
        } else {
            S::default_style(classes, state, self.style())
        }
    }
}

#[derive(Default)]
pub(crate) struct Themes {
    /// The different theme plugin installed (Light, Dark, Custom,...)
    themes: HashMap<String, ThemeStyles>,

    /// The current theme
    current: Option<String>,

    /// The current theme cache
    /// TODO add classes and state !
    cache: HashMap<(std::any::TypeId, u64), Box<dyn Any + Send + Sync>>,
}

impl Themes {
    /// Register a theme and the style associated
    pub(crate) fn register<S>(
        &mut self,
        theme: Arc<Mutex<impl ThemeStyle<S> + Send + Sync + 'static>>,
    ) where
        S: StyleStruct + 'static,
    {
        let plugin_type = theme.lock().debug_name().to_owned();

        self.themes
            .entry(plugin_type.clone())
            .or_default()
            .add_style(theme);

        if self.current.is_none() {
            self.current = Some(plugin_type);
        }
    }

    /// Fetch the style of the current theme
    pub(crate) fn get<S: StyleStruct + Clone + 'static>(
        &mut self,
        classes: &Classes,
        state: WidgetState,
        base: &Style,
    ) -> Option<S> {
        let mut hasher = ahash::AHasher::default();
        classes.hash(&mut hasher);
        state.hash(&mut hasher);
        let hash = hasher.finish();

        let plugin_type = self.current.as_ref()?;
        let style_type = std::any::TypeId::of::<S>();
        if let Some(cached_style) = self.cache.get(&(style_type, hash)) {
            let x = cached_style.downcast_ref::<S>();
            return Some(x?.clone());
        }

        let style: S = self.themes.get(plugin_type)?.get(classes, state, base)?;

        self.cache
            .insert((style_type, hash), Box::new(style.clone()));
        Some(style)
    }

    pub fn set_current_theme(&mut self, plugin: &impl ToString) {
        let plugin = plugin.to_string();
        if self.themes.contains_key(&plugin) {
            self.current = Some(plugin);
            self.cache.clear();
        }
    }

    pub fn current_theme(&self) -> Option<String> {
        self.current.clone()
    }

    pub fn available_themes(&self) -> Vec<String> {
        self.themes.keys().cloned().collect()
    }

    pub fn invalidate_cache(&mut self) {
        self.cache.clear();
    }
}

#[derive(Default)]
struct ThemeStyles {
    styles: HashMap<std::any::TypeId, Arc<dyn DynThemeStyle>>,
}

impl ThemeStyles {
    fn add_style<S: StyleStruct + 'static>(
        &mut self,
        theme: Arc<Mutex<impl ThemeStyle<S> + Send + Sync + 'static>>,
    ) {
        let style_type = std::any::TypeId::of::<S>();

        if self.styles.contains_key(&style_type) {
            return;
        }

        self.styles.insert(
            style_type,
            Arc::new(Wrapper {
                inner: theme,
                _marker: std::marker::PhantomData,
            }),
        );
    }

    pub(crate) fn get<S: StyleStruct + 'static>(
        &self,
        classes: &Classes,
        state: WidgetState,
        base: &Style,
    ) -> Option<S> {
        let style_type = std::any::TypeId::of::<S>();

        let theme: &Arc<dyn DynThemeStyle> = self.styles.get(&style_type)?;
        let style = theme.call(classes, state, base);

        Some(*style.downcast::<S>().ok()?)
    }
}

/// Abstract the `ThemeStyle`
pub trait DynThemeStyle: Send + Sync {
    fn call(
        &self,
        classes: &Classes,
        state: WidgetState,
        base: &Style,
    ) -> Box<dyn Any + Send + Sync>;
}

struct Wrapper<T, S>
where
    T: ThemeStyle<S>,
    S: StyleStruct,
{
    inner: Arc<Mutex<T>>,
    _marker: std::marker::PhantomData<S>,
}

impl<T, S> DynThemeStyle for Wrapper<T, S>
where
    T: ThemeStyle<S> + Send + Sync + 'static,
    S: StyleStruct + 'static,
{
    fn call(
        &self,
        classes: &Classes,
        state: WidgetState,
        base: &Style,
    ) -> Box<dyn Any + Send + Sync> {
        let guard = self.inner.lock();
        let y = guard.style(classes, state, base);
        Box::new(y)
    }
}
