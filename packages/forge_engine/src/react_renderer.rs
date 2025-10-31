use crate::renderer_adapter::{
    RenderContext, RenderDialect, RenderResult, RenderUnit, RendererAdapter,
};
use crate::{BindingReference, PropValue, WidgetNode};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

const REACT_DEPENDENCIES: &[(&str, &str)] = &[("react", "^18.0.0"), ("react-dom", "^18.0.0")];

/// Renderer implementation that produces React (JSX) source code from Forge widget graphs.
pub struct ReactRenderer;

impl RendererAdapter for ReactRenderer {
    fn name(&self) -> &'static str {
        "react"
    }

    fn render_node(&self, node: &WidgetNode, ctx: &RenderContext<'_>) -> RenderResult {
        let mut unit = RenderUnit::new(render_element(node, ctx, ctx.indent));
        append_imports(&mut unit.imports, ctx.options.dialect);
        unit.dependencies = REACT_DEPENDENCIES
            .iter()
            .map(|(name, version)| ((*name).to_string(), (*version).to_string()))
            .collect();
        Ok(unit)
    }

    fn render_prelude(&self, ctx: &RenderContext<'_>) -> String {
        match ctx.options.dialect {
            RenderDialect::Jsx => "import React from 'react';\n".into(),
            RenderDialect::Tsx => "import * as React from 'react';\n".into(),
            _ => String::new(),
        }
    }

    fn dependencies(&self) -> HashMap<String, String> {
        REACT_DEPENDENCIES
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
        rendered.push(' ');
        rendered.push_str(key);
        rendered.push('=');

        match value {
            PropValue::Literal { value: literal } => {
                rendered.push_str(&render_literal_prop(literal));
            }
            _ => {
                rendered.push('{');
                rendered.push_str(&render_prop_value(value, ctx));
                rendered.push('}');
            }
        }
    }
    rendered
}

fn render_prop_value(value: &PropValue, ctx: &RenderContext<'_>) -> String {
    match value {
        PropValue::Literal { value } => serialize_literal(value),
        PropValue::Expression { expression } => expression.clone(),
        PropValue::Binding { binding } => render_binding(binding, ctx),
    }
}

fn serialize_literal(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", escape_attr(s)),
        Value::Bool(true) => "true".to_string(),
        Value::Bool(false) => "false".to_string(),
        Value::Null => "null".to_string(),
        Value::Number(num) => num.to_string(),
        other => serde_json::to_string(other).unwrap_or_else(|_| "null".into()),
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
    value.replace('"', "\\\"")
}

fn render_literal_prop(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", escape_attr(s)),
        Value::Bool(_) | Value::Number(_) | Value::Null => {
            format!("{{{}}}", serialize_literal(value))
        }
        Value::Array(_) | Value::Object(_) => {
            format!("{{{}}}", serialize_literal(value))
        }
    }
}

fn append_imports(imports: &mut Vec<String>, dialect: RenderDialect) {
    match dialect {
        RenderDialect::Jsx => imports.push("import React from 'react';".into()),
        RenderDialect::Tsx => imports.push("import * as React from 'react';".into()),
        _ => {}
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{renderer_adapter::RenderOptions, state_adapter::RiverpodAdapter};

    #[test]
    fn renderer_emits_jsx_element() {
        let node = WidgetNode {
            widget: "Button".to_string(),
            props: BTreeMap::from([(
                "text".to_string(),
                PropValue::Literal {
                    value: Value::String("Click Me".into()),
                },
            )]),
            children: Vec::new(),
        };
        let renderer = ReactRenderer;
        let adapter = RiverpodAdapter::new();
        let options = RenderOptions {
            pretty: true,
            include_comments: false,
            dialect: RenderDialect::Jsx,
        };
        let ctx = RenderContext::new(0, &adapter, &options);
        let unit = renderer.render_node(&node, &ctx).expect("render");
        assert!(unit.code.starts_with("<Button"));
        assert!(unit.code.contains("text=\"Click Me\""));
        assert!(unit.dependencies.contains_key("react"));
        assert!(unit
            .imports
            .iter()
            .any(|line| line.contains("import React")));
    }
}
