use crate::renderer_adapter::{
    RenderContext, RenderDialect, RenderResult, RenderUnit, RendererAdapter,
};
use crate::{BindingReference, BindingTarget, PropValue, ScreenGraph, WidgetNode};
use serde_json::Value;
use std::collections::{BTreeMap, HashMap};

/// Renderer implementation that produces Flutter (Dart) source code from
/// Forge widget graphs.
pub struct FlutterRenderer;

impl RendererAdapter for FlutterRenderer {
    fn name(&self) -> &'static str {
        "flutter"
    }

    fn render_node(&self, node: &WidgetNode, ctx: &RenderContext<'_>) -> RenderResult {
        let code = render_widget(node, ctx.indent);
        let unit = RenderUnit::new(code).with_dependencies(self.dependencies());
        Ok(unit)
    }

    fn dependencies(&self) -> HashMap<String, String> {
        HashMap::from([(String::from("flutter"), String::from("sdk: flutter"))])
    }
}

/// Generates Dart code for a stateless widget from a [ScreenGraph].
pub fn generate_stateless_widget(screen: &ScreenGraph) -> String {
    let body = render_widget(&screen.root, 6);
    format!(
        "class {} extends StatelessWidget {{\n  const {}({{ super.key }});\n\n  @override\n  Widget build(BuildContext context) {{\n    return {}\n  }}\n}}\n",
        screen.id, screen.id, body
    )
}

/// Generates a Dart module containing all provided [ScreenGraph] widgets.
pub fn generate_dart_module(graphs: &[ScreenGraph]) -> String {
    let mut buffer = String::new();
    buffer.push_str("import 'package:flutter/widgets.dart';\n\n");
    for graph in graphs {
        buffer.push_str(&generate_stateless_widget(graph));
        buffer.push('\n');
    }
    buffer
}

fn render_widget(node: &WidgetNode, indent: usize) -> String {
    let mut buffer = String::new();
    buffer.push_str(&" ".repeat(indent));
    buffer.push_str(&node.widget);
    buffer.push_str("(\n");

    let props = render_props(&node.props, indent + 2);
    if !props.is_empty() {
        buffer.push_str(&props);
        if !props.ends_with('\n') {
            buffer.push_str(",\n");
        } else {
            buffer.push_str(",");
        }
        buffer.push('\n');
    }

    let children = render_children(&node.children, indent + 2);
    if !children.is_empty() {
        buffer.push_str(&children);
        buffer.push('\n');
    }

    buffer.push_str(&" ".repeat(indent));
    buffer.push_str(")");
    buffer
}

fn render_children(children: &[WidgetNode], indent: usize) -> String {
    if children.is_empty() {
        return String::new();
    }

    let mut rendered = String::new();
    rendered.push_str(&" ".repeat(indent));
    rendered.push_str("children: [\n");
    for child in children {
        rendered.push_str(&render_widget(child, indent + 2));
        rendered.push_str(",\n");
    }
    rendered.push_str(&" ".repeat(indent));
    rendered.push_str("]");
    rendered
}

fn render_props(props: &BTreeMap<String, PropValue>, indent: usize) -> String {
    if props.is_empty() {
        return String::new();
    }

    let mut rendered = String::new();
    for (key, value) in props {
        if !rendered.is_empty() {
            rendered.push_str(",\n");
        }
        rendered.push_str(&" ".repeat(indent));
        rendered.push_str(key);
        rendered.push_str(": ");
        rendered.push_str(&render_prop_value(value));
    }
    rendered
}

fn render_prop_value(value: &PropValue) -> String {
    match value {
        PropValue::Literal { value } => match value {
            Value::String(s) => format!("\"{}\"", s),
            Value::Bool(true) => "true".to_string(),
            Value::Bool(false) => "false".to_string(),
            Value::Null => "null".to_string(),
            Value::Number(num) => num.to_string(),
            other => serde_json::to_string(other).unwrap_or_default(),
        },
        PropValue::Expression { expression } => expression.clone(),
        PropValue::Binding { binding } => render_binding(binding),
    }
}

fn render_binding(binding: &BindingReference) -> String {
    if binding.target != BindingTarget::Provider {
        return String::from("<unsupported binding>");
    }

    let mut rendered = format!("ref.watch({})", binding.reference);
    if let Some(path) = &binding.path {
        rendered.push('.');
        rendered.push_str(path);
    }
    rendered
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{renderer_adapter::RenderOptions, state_adapter::RiverpodAdapter};

    #[test]
    fn renderer_emits_widget_invocation() {
        let node = WidgetNode {
            widget: "Text".to_string(),
            props: BTreeMap::from([(
                String::from("data"),
                PropValue::Literal {
                    value: Value::String("Hello".into()),
                },
            )]),
            children: Vec::new(),
        };
        let renderer = FlutterRenderer;
        let adapter = RiverpodAdapter::new();
        let options = RenderOptions {
            pretty: true,
            include_comments: false,
            dialect: RenderDialect::Dart,
        };
        let ctx = RenderContext::new(0, &adapter, &options);
        let unit = renderer.render_node(&node, &ctx).expect("render");
        assert!(unit.code.contains("Text"));
        assert!(unit.code.contains("\"Hello\""));
    }
}
