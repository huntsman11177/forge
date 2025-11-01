use std::collections::HashMap;

use serde_json::Value;

/// Handle referencing a theme definition that can be resolved at runtime.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct ThemeHandle {
    pub name: String,
}

impl ThemeHandle {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self { name: name.into() }
    }
}

/// Listener callback invoked whenever the evaluation context signals an update.
pub type ContextListener = Box<dyn FnMut() + Send + 'static>;

/// Shared evaluation context storing runtime state, providers, and theming.
#[derive(Debug, Default)]
pub struct RuntimeContext {
    theme: Option<ThemeHandle>,
    providers: HashMap<String, Value>,
    state: HashMap<String, Value>,
    listeners: Vec<ContextListener>,
}

impl RuntimeContext {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn theme(&self) -> Option<&ThemeHandle> {
        self.theme.as_ref()
    }

    pub fn set_theme(&mut self, theme: Option<ThemeHandle>) {
        self.theme = theme;
        self.trigger_update();
    }

    pub fn get_provider(&self, key: &str) -> Option<&Value> {
        self.providers.get(key)
    }

    pub fn set_provider<S: Into<String>>(&mut self, key: S, value: Value) {
        self.providers.insert(key.into(), value);
        self.trigger_update();
    }

    pub fn get_state(&self, key: &str) -> Option<&Value> {
        self.state.get(key)
    }

    pub fn set_state<S: Into<String>>(&mut self, key: S, value: Value) {
        self.state.insert(key.into(), value);
        self.trigger_update();
    }

    pub fn register_listener<F>(&mut self, listener: F)
    where
        F: FnMut() + Send + 'static,
    {
        self.listeners.push(Box::new(listener));
    }

    pub fn trigger_update(&mut self) {
        for listener in &mut self.listeners {
            listener();
        }
    }
}
