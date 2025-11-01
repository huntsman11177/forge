use std::collections::HashMap;

use once_cell::sync::Lazy;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// Describes a single widget property supported by a widget descriptor.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct PropDescriptor {
    pub name: String,
    pub required: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub ty: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub doc: Option<String>,
}

impl PropDescriptor {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            required: false,
            ty: None,
            doc: None,
        }
    }

    pub fn required(mut self, required: bool) -> Self {
        self.required = required;
        self
    }

    pub fn ty<S: Into<String>>(mut self, ty: S) -> Self {
        self.ty = Some(ty.into());
        self
    }

    pub fn doc<S: Into<String>>(mut self, doc: S) -> Self {
        self.doc = Some(doc.into());
        self
    }
}

/// Metadata describing a widget that can be rendered by Forge.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct WidgetDescriptor {
    pub name: String,
    #[serde(default)]
    pub props: Vec<PropDescriptor>,
    #[serde(default)]
    pub category: Option<String>,
    #[serde(default)]
    pub description: Option<String>,
}

impl WidgetDescriptor {
    pub fn new<S: Into<String>>(name: S) -> Self {
        Self {
            name: name.into(),
            props: Vec::new(),
            category: None,
            description: None,
        }
    }

    pub fn props(mut self, props: Vec<PropDescriptor>) -> Self {
        self.props = props;
        self
    }

    pub fn category<S: Into<String>>(mut self, category: S) -> Self {
        self.category = Some(category.into());
        self
    }

    pub fn description<S: Into<String>>(mut self, description: S) -> Self {
        self.description = Some(description.into());
        self
    }
}

#[derive(Debug, thiserror::Error)]
pub enum WidgetRegistryError {
    #[error("widget '{0}' is already registered")]
    AlreadyRegistered(String),
}

static REGISTRY: Lazy<RwLock<HashMap<String, WidgetDescriptor>>> = Lazy::new(|| {
    RwLock::new(HashMap::new())
});

/// Registers a widget descriptor with the global registry.
pub fn register_widget(descriptor: WidgetDescriptor) -> Result<(), WidgetRegistryError> {
    let mut registry = REGISTRY.write().unwrap();
    let key = descriptor.name.clone();
    if registry.contains_key(&key) {
        return Err(WidgetRegistryError::AlreadyRegistered(key));
    }
    registry.insert(key, descriptor);
    Ok(())
}

/// Returns a copy of an existing widget descriptor.
pub fn get_widget(name: &str) -> Option<WidgetDescriptor> {
    let registry = REGISTRY.read().unwrap();
    registry.get(name).cloned()
}

/// Lists all registered widget descriptors.
pub fn list_widgets() -> Vec<WidgetDescriptor> {
    let registry = REGISTRY.read().unwrap();
    let mut descriptors: Vec<_> = registry.values().cloned().collect();
    descriptors.sort_by(|a, b| a.name.cmp(&b.name));
    descriptors
}

/// Clears the registry. Intended for tests and should not be used at runtime.
pub fn clear_registry() {
    REGISTRY.write().unwrap().clear();
}
