mod analyzer_service;
mod expr;
mod flutter_renderer;
mod logic_engine;
mod logic_types;
mod merge_engine;
mod plugin_registry;
mod plugin_sandbox;
mod renderer_adapter;
mod state_adapter;

/// Semantic version for the analysis report JSON contract emitted by the CLI.
pub const ANALYSIS_REPORT_VERSION: &str = "1.0.0";

pub use analyzer_service::{
    AnalysisDecision, AnalysisOutcome, AnalysisStrategy, AnalyzerInvocation, AnalyzerService,
};
pub use expr::{
    eval_expression, parse_expression, BinaryOp, EvalContext, Expr, ExprError, ExprResult, UnaryOp,
};
pub use flutter_renderer::{generate_dart_module, generate_stateless_widget, FlutterRenderer};
pub use logic_engine::{simulate_flow, EvalConfig, LogicError};
pub use logic_types::{
    BuiltinLogicNodeKind, EvalResult, EvalTraceEntry, ExprValue, Flow, LogicEdge, LogicGraph,
    LogicNode,
};
pub use merge_engine::{merge_screen_graphs, MergeConflict, MergeOutcome};
pub use plugin_registry::{PluginDescriptor, PluginRegistry, PluginRegistryError};
pub use plugin_sandbox::{PluginSandbox, SandboxError};
pub use renderer_adapter::{RenderContext, RenderOptions, RendererAdapter};
pub use state_adapter::{ResolvedBinding, RiverpodAdapter, StateAdapter};

use once_cell::sync::Lazy;
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::{
    collections::{BTreeMap, HashSet},
    fs,
    path::Path,
};
use thiserror::Error;
use walkdir::WalkDir;

fn skip_generic_block(input: &str) -> Option<&str> {
    let mut depth = 0_i32;
    let mut chars = input.char_indices();
    let (_, first) = chars.next()?;
    if first != '<' {
        return None;
    }
    depth += 1;

    while let Some((idx, ch)) = chars.next() {
        match ch {
            '<' => depth += 1,
            '>' => {
                depth -= 1;
                if depth == 0 {
                    return Some(&input[idx + 1..]);
                }
            }
            _ => {}
        }
    }
    None
}

fn extract_parenthesized_block(input: &str) -> Option<(String, &str)> {
    let mut depth = 0_i32;
    let mut result = String::new();
    let mut chars = input.char_indices();
    let (_, first) = chars.next()?;
    if first != '(' {
        return None;
    }
    depth += 1;
    while let Some((idx, ch)) = chars.next() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    return Some((result, &input[idx + 1..]));
                }
            }
            _ => {}
        }
        result.push(ch);
    }
    None
}

fn split_widget_signature(input: &str) -> Option<(&str, &str)> {
    let mut chars = input.chars().enumerate();
    let mut end_idx = None;
    while let Some((idx, ch)) = chars.next() {
        if ch == '(' {
            end_idx = Some(idx);
            break;
        }
    }
    let idx = end_idx?;
    let widget_part = input[..idx].trim();
    let widget = widget_part.split_whitespace().last()?;
    let rest = &input[idx..];
    Some((widget, rest))
}

fn render_prop_value(value: &PropValue) -> String {
    match value {
        PropValue::Literal { value } => match value {
            serde_json::Value::String(s) => format!("\"{}\"", s),
            serde_json::Value::Bool(true) => "true".to_string(),
            serde_json::Value::Bool(false) => "false".to_string(),
            serde_json::Value::Null => "null".to_string(),
            serde_json::Value::Number(num) => num.to_string(),
            other => serde_json::to_string(other).unwrap_or_default(),
        },
        PropValue::Expression { expression } => expression.clone(),
        PropValue::Binding { binding } => {
            let mut rendered = format!("ref.watch({})", binding.reference);
            if let Some(path) = &binding.path {
                rendered.push('.');
                rendered.push_str(path);
            }
            rendered
        }
    }
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum PropValue {
    Literal { value: serde_json::Value },
    Expression { expression: String },
    Binding { binding: BindingReference },
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
#[serde(rename_all = "lowercase")]
pub enum BindingTarget {
    Provider,
    Widget,
    Logic,
    External,
}

#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct BindingReference {
    #[serde(rename = "type")]
    pub target: BindingTarget,
    #[serde(rename = "ref")]
    pub reference: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub provider_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub type_hint: Option<String>,
}

/// Represents a widget tree captured by the Forge graph format.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct WidgetNode {
    pub widget: String,
    #[serde(default)]
    pub props: BTreeMap<String, PropValue>,
    #[serde(default)]
    pub children: Vec<WidgetNode>,
}

