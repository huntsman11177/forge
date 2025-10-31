use crate::renderer_adapter::{RenderContext, RenderResult, RenderUnit, RendererAdapter};
use crate::{BindingReference, PropValue, WidgetNode};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

const ANGULAR_DEPENDENCIES: &[(&str, &str)] = &[("@angular/core", "^17.0.0")];

/// Renderer that produces Angular-compatible HTML templates from Forge widget graphs.
pub struct AngularRenderer;

impl RendererAdapter for AngularRenderer {
    fn name(&self) -> &'static str {
        "angular"
    }

    fn render_node(&self, node: &WidgetNode, ctx: &RenderContext<'_>) -> RenderResult {
        let mut unit = RenderUnit::new(render_element(node, ctx, ctx.indent));
        unit.dependencies = ANGULAR_DEPENDENCIES
            .iter()
            .map(|(name, version)| ((*name).to_string(), (*version).to_string()))
            .collect();
        Ok(unit)
    }

    fn dependencies(&self) -> HashMap<String, String> {
        ANGULAR_DEPENDENCIES
            .iter()
            .map(|(name, version)| (name.to_string(), version.to_string()))
            .collect()
    }
}

fn render_element(node: &WidgetNode, ctx: &RenderContext<'_>, indent: usize) -> String {
    let indent_str = " ".repeat(indent);
    let props = render_props(&node.props, ctx);

    if node.children.is_empty() {
        return format!("{indent_str}<{}{} />", node.widget, props);
    }

    let mut buffer = String::new();
    buffer.push_str(&format!("{indent_str}<{}{}>", node.widget, props));
    buffer.push('\n');

    for child in &node.children {
        buffer.push_str(&render_element(child, ctx, indent + 2));
        buffer.push('\n');
    }

    buffer.push_str(&indent_str);
    buffer.push_str(&format!("</{}>", node.widget));
    buffer
}

fn render_props(props: &BTreeMap<String, PropValue>, ctx: &RenderContext<'_>) -> String {
    if props.is_empty() {
        return String::new();
    }

    let mut rendered = String::new();
    for (key, value) in props {
        match value {
            PropValue::Literal { value } => {
                rendered.push(' ');
                rendered.push_str(key);
                rendered.push_str("=\"");
                rendered.push_str(&escape_attr(&literal_to_string(value)));
                rendered.push('"');
            }
            PropValue::Expression { expression } => {
                rendered.push(' ');
                rendered.push('[');
                rendered.push_str(key);
                rendered.push(']');
                rendered.push_str("=\"");
                rendered.push_str(&escape_attr(expression));
                rendered.push('"');
            }
            PropValue::Binding { binding } => {
                let expr = render_binding(binding, ctx);
                rendered.push(' ');
                rendered.push('[');
                rendered.push_str(key);
                rendered.push(']');
                rendered.push_str("=\"");
                rendered.push_str(&escape_attr(&expr));
                rendered.push('"');
            }
        }
    }

    rendered
}

fn literal_to_string(value: &Value) -> String {
    match value {
        Value::String(s) => s.clone(),
        Value::Bool(b) => b.to_string(),
        Value::Number(n) => n.to_string(),
        Value::Null => String::new(),
        other => serde_json::to_string(other).unwrap_or_default(),
    }
}

fn render_binding(binding: &BindingReference, ctx: &RenderContext<'_>) -> String {
    if let Some(resolved) = ctx.state_adapter.resolve(binding) {
        let mut expr = resolved.provider_id;
        if let Some(path) = resolved.path {
            if !path.is_empty() {
                expr.push('.');
                expr.push_str(&path);
            }
        }
        return expr;
    }

    let mut expr = binding.reference.clone();
    if let Some(path) = &binding.path {
        if !path.is_empty() {
            expr.push('.');
            expr.push_str(path);
        }
    }
    expr
}

fn escape_attr(value: &str) -> String {
    let mut escaped = String::new();
    for ch in value.chars() {
        match ch {
            '&' => escaped.push_str("&amp;"),
            '<' => escaped.push_str("&lt;"),
            '>' => escaped.push_str("&gt;"),
            '"' => escaped.push_str("&quot;"),
            '\'' => escaped.push_str("&#39;"),
            _ => escaped.push(ch),
        }
    }
    escaped
}
