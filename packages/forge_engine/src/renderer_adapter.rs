use crate::state_adapter::StateAdapter;
use crate::WidgetNode;
use serde::Serialize;
use std::collections::HashMap;

/// Rendering dialects supported by the engine.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RenderDialect {
    Dart,
    Jsx,
    Tsx,
    Html,
}

/// Rendering options shared across adapters.
#[derive(Debug, Clone)]
pub struct RenderOptions {
    pub pretty: bool,
    pub include_comments: bool,
    pub dialect: RenderDialect,
}

impl Default for RenderOptions {
    fn default() -> Self {
        Self {
            pretty: true,
            include_comments: false,
            dialect: RenderDialect::Dart,
        }
    }
}

/// Context supplied to renderer implementations.
pub struct RenderContext<'a> {
    pub indent: usize,
    pub state_adapter: &'a dyn StateAdapter,
    pub options: &'a RenderOptions,
}

impl<'a> RenderContext<'a> {
    pub fn new(
        indent: usize,
        state_adapter: &'a dyn StateAdapter,
        options: &'a RenderOptions,
    ) -> Self {
        Self {
            indent,
            state_adapter,
            options,
        }
    }

    /// Creates a new context with an updated indentation level.
    pub fn with_indent(&self, indent: usize) -> Self {
        Self {
            indent,
            state_adapter: self.state_adapter,
            options: self.options,
        }
    }
}

/// Structured rendering error returned by adapters.
#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub struct RenderError {
    pub node_id: Option<String>,
    pub message: String,
    pub severity: RenderSeverity,
}

impl RenderError {
    pub fn new<S: Into<String>>(message: S) -> Self {
        Self {
            node_id: None,
            message: message.into(),
            severity: RenderSeverity::Error,
        }
    }

    pub fn with_node_id<S: Into<String>>(mut self, node_id: S) -> Self {
        self.node_id = Some(node_id.into());
        self
    }

    pub fn with_severity(mut self, severity: RenderSeverity) -> Self {
        self.severity = severity;
        self
    }
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum RenderSeverity {
    Error,
    Warning,
}

/// Successful render output enriched with metadata.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderUnit {
    pub code: String,
    pub imports: Vec<String>,
    pub dependencies: HashMap<String, String>,
}

impl RenderUnit {
    pub fn new<S: Into<String>>(code: S) -> Self {
        Self {
            code: code.into(),
            imports: Vec::new(),
            dependencies: HashMap::new(),
        }
    }

    pub fn with_imports(mut self, imports: Vec<String>) -> Self {
        self.imports = imports;
        self
    }

    pub fn with_dependencies(mut self, deps: HashMap<String, String>) -> Self {
        self.dependencies = deps;
        self
    }
}

pub type RenderResult = Result<RenderUnit, RenderError>;

/// Defines the interface that all framework renderers must implement.
pub trait RendererAdapter: Send + Sync {
    /// Returns a stable identifier for the renderer (e.g. "flutter").
    fn name(&self) -> &'static str;

    /// Emits optional imports or prelude content.
    fn render_prelude(&self, _ctx: &RenderContext<'_>) -> String {
        String::new()
    }

    /// Emits optional postlude content (closing blocks, exports, etc.).
    fn render_postlude(&self, _ctx: &RenderContext<'_>) -> String {
        String::new()
    }

    /// Renders a Forge [WidgetNode] into target-language source code.
    fn render_node(&self, node: &WidgetNode, ctx: &RenderContext<'_>) -> RenderResult;

    /// Convenience wrapper that renders a complete tree with prelude/postlude.
    fn render_tree(&self, root: &WidgetNode, ctx: &RenderContext<'_>) -> RenderResult {
        let mut unit = self.render_node(root, ctx)?;
        let prelude = self.render_prelude(ctx);
        if !prelude.is_empty() {
            unit.code = format!("{}{}", prelude, unit.code);
        }
        let postlude = self.render_postlude(ctx);
        if !postlude.is_empty() {
            unit.code.push_str(&postlude);
        }
        Ok(unit)
    }

    /// Returns a map of framework-specific dependencies (name → version).
    fn dependencies(&self) -> HashMap<String, String> {
        HashMap::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::state_adapter::RiverpodAdapter;

    struct MockRenderer;

    impl RendererAdapter for MockRenderer {
        fn name(&self) -> &'static str {
            "mock"
        }

        fn render_node(&self, node: &WidgetNode, ctx: &RenderContext<'_>) -> RenderResult {
            let code = format!("{}{}", " ".repeat(ctx.indent), node.widget);
            Ok(RenderUnit::new(code))
        }
    }

    #[test]
    fn renderer_contract_produces_output() {
        let renderer = MockRenderer;
        let node = WidgetNode {
            widget: "Text".to_string(),
            props: Default::default(),
            children: Vec::new(),
        };
        let adapter = RiverpodAdapter::new();
        let options = RenderOptions::default();
        let ctx = RenderContext::new(2, &adapter, &options);

        let unit = renderer.render_tree(&node, &ctx).expect("render tree");
        assert!(!unit.code.trim().is_empty());
        assert!(unit.code.contains("Text"));
    }
}