/// Top-level graph describing a Flutter screen.
#[derive(Debug, Serialize, Deserialize, PartialEq, Eq, Clone)]
pub struct ScreenGraph {
    pub id: String,
    pub root: WidgetNode,
}

/// Minimal logical representation of a Flutter screen discovered in Dart code.
#[derive(Debug, PartialEq, Eq)]
pub struct ParsedScreen {
    pub name: String,
    pub body: String,
}

/// Errors that can occur when interacting with the engine.
#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Workspace not found at {0}")]
    WorkspaceNotFound(String),

    #[error("Failed to read file {path}: {source}")]
    IoError {
        path: String,
        #[source]
        source: std::io::Error,
    },

    #[error("Failed to parse JSON for {path}: {source}")]
    JsonError {
        path: String,
        #[source]
        source: serde_json::Error,
    },
}

/// Discovers Dart files within a Flutter workspace.
pub fn discover_dart_files<P: AsRef<Path>>(workspace_root: P) -> Result<Vec<String>, EngineError> {
    let root = workspace_root.as_ref();
    if !root.exists() {
        return Err(EngineError::WorkspaceNotFound(root.display().to_string()));
    }

    let mut files = Vec::new();
    let mut visited = HashSet::new();

    for entry in WalkDir::new(root) {
        let entry = match entry {
            Ok(entry) => entry,
            Err(err) => {
                return Err(EngineError::IoError {
                    path: root.display().to_string(),
                    source: std::io::Error::new(std::io::ErrorKind::Other, err),
                });
            }
        };

        if entry.file_type().is_file()
            && entry
                .path()
                .extension()
                .map(|ext| ext == "dart")
                .unwrap_or(false)
        {
            let canonical = match entry.path().canonicalize() {
                Ok(path) => path,
                Err(source) => {
                    return Err(EngineError::IoError {
                        path: entry.path().display().to_string(),
                        source,
                    });
                }
            };

            if visited.insert(canonical.clone()) {
                files.push(canonical.display().to_string());
            }
        }
    }

    Ok(files)
}

/// Reads and parses a JSON graph file into a [ScreenGraph].
pub fn read_graph<P: AsRef<Path>>(path: P) -> Result<ScreenGraph, EngineError> {
    let path_ref = path.as_ref();
    let contents = fs::read_to_string(path_ref).map_err(|source| EngineError::IoError {
        path: path_ref.display().to_string(),
        source,
    })?;

    let graph = serde_json::from_str(&contents).map_err(|source| EngineError::JsonError {
        path: path_ref.display().to_string(),
        source,
    })?;

    Ok(graph)
}

const SINGLE_CHILD_PROPS: &[&str] = &["child", "body", "appBar", "floatingActionButton"];

static CLASS_REGEX: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"(?s)class\s+(?P<name>[A-Za-z0-9_]+)\s+extends\s+StatelessWidget\s*\{.*?Widget\s+build\s*\([^)]*\)\s*\{(?P<body>.*?)\n\s*\}\s*\}")
        .expect("stateless regex")
});

/// Parses Dart source and returns simple [ParsedScreen] representations of
/// stateless widgets.
pub fn parse_stateless_screens(source: &str) -> Vec<ParsedScreen> {
    CLASS_REGEX
        .captures_iter(source)
        .map(|caps| ParsedScreen {
            name: caps["name"].to_string(),
            body: caps["body"].to_string(),
        })
        .collect()
}

