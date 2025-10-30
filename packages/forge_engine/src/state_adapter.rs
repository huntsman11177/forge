use crate::{BindingReference, BindingTarget};

/// Represents an adapter capable of resolving state bindings that appear inside
/// widget properties.
pub trait StateAdapter {
    /// The canonical name of the adapter (e.g. "riverpod").
    fn name(&self) -> &'static str;

    /// Returns `true` if this adapter understands the provided binding.
    fn can_resolve(&self, binding: &BindingReference) -> bool;

    /// Resolves a binding into a [`ResolvedBinding`] entry usable by the engine.
    fn resolve(&self, binding: &BindingReference) -> Option<ResolvedBinding>;
}

/// Concrete binding information returned by a [`StateAdapter`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedBinding {
    /// Identifier for the backing provider/state entry.
    pub provider_id: String,
    /// Optional nested path the binding drills into.
    pub path: Option<String>,
    /// Type hint supplied by the binding, if available.
    pub type_hint: Option<String>,
    /// Adapter that produced the resolution.
    pub adapter: &'static str,
}

/// Adapter implementation for Riverpod provider bindings detected by the parser.
#[derive(Debug, Default, Clone)]
pub struct RiverpodAdapter;

impl RiverpodAdapter {
    /// Creates a new Riverpod adapter instance.
    pub fn new() -> Self {
        Self {}
    }
}

impl StateAdapter for RiverpodAdapter {
    fn name(&self) -> &'static str {
        "riverpod"
    }

    fn can_resolve(&self, binding: &BindingReference) -> bool {
        if binding.target != BindingTarget::Provider {
            return false;
        }
        // Riverpod providers are typically named <identifier>Provider. Accept alphanumeric
        // identifiers that end with "Provider" to avoid matching arbitrary provider-like strings.
        let reference = binding.reference.trim();
        if reference.is_empty() {
            return false;
        }
        let valid_chars = reference
            .chars()
            .all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '.');
        valid_chars && reference.ends_with("Provider")
    }

    fn resolve(&self, binding: &BindingReference) -> Option<ResolvedBinding> {
        if !self.can_resolve(binding) {
            return None;
        }
        let provider_id = binding
            .provider_id
            .as_ref()
            .cloned()
            .unwrap_or_else(|| binding.reference.clone());
        Some(ResolvedBinding {
            provider_id,
            path: binding.path.clone(),
            type_hint: binding.type_hint.clone(),
            adapter: self.name(),
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::BindingReference;

    fn make_binding(reference: &str, path: Option<&str>) -> BindingReference {
        BindingReference {
            target: BindingTarget::Provider,
            reference: reference.to_string(),
            provider_id: Some(reference.to_string()),
            path: path.map(|p| p.to_string()),
            type_hint: None,
        }
    }

    #[test]
    fn riverpod_adapter_matches_provider_suffix() {
        let adapter = RiverpodAdapter::default();
        let binding = make_binding("balanceProvider", None);
        assert!(adapter.can_resolve(&binding));
        let resolved = adapter.resolve(&binding).expect("resolved binding");
        assert_eq!(resolved.provider_id, "balanceProvider");
        assert_eq!(resolved.path, None);
        assert_eq!(resolved.type_hint, None);
        assert_eq!(resolved.adapter, "riverpod");
    }

    #[test]
    fn riverpod_adapter_rejects_non_provider_target() {
        let adapter = RiverpodAdapter::default();
        let mut binding = make_binding("balanceProvider", None);
        binding.target = BindingTarget::Widget;
        assert!(!adapter.can_resolve(&binding));
        assert!(adapter.resolve(&binding).is_none());
    }

    #[test]
    fn riverpod_adapter_supports_paths() {
        let adapter = RiverpodAdapter::default();
        let binding = make_binding("userProvider", Some("state.name"));
        let mut binding = binding;
        binding.type_hint = Some("User".to_string());
        let resolved = adapter.resolve(&binding).expect("resolved binding");
        assert_eq!(resolved.path, Some("state.name".to_string()));
        assert_eq!(resolved.type_hint.as_deref(), Some("User"));
    }

    #[test]
    fn riverpod_adapter_ignores_malformed_identifiers() {
        let adapter = RiverpodAdapter::default();
        let binding = make_binding("invalid-provider", None);
        assert!(!adapter.can_resolve(&binding));
    }
}
