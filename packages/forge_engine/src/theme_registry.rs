use std::collections::HashMap;

use once_cell::sync::Lazy;
use std::sync::RwLock;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ThemeData {
    pub name: String,
    pub colors: HashMap<String, String>,
    pub typography: HashMap<String, String>,
    pub shapes: HashMap<String, String>,
}

impl ThemeData {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            colors: HashMap::new(),
            typography: HashMap::new(),
            shapes: HashMap::new(),
        }
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ThemeRegistryError {
    #[error("theme '{0}' is already registered")]
    AlreadyRegistered(String),
    #[error("theme '{0}' not found")]
    NotFound(String),
}

static THEMES: Lazy<RwLock<HashMap<String, ThemeData>>> = Lazy::new(|| RwLock::new(HashMap::new()));

pub fn register_theme(theme: ThemeData) -> Result<(), ThemeRegistryError> {
    let mut registry = THEMES.write().unwrap();
    let key = theme.name.clone();
    if registry.contains_key(&key) {
        return Err(ThemeRegistryError::AlreadyRegistered(key));
    }
    registry.insert(key, theme);
    Ok(())
}

pub fn get_theme(name: &str) -> Option<ThemeData> {
    THEMES.read().unwrap().get(name).cloned()
}

pub fn list_themes() -> Vec<ThemeData> {
    let registry = THEMES.read().unwrap();
    let mut themes: Vec<_> = registry.values().cloned().collect();
    themes.sort_by(|a, b| a.name.cmp(&b.name));
    themes
}

pub fn clear_themes() {
    THEMES.write().unwrap().clear();
}