/// Converts the body of a `build` method into a [WidgetNode]. This parser is
/// intentionally conservative and only supports nested constructor expressions
/// with `children` or inline widget instances.
pub fn parse_widget_tree(body: &str) -> Option<WidgetNode> {
    let mut trimmed = body
        .trim()
        .trim_end_matches(';')
        .trim_end_matches(',')
        .trim();
    if let Some(stripped) = trimmed.strip_prefix("return") {
        trimmed = stripped.trim_start();
    }
    if trimmed.is_empty() {
        return None;
    }

    let (widget, rest) = split_widget_signature(trimmed)?;
    let (inner, _) = extract_parenthesized_block(rest)?;
    let inner = inner.trim();

    let mut props: BTreeMap<String, PropValue> = BTreeMap::new();
    let mut children = Vec::new();

    if let Some(raw_children) = extract_children_block(inner) {
        for child_src in split_children(raw_children) {
            if let Some(child_node) = parse_widget_tree(child_src) {
                children.push(child_node);
            }
        }
    } else {
        for child_expr in extract_single_child_expressions(inner, SINGLE_CHILD_PROPS) {
            if let Some(child_node) = parse_widget_tree(child_expr) {
                children.push(child_node);
            }
        }
    }

    let mut positional_index = 0;
    for entry in split_top_level_entries(inner) {
        if let Some((key, value)) = split_key_value(entry) {
            if key == "children" || SINGLE_CHILD_PROPS.iter().any(|candidate| candidate == &key) {
                continue;
            }
            props.insert(key.to_string(), parse_prop_value(value.trim()));
        } else {
            if let Some(name) = map_positional_prop(widget, positional_index) {
                props.insert(name.to_string(), parse_prop_value(entry.trim()));
            } else if !entry.trim().is_empty() {
                let fallback = format!("positional{positional_index}");
                props.insert(fallback, parse_prop_value(entry.trim()));
            }
            positional_index += 1;
        }
    }

    Some(WidgetNode {
        widget: widget.to_string(),
        props,
        children,
    })
}

fn extract_children_block(inner: &str) -> Option<&str> {
    let mut chars = inner.char_indices();
    while let Some((idx, ch)) = chars.next() {
        if ch == 'c' && inner[idx..].starts_with("children") {
            let rest = &inner[idx + "children".len()..];
            let mut rest = rest.trim_start();
            if !rest.starts_with(':') {
                continue;
            }
            rest = rest[1..].trim_start();
            if let Some(stripped) = rest.strip_prefix("const") {
                rest = stripped.trim_start();
            }
            if rest.starts_with('<') {
                if let Some(after_generic) = skip_generic_block(rest) {
                    rest = after_generic.trim_start();
                } else {
                    continue;
                }
            }
            if !rest.starts_with('[') {
                continue;
            }
            return extract_bracket_block(rest);
        }
    }
    None
}

fn extract_single_child_expressions<'a>(inner: &'a str, keys: &[&str]) -> Vec<&'a str> {
    let mut results = Vec::new();
    let mut index = 0;
    while index < inner.len() {
        if let Some((expr, consumed)) = match_single_child_prop(inner, index, keys) {
            results.push(expr);
            index += consumed;
        } else {
            index += 1;
        }
    }
    results
}

fn match_single_child_prop<'a>(
    inner: &'a str,
    start: usize,
    keys: &[&str],
) -> Option<(&'a str, usize)> {
    let slice = &inner[start..];
    for key in keys {
        if !slice.starts_with(key) {
            continue;
        }
        // Ensure we are matching the property name boundary.
        if slice[key.len()..]
            .chars()
            .next()
            .map(|c| c.is_alphanumeric() || c == '_')
            == Some(true)
        {
            continue;
        }
        if start > 0 {
            if let Some(prev) = inner[..start].chars().rev().find(|c| !c.is_whitespace()) {
                if prev.is_alphanumeric() || prev == '_' {
                    continue;
                }
            }
        }

        let after_key = &slice[key.len()..];
        let trimmed_after_key = after_key.trim_start();
        let mut offset = key.len() + (after_key.len() - trimmed_after_key.len());
        if !trimmed_after_key.starts_with(':') {
            continue;
        }
        offset += 1; // consume ':'
        let after_colon = &trimmed_after_key[1..];
        let trimmed_after_colon = after_colon.trim_start();
        offset += after_colon.len() - trimmed_after_colon.len();

        let mut expr_slice = trimmed_after_colon;
        let mut consumed = offset;
        if expr_slice.starts_with("const")
            && expr_slice
                .chars()
                .nth("const".len())
                .map(|c| c.is_alphanumeric() || c == '_')
                != Some(true)
        {
            expr_slice = &expr_slice["const".len()..];
            let trimmed_after_const = expr_slice.trim_start();
            consumed += "const".len() + (expr_slice.len() - trimmed_after_const.len());
            expr_slice = trimmed_after_const;
        }

        if let Some((expr, expr_len)) = take_widget_expression(expr_slice) {
            consumed += expr_len;
            return Some((expr, consumed));
        }
    }
    None
}

fn extract_bracket_block(input: &str) -> Option<&str> {
    let mut depth = 0_i32;
    let mut start_content = None;
    for (idx, ch) in input.char_indices() {
        match ch {
            '[' => {
                depth += 1;
                if depth == 1 {
                    start_content = Some(idx + ch.len_utf8());
                }
            }
            ']' => {
                depth -= 1;
                if depth == 0 {
                    let start = start_content?;
                    return Some(input[start..idx].trim());
                }
            }
            _ => {}
        }
    }
    None
}

fn take_widget_expression(input: &str) -> Option<(&str, usize)> {
    let (_, rest) = split_widget_signature(input)?;
    let (_, remaining) = extract_parenthesized_block(rest)?;
    let consumed = input.len() - remaining.len();
    let expr = input[..consumed].trim_end_matches(',').trim();
    if expr.is_empty() {
        None
    } else {
        Some((expr, consumed))
    }
}

fn split_top_level_entries(input: &str) -> Vec<&str> {
    let mut entries = Vec::new();
    let mut paren = 0_i32;
    let mut bracket = 0_i32;
    let mut brace = 0_i32;
    let mut angle = 0_i32;
    let mut start = 0;
    for (idx, ch) in input.char_indices() {
        match ch {
            '(' => paren += 1,
            ')' => paren -= 1,
            '[' => bracket += 1,
            ']' => bracket -= 1,
            '{' => brace += 1,
            '}' => brace -= 1,
            '<' => angle += 1,
            '>' => {
                if angle > 0 {
                    angle -= 1;
                }
            }
            ',' if paren == 0 && bracket == 0 && brace == 0 && angle == 0 => {
                let segment = input[start..idx].trim();
                if !segment.is_empty() {
                    entries.push(segment);
                }
                start = idx + 1;
            }
            _ => {}
        }
    }
    if start < input.len() {
        let segment = input[start..].trim();
        if !segment.is_empty() {
            entries.push(segment);
        }
    }
    entries
}

fn split_key_value(entry: &str) -> Option<(&str, &str)> {
    let mut paren = 0_i32;
    let mut bracket = 0_i32;
    let mut brace = 0_i32;
    let mut angle = 0_i32;
    let mut single_quote = false;
    let mut double_quote = false;
    let mut escape = false;
    for (idx, ch) in entry.char_indices() {
        if escape {
            escape = false;
            continue;
        }

        match ch {
            '\\' if single_quote || double_quote => {
                escape = true;
            }
            '\'' if !double_quote => {
                single_quote = !single_quote;
            }
            '"' if !single_quote => {
                double_quote = !double_quote;
            }
            '(' if !single_quote && !double_quote => paren += 1,
            ')' if !single_quote && !double_quote => paren -= 1,
            '[' if !single_quote && !double_quote => bracket += 1,
            ']' if !single_quote && !double_quote => bracket -= 1,
            '{' if !single_quote && !double_quote => brace += 1,
            '}' if !single_quote && !double_quote => brace -= 1,
            '<' if !single_quote && !double_quote => angle += 1,
            '>' if !single_quote && !double_quote => {
                if angle > 0 {
                    angle -= 1;
                }
            }
            ':' if !single_quote
                && !double_quote
                && paren == 0
                && bracket == 0
                && brace == 0
                && angle == 0 =>
            {
                let key = entry[..idx].trim();
                if key.is_empty() {
                    return None;
                }
                let value = entry[idx + 1..].trim();
                return Some((key, value));
            }
            _ => {}
        }
    }
    None
}

fn parse_prop_value(raw: &str) -> PropValue {
    if let Some(binding) = parse_binding(raw) {
        return binding;
    }
    if let Some(literal) = parse_literal(raw) {
        return literal;
    }
    PropValue::Expression {
        expression: raw.to_string(),
    }
}

fn parse_binding(raw: &str) -> Option<PropValue> {
    let trimmed = raw.trim();
    const PREFIX: &str = "ref.watch(";
    if !trimmed.starts_with(PREFIX) {
        return None;
    }

    let after_prefix = &trimmed[PREFIX.len()..];
    let mut depth = 1_i32;
    let mut closing_rel = None;
    for (idx, ch) in after_prefix.char_indices() {
        match ch {
            '(' => depth += 1,
            ')' => {
                depth -= 1;
                if depth == 0 {
                    closing_rel = Some(idx);
                    break;
                }
            }
            _ => {}
        }
    }

    let closing_rel = closing_rel?;
    let provider_expr = after_prefix[..closing_rel].trim();
    if provider_expr.is_empty() {
        return None;
    }

    let mut remainder = after_prefix[closing_rel + 1..].trim();
    let mut path: Option<String> = None;

    if remainder.starts_with('.') {
        let mut idx = 1; // skip the dot
        while idx < remainder.len() {
            let ch = remainder[idx..].chars().next().unwrap();
            if ch.is_alphanumeric() || ch == '_' || ch == '.' {
                idx += ch.len_utf8();
            } else {
                break;
            }
        }

        let candidate = &remainder[1..idx];
        if candidate.is_empty()
            || candidate.ends_with('.')
            || candidate.split('.').any(|segment| {
                segment.is_empty()
                    || !segment.chars().enumerate().all(|(i, c)| {
                        if i == 0 {
                            c.is_alphabetic() || c == '_'
                        } else {
                            c.is_alphanumeric() || c == '_'
                        }
                    })
            })
        {
            return None;
        }

        path = Some(candidate.to_string());
        remainder = remainder[idx..].trim();
    }

    if !remainder.is_empty() {
        return None;
    }

    let reference = provider_expr.to_string();
    let provider_id = extract_provider_identifier(provider_expr);

    Some(PropValue::Binding {
        binding: BindingReference {
            target: BindingTarget::Provider,
            reference,
            provider_id,
            path,
            type_hint: None,
        },
    })
}

fn extract_provider_identifier(reference: &str) -> Option<String> {
    let trimmed = reference.trim();
    if trimmed.is_empty() {
        return None;
    }
    let valid_identifier = trimmed
        .chars()
        .all(|ch| ch.is_alphanumeric() || ch == '_' || ch == '.');
    if valid_identifier {
        Some(trimmed.to_string())
    } else {
        None
    }
}

fn parse_literal(raw: &str) -> Option<PropValue> {
    let trimmed = raw.trim();
    if trimmed.len() >= 2
        && ((trimmed.starts_with('"') && trimmed.ends_with('"'))
            || (trimmed.starts_with('\'') && trimmed.ends_with('\'')))
    {
        let inner = &trimmed[1..trimmed.len() - 1];
        return Some(PropValue::Literal {
            value: serde_json::Value::String(inner.to_string()),
        });
    }

    if trimmed.eq("true") {
        return Some(PropValue::Literal {
            value: serde_json::Value::Bool(true),
        });
    }
    if trimmed.eq("false") {
        return Some(PropValue::Literal {
            value: serde_json::Value::Bool(false),
        });
    }
    if trimmed.eq("null") {
        return Some(PropValue::Literal {
            value: serde_json::Value::Null,
        });
    }

    if let Ok(int_val) = trimmed.parse::<i64>() {
        return Some(PropValue::Literal {
            value: serde_json::Value::Number(int_val.into()),
        });
    }
    if let Ok(float_val) = trimmed.parse::<f64>() {
        if let Some(num) = serde_json::Number::from_f64(float_val) {
            return Some(PropValue::Literal {
                value: serde_json::Value::Number(num),
            });
        }
    }

    None
}

fn map_positional_prop(widget: &str, index: usize) -> Option<&'static str> {
    match widget {
        "Text" => match index {
            0 => Some("data"),
            _ => None,
        },
        "Icon" => match index {
            0 => Some("icon"),
            _ => None,
        },
        "Image.network" | "Image" => match index {
            0 => Some("src"),
            _ => None,
        },
        _ => None,
    }
}

fn split_children(src: &str) -> Vec<&str> {
    let mut segments = Vec::new();
    let mut depth = 0_i32;
    let mut start = 0;
    let bytes = src.as_bytes();
    for (idx, ch) in bytes.iter().enumerate() {
        match ch {
            b'(' | b'[' | b'{' => depth += 1,
            b')' | b']' | b'}' => depth -= 1,
            b',' if depth == 0 => {
                segments.push(src[start..idx].trim());
                start = idx + 1;
            }
            _ => {}
        }
    }
    if start < bytes.len() {
        segments.push(src[start..].trim());
    }
    segments.into_iter().filter(|s| !s.is_empty()).collect()
}

/// Parses Dart source and returns [ScreenGraph] entries for each stateless
/// widget discovered. Only the top-level widget returned by `build` is
/// considered.
pub fn build_graphs_from_source(source: &str) -> Vec<ScreenGraph> {
    parse_stateless_screens(source)
        .into_iter()
        .filter_map(|screen| parse_widget_tree(&screen.body).map(|root| (screen, root)))
        .map(|(screen, root)| ScreenGraph {
            id: screen.name,
            root,
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::Value;

    fn assert_prop_literal(node: &WidgetNode, key: &str, expected: &str) {
        match node
            .props
            .get(key)
            .unwrap_or_else(|| panic!("missing prop {key}"))
        {
            PropValue::Literal { value } => match value {
                Value::String(s) => assert_eq!(s, expected),
                Value::Bool(b) => assert_eq!(expected, if *b { "true" } else { "false" }),
                Value::Number(n) => assert_eq!(n.to_string(), expected),
                other => panic!("unexpected literal {:?}", other),
            },
            PropValue::Expression { expression } => assert_eq!(expression, expected),
            other => panic!("unexpected prop variant {:?}", other),
        }
    }

    fn assert_prop_binding(node: &WidgetNode, key: &str, provider: &str, path: Option<&str>) {
        match node
            .props
            .get(key)
            .unwrap_or_else(|| panic!("missing prop {key}"))
        {
            PropValue::Binding { binding } => {
                assert_eq!(binding.target, BindingTarget::Provider);
                assert_eq!(binding.reference, provider);
                assert_eq!(binding.provider_id.as_deref(), Some(provider));
                assert_eq!(binding.path.as_deref(), path);
                assert_eq!(binding.type_hint, None);
            }
            other => panic!("expected binding prop, found {:?}", other),
        }
    }

    #[test]
    fn discover_dart_files_returns_unique_paths() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_a = temp_dir.path().join("lib/main.dart");
        let file_b = temp_dir.path().join("lib/src/app.dart");
        std::fs::create_dir_all(file_a.parent().unwrap()).unwrap();
        std::fs::create_dir_all(file_b.parent().unwrap()).unwrap();
        std::fs::write(&file_a, "void main() {}").unwrap();
        std::fs::write(&file_b, "void mainApp() {}").unwrap();

        let mut result = discover_dart_files(temp_dir.path()).unwrap();
        result.sort();
        assert_eq!(result.len(), 2);
        assert!(
            Path::new(&result[0]).ends_with(Path::new("lib").join("main.dart")),
            "unexpected first path: {}",
            result[0]
        );
        assert!(
            Path::new(&result[1]).ends_with(Path::new("lib").join("src").join("app.dart")),
            "unexpected second path: {}",
            result[1]
        );
    }

    #[test]
    fn read_graph_parses_valid_json() {
        let temp_dir = tempfile::tempdir().unwrap();
        let graph_path = temp_dir.path().join("graph.json");
        let sample_graph = serde_json::json!({
            "id": "screen_home",
            "root": {
                "widget": "Scaffold",
                "props": {
                    "title": { "type": "literal", "value": "Home" }
                },
                "children": [
                    {
                        "widget": "Text",
                        "props": {
                            "data": { "type": "literal", "value": "Hello" }
                        },
                        "children": []
                    }
                ]
            }
        });
        std::fs::write(&graph_path, sample_graph.to_string()).unwrap();

        let graph = read_graph(&graph_path).unwrap();
        assert_eq!(graph.id, "screen_home");
        assert_eq!(graph.root.widget, "Scaffold");
        assert_eq!(graph.root.children.len(), 1);
    }

    #[test]
    fn parse_stateless_widget_extracts_screen() {
        let source = r#"
import 'package:flutter/widgets.dart';

class HomeScreen extends StatelessWidget {
  const HomeScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Home')),
      body: Column(
        children: [
          const Text('Hello'),
          ElevatedButton(
            onPressed: null,
            child: const Text('Tap'),
          ),
        ],
      ),
    );
  }
}
"#;

        let graphs = build_graphs_from_source(source);
        assert_eq!(graphs.len(), 1);
        let graph = &graphs[0];
        assert_eq!(graph.id, "HomeScreen");
        assert_eq!(graph.root.widget, "Scaffold");
        assert_eq!(graph.root.children.len(), 2);
        assert_eq!(graph.root.children[0].widget, "Text");
    }

    #[test]
    fn parse_widget_tree_parses_children() {
        let source = r#"Column(
      children: [
        Text('One'),
        Row(children: [Text('Nested')])
      ],
    )"#;

        let node = parse_widget_tree(source).expect("column node");
        assert_eq!(node.widget, "Column");
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].widget, "Text");
        assert_eq!(node.children[1].widget, "Row");
    }

    #[test]
    fn parse_widget_tree_parses_child_property() {
        let source = r#"Container(
      child: Padding(
        child: Text('Nested'),
      ),
    )"#;

        let node = parse_widget_tree(source).expect("container node");
        assert_eq!(node.widget, "Container");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].widget, "Padding");
        assert_eq!(node.children[0].children.len(), 1);
        assert_eq!(node.children[0].children[0].widget, "Text");
    }

    #[test]
    fn parse_children_allows_const_lists() {
        let source = r#"Column(
      children: const [
        Text('One'),
        Text('Two'),
      ],
    )"#;

        let node = parse_widget_tree(source).expect("column node");
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].widget, "Text");
        assert_eq!(node.children[1].widget, "Text");
    }

    #[test]
    fn parse_padding_with_child_and_props() {
        let source = r#"Padding(
      padding: const EdgeInsets.symmetric(horizontal: 16, vertical: 8),
      child: Container(width: 100, height: 50),
    )"#;

        let node = parse_widget_tree(source).expect("padding node");
        assert_eq!(node.widget, "Padding");
        assert!(node.props.contains_key("padding"));
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].widget, "Container");
    }

    #[test]
    fn parse_scaffold_single_children() {
        let source = r#"Scaffold(
      appBar: AppBar(title: const Text('Title')),
      body: Center(child: Text('Body')),
    )"#;

        let node = parse_widget_tree(source).expect("scaffold node");
        assert_eq!(node.widget, "Scaffold");
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].widget, "AppBar");
        assert_eq!(node.children[1].widget, "Center");
    }

    #[test]
    fn parse_list_view_children_with_typed_list() {
        let source = r#"ListView(
      children: <Widget>[
        Text('One'),
        const SizedBox(height: 8),
      ],
    )"#;

        let node = parse_widget_tree(source).expect("listview node");
        assert_eq!(node.widget, "ListView");
        assert_eq!(node.children.len(), 2);
        assert_eq!(node.children[0].widget, "Text");
        assert_eq!(node.children[1].widget, "SizedBox");
    }

    #[test]
    fn parse_stack_with_positioned_children() {
        let source = r#"Stack(
      children: [
        Positioned(
          left: 0,
          top: 0,
          child: Text('One'),
        ),
        Positioned(
          right: 0,
          bottom: 0,
          child: Text('Two'),
        ),
      ],
    )"#;

        let node = parse_widget_tree(source).expect("stack node");
        assert_eq!(node.widget, "Stack");
        assert_eq!(node.children.len(), 2);

        let first = &node.children[0];
        assert_eq!(first.widget, "Positioned");
        assert_prop_literal(first, "left", "0");
        assert_prop_literal(first, "top", "0");

        let second = &node.children[1];
        assert_eq!(second.widget, "Positioned");
        assert_prop_literal(second, "right", "0");
        assert_prop_literal(second, "bottom", "0");
    }

    #[test]
    fn parse_elevated_button_with_action_and_child() {
        let source = r#"ElevatedButton(
      onPressed: () {},
      child: const Text('Tap'),
    )"#;

        let node = parse_widget_tree(source).expect("button node");
        assert_eq!(node.widget, "ElevatedButton");
        match node.props.get("onPressed").expect("onPressed") {
            PropValue::Expression { expression } => assert_eq!(expression, "() {}"),
            other => panic!("unexpected prop: {:?}", other),
        }
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].widget, "Text");
    }

    #[test]
    fn parse_center_wraps_child() {
        let source = r#"Center(
      child: Text('Aligned'),
    )"#;

        let node = parse_widget_tree(source).expect("center node");
        assert_eq!(node.widget, "Center");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].widget, "Text");
    }

    #[test]
    fn parse_icon_with_size() {
        let source = r#"Icon(
      Icons.home,
      size: 24,
    )"#;

        let node = parse_widget_tree(source).expect("icon node");
        assert_eq!(node.widget, "Icon");
        assert_prop_literal(&node, "icon", "Icons.home");
        assert_prop_literal(&node, "size", "24");
    }

    #[test]
    fn parse_icon_with_provider_binding() {
        let source = r#"Icon(ref.watch(iconProvider))"#;

        let node = parse_widget_tree(source).expect("icon node");
        assert_eq!(node.widget, "Icon");
        assert_prop_binding(&node, "icon", "iconProvider", None);
    }

    #[test]
    fn parse_icon_with_provider_binding_and_path() {
        let source = r#"Icon(ref.watch(iconProvider).iconData)"#;

        let node = parse_widget_tree(source).expect("icon node");
        assert_eq!(node.widget, "Icon");
        assert_prop_binding(&node, "icon", "iconProvider", Some("iconData"));
    }

    #[test]
    fn parse_text_field_with_controller() {
        let source = r#"TextField(
      controller: controller,
      keyboardType: TextInputType.emailAddress,
    )"#;

        let node = parse_widget_tree(source).expect("textfield node");
        assert_eq!(node.widget, "TextField");
        assert_prop_literal(&node, "controller", "controller");
        assert_prop_literal(&node, "keyboardType", "TextInputType.emailAddress");
    }

    #[test]
    fn parse_text_field_with_provider_controller_binding() {
        let source = r#"TextField(
      controller: ref.watch(textControllerProvider),
    )"#;

        let node = parse_widget_tree(source).expect("textfield node");
        assert_eq!(node.widget, "TextField");
        assert_prop_binding(&node, "controller", "textControllerProvider", None);
    }

    #[test]
    fn parse_text_field_with_provider_controller_path_binding() {
        let source = r#"TextField(
      controller: ref.watch(textControllerProvider).state.controller,
    )"#;

        let node = parse_widget_tree(source).expect("textfield node");
        assert_eq!(node.widget, "TextField");
        assert_prop_binding(
            &node,
            "controller",
            "textControllerProvider",
            Some("state.controller"),
        );
    }

    #[test]
    fn parse_image_network_with_fit() {
        let source = r#"Image.network(
      'https://example.com/image.png',
      fit: BoxFit.cover,
    )"#;

        let node = parse_widget_tree(source).expect("image node");
        assert_eq!(node.widget, "Image.network");
        assert_prop_literal(&node, "src", "https://example.com/image.png");
        assert_prop_literal(&node, "fit", "BoxFit.cover");
    }

    #[test]
    fn parse_text_with_provider_binding_positional() {
        let source = r#"Text(ref.watch(balanceProvider))"#;

        let node = parse_widget_tree(source).expect("text node");
        assert_eq!(node.widget, "Text");
        assert_prop_binding(&node, "data", "balanceProvider", None);
    }

    #[test]
    fn parse_text_with_provider_binding_and_path() {
        let source = r#"Text(
      data: ref.watch(balanceProvider).value,
    )"#;

        let node = parse_widget_tree(source).expect("text node");
        assert_eq!(node.widget, "Text");
        assert_prop_binding(&node, "data", "balanceProvider", Some("value"));
    }

    #[test]
    fn parse_list_tile_with_provider_binding() {
        let source = r#"ListTile(
      subtitle: ref.watch(statusProvider),
    )"#;

        let node = parse_widget_tree(source).expect("list tile node");
        assert_eq!(node.widget, "ListTile");
        assert_prop_binding(&node, "subtitle", "statusProvider", None);
    }

    #[test]
    fn parse_list_tile_with_provider_binding_and_path() {
        let source = r#"ListTile(
      subtitle: ref.watch(statusProvider).label,
      trailing: ref.watch(menuProvider).icon,
    )"#;

        let node = parse_widget_tree(source).expect("list tile node");
        assert_eq!(node.widget, "ListTile");
        assert_prop_binding(&node, "subtitle", "statusProvider", Some("label"));
        assert_prop_binding(&node, "trailing", "menuProvider", Some("icon"));
    }

    #[test]
    fn parse_expanded_with_flex() {
        let source = r#"Expanded(
      flex: 2,
      child: Text('Wide'),
    )"#;

        let node = parse_widget_tree(source).expect("expanded node");
        assert_eq!(node.widget, "Expanded");
        assert_prop_literal(&node, "flex", "2");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].widget, "Text");
    }

    #[test]
    fn parse_flexible_with_fit() {
        let source = r#"Flexible(
      fit: FlexFit.tight,
      child: Text('Flexible'),
    )"#;

        let node = parse_widget_tree(source).expect("flexible node");
        assert_eq!(node.widget, "Flexible");
        assert_prop_literal(&node, "fit", "FlexFit.tight");
        assert_eq!(node.children.len(), 1);
        assert_eq!(node.children[0].widget, "Text");
    }

    #[test]
    fn parse_divider_with_dimensions() {
        let source = r#"Divider(
      height: 2,
      thickness: 1,
    )"#;

        let node = parse_widget_tree(source).expect("divider node");
        assert_eq!(node.widget, "Divider");
        assert_prop_literal(&node, "height", "2");
        assert_prop_literal(&node, "thickness", "1");
        assert!(node.children.is_empty());
    }

    #[test]
    fn round_trip_generate_and_parse_preserves_structure() {
        let source = r#"
import 'package:flutter/widgets.dart';

class GreetingScreen extends StatelessWidget {
  const GreetingScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Column(
      children: [
        Text('Hello'),
        Text('World'),
      ],
    );
  }
}
"#;

        let graphs = build_graphs_from_source(source);
        assert_eq!(graphs.len(), 1);
        let dart = generate_dart_module(&graphs);
        let regenerated = build_graphs_from_source(&dart);
        assert_eq!(graphs, regenerated);
    }
}
