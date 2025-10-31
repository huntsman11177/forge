use crate::expr::{eval_expression, parse_expression, EvalContext};
use crate::logic_types::{
    BuiltinLogicNodeKind, EvalResult, EvalTraceEntry, Flow, LogicEdge, LogicGraph, LogicNode,
    LogicNodeId,
};
use chrono::{DateTime, Utc};
use serde_json::{json, Map, Value};
use std::collections::{HashMap, VecDeque};
use thiserror::Error;

const DEFAULT_MAX_STEPS: usize = 10_000;
const DEFAULT_MAX_TRACE: usize = 1_000;

#[derive(Debug, Clone)]
pub struct EvalConfig {
    pub max_steps: usize,
    pub max_trace: usize,
}

impl Default for EvalConfig {
    fn default() -> Self {
        Self {
            max_steps: DEFAULT_MAX_STEPS,
            max_trace: DEFAULT_MAX_TRACE,
        }
    }
}

#[derive(Debug, Error)]
pub enum LogicError {
    #[error("flow '{0}' not found")]
    FlowNotFound(String),
    #[error("node '{node_id}' not found in flow '{flow_id}'")]
    NodeNotFound { flow_id: String, node_id: String },
    #[error("no entry node available for flow '{0}'")]
    MissingEntryNode(String),
    #[error("max steps {0} exceeded during simulation")]
    MaxStepsExceeded(usize),
    #[error("expression error in node '{node_id}': {message}")]
    ExpressionError { node_id: String, message: String },
}

struct Activation {
    node_id: LogicNodeId,
    input: Value,
}

struct NodeExecution {
    outputs: Vec<(Option<String>, Value)>,
    return_value: Option<Value>,
    diagnostics: Vec<String>,
    error: Option<String>,
    output: Option<Value>,
    provider_updates: Vec<ProviderUpdate>,
}

#[derive(Debug)]
struct ProviderUpdate {
    provider_id: String,
    path: Option<String>,
    value: Value,
}

pub fn simulate_flow(
    graph: &LogicGraph,
    flow_id: &str,
    entry: Option<&str>,
    seed_providers: Option<&HashMap<String, Value>>,
    config: EvalConfig,
) -> Result<EvalResult, LogicError> {
    let flow = graph
        .flows
        .iter()
        .find(|flow| flow.id == flow_id)
        .ok_or_else(|| LogicError::FlowNotFound(flow_id.to_string()))?;

    let entry_nodes = resolve_entry_nodes(flow, entry)?;

    let mut queue: VecDeque<Activation> = entry_nodes
        .into_iter()
        .map(|node_id| Activation {
            node_id,
            input: Value::Null,
        })
        .collect();

    let mut providers = seed_providers.cloned().unwrap_or_default();
    let mut diagnostics = Vec::new();
    let mut traces = Vec::new();
    let mut success = true;
    let mut return_value: Option<Value> = None;
    let mut steps: usize = 0;

    while let Some(activation) = queue.pop_front() {
        steps += 1;
        if steps > config.max_steps {
            return Err(LogicError::MaxStepsExceeded(config.max_steps));
        }

        let node = find_node(flow, &activation.node_id)?;
        let execution = execute_node(node, &activation.input, &providers)?;

        record_trace(
            &mut traces,
            node,
            &activation.input,
            &execution,
            config.max_trace,
        );

        diagnostics.extend(execution.diagnostics.iter().cloned());

        if let Some(error) = execution.error {
            success = false;
            diagnostics.push(error);
            break;
        }

        if let Some(value) = execution.return_value {
            return_value = Some(value);
            break;
        }

        for update in execution.provider_updates {
            set_provider_value(
                &mut providers,
                &update.provider_id,
                update.path.as_deref(),
                update.value,
            );
        }

        for (port, value) in execution.outputs {
            for edge in matching_edges(flow, &node.id, port.as_deref()) {
                queue.push_back(Activation {
                    node_id: edge.to_node.clone(),
                    input: value.clone(),
                });
            }
        }
    }

    Ok(EvalResult {
        success,
        return_value,
        traces,
        diagnostics,
        provider_state: providers,
    })
}

fn resolve_entry_nodes(
    flow: &Flow,
    explicit: Option<&str>,
) -> Result<Vec<LogicNodeId>, LogicError> {
    if let Some(id) = explicit {
        return Ok(vec![id.to_string()]);
    }

    if !flow.entry_nodes.is_empty() {
        return Ok(flow.entry_nodes.clone());
    }

    if let Some(entry) = flow
        .nodes
        .iter()
        .find(|node| matches!(node.kind, Some(BuiltinLogicNodeKind::EventEntry)))
    {
        return Ok(vec![entry.id.clone()]);
    }

    Err(LogicError::MissingEntryNode(flow.id.clone()))
}

fn find_node<'a>(flow: &'a Flow, node_id: &str) -> Result<&'a LogicNode, LogicError> {
    flow.nodes
        .iter()
        .find(|node| node.id == node_id)
        .ok_or_else(|| LogicError::NodeNotFound {
            flow_id: flow.id.clone(),
            node_id: node_id.to_string(),
        })
}

fn matching_edges<'a>(
    flow: &'a Flow,
    from_node: &str,
    from_port: Option<&str>,
) -> Vec<&'a LogicEdge> {
    flow.edges
        .iter()
        .filter(|edge| {
            edge.from_node == from_node
                && match (&edge.from_port, from_port) {
                    (None, _) => true,
                    (Some(port), Some(candidate)) => port == candidate,
                    _ => false,
                }
        })
        .collect()
}

fn execute_node(
    node: &LogicNode,
    input: &Value,
    providers: &HashMap<String, Value>,
) -> Result<NodeExecution, LogicError> {
    match node.kind {
        Some(BuiltinLogicNodeKind::EventEntry) => {
            let outputs = default_outputs(node, input.clone());
            Ok(NodeExecution {
                outputs,
                return_value: None,
                diagnostics: Vec::new(),
                error: None,
                output: Some(input.clone()),
                provider_updates: Vec::new(),
            })
        }
        Some(BuiltinLogicNodeKind::Transform) => {
            let expression = string_prop(&node.props, "expression", &node.id)?;
            let value = evaluate_expression(&expression, input, providers, &node.id)?;
            let outputs = default_outputs(node, value.clone());
            Ok(NodeExecution {
                outputs,
                return_value: None,
                diagnostics: Vec::new(),
                error: None,
                output: Some(value),
                provider_updates: Vec::new(),
            })
        }
        Some(BuiltinLogicNodeKind::Condition) => {
            let expression = string_prop(&node.props, "expression", &node.id)?;
            let result = evaluate_expression(&expression, input, providers, &node.id)?;
            let choice = truthy(&result);
            let port_key = if choice {
                optional_string_prop(&node.props, "true_port")?.unwrap_or_else(|| "then".into())
            } else {
                optional_string_prop(&node.props, "false_port")?.unwrap_or_else(|| "else".into())
            };
            Ok(NodeExecution {
                outputs: vec![(Some(port_key), input.clone())],
                return_value: None,
                diagnostics: Vec::new(),
                error: None,
                output: Some(result),
                provider_updates: Vec::new(),
            })
        }
        Some(BuiltinLogicNodeKind::Return) => {
            let value = if let Some(expr) = optional_string_prop(&node.props, "expression")? {
                evaluate_expression(&expr, input, providers, &node.id)?
            } else {
                input.clone()
            };
            Ok(NodeExecution {
                outputs: Vec::new(),
                return_value: Some(value.clone()),
                diagnostics: Vec::new(),
                error: None,
                output: Some(value),
                provider_updates: Vec::new(),
            })
        }
        Some(BuiltinLogicNodeKind::ActionSetState) => {
            let provider_id = string_prop(&node.props, "provider_id", &node.id)?;
            let path = optional_string_prop(&node.props, "path")?;
            let expression = string_prop(&node.props, "expression", &node.id)?;
            let value = evaluate_expression(&expression, input, providers, &node.id)?;
            Ok(NodeExecution {
                outputs: default_outputs(node, input.clone()),
                return_value: None,
                diagnostics: vec![format!(
                    "__provider_update__:{}",
                    json!({"provider_id": provider_id, "path": path, "value": value})
                )],
                error: None,
                output: Some(value.clone()),
                provider_updates: vec![ProviderUpdate {
                    provider_id,
                    path,
                    value,
                }],
            })
        }
        _ => Ok(NodeExecution {
            outputs: Vec::new(),
            return_value: None,
            diagnostics: vec![format!("unsupported node kind on '{}'; skipping", node.id)],
            error: None,
            output: None,
            provider_updates: Vec::new(),
        }),
    }
}

fn evaluate_expression(
    expression: &str,
    input: &Value,
    providers: &HashMap<String, Value>,
    node_id: &str,
) -> Result<Value, LogicError> {
    let expr = parse_expression(expression).map_err(|err| LogicError::ExpressionError {
        node_id: node_id.to_string(),
        message: err.to_string(),
    })?;

    let root = build_eval_root(input, providers);
    let ctx = EvalContext::with_now(&root);
    eval_expression(&expr, &ctx).map_err(|err| LogicError::ExpressionError {
        node_id: node_id.to_string(),
        message: err.to_string(),
    })
}

fn build_eval_root(input: &Value, providers: &HashMap<String, Value>) -> Value {
    json!({
        "input": input,
        "providers": providers,
    })
}

fn default_outputs(node: &LogicNode, value: Value) -> Vec<(Option<String>, Value)> {
    if node.outputs.is_empty() {
        vec![(None, value)]
    } else {
        node.outputs
            .iter()
            .map(|port| (Some(port.clone()), value.clone()))
            .collect()
    }
}

fn string_prop(props: &Value, key: &str, node_id: &str) -> Result<String, LogicError> {
    match props {
        Value::Object(map) => map
            .get(key)
            .and_then(Value::as_str)
            .map(|v| v.to_string())
            .ok_or_else(|| LogicError::ExpressionError {
                node_id: node_id.to_string(),
                message: format!("missing string property '{key}'"),
            }),
        _ => Err(LogicError::ExpressionError {
            node_id: node_id.to_string(),
            message: format!("missing property container for '{key}'"),
        }),
    }
}

fn optional_string_prop(props: &Value, key: &str) -> Result<Option<String>, LogicError> {
    match props {
        Value::Object(map) => Ok(map
            .get(key)
            .and_then(Value::as_str)
            .map(|value| value.to_string())),
        _ => Err(LogicError::ExpressionError {
            node_id: key.to_string(),
            message: format!("missing property container for '{key}'"),
        }),
    }
}

fn truthy(value: &Value) -> bool {
    match value {
        Value::Bool(b) => *b,
        Value::Null => false,
        Value::Number(n) => n.as_f64().map_or(false, |v| v != 0.0),
        Value::String(s) => !s.is_empty(),
        Value::Array(arr) => !arr.is_empty(),
        Value::Object(obj) => !obj.is_empty(),
    }
}

fn record_trace(
    traces: &mut Vec<EvalTraceEntry>,
    node: &LogicNode,
    input: &Value,
    execution: &NodeExecution,
    max_trace: usize,
) {
    if traces.len() >= max_trace {
        return;
    }

    let timestamp: DateTime<Utc> = Utc::now();
    traces.push(EvalTraceEntry {
        timestamp,
        node_id: node.id.clone(),
        node_kind: node.kind.clone(),
        custom_kind: node.custom_kind.clone(),
        input: input.clone(),
        output: execution.output.clone(),
        error: execution.error.clone(),
        duration_ms: 0,
    });
}

fn set_provider_value(
    providers: &mut HashMap<String, Value>,
    provider_id: &str,
    path: Option<&str>,
    value: Value,
) {
    let entry = providers
        .entry(provider_id.to_string())
        .or_insert(Value::Null);

    match path {
        None => {
            *entry = value;
        }
        Some(path) => {
            let segments: Vec<&str> = path
                .split('.')
                .filter(|segment| !segment.is_empty())
                .collect();
            if segments.is_empty() {
                *entry = value;
                return;
            }
            set_nested_value(entry, &segments, value);
        }
    }
}

fn set_nested_value(target: &mut Value, segments: &[&str], value: Value) {
    if segments.is_empty() {
        *target = value;
        return;
    }

    let key = segments[0];
    let map = as_object_mut(target);
    if segments.len() == 1 {
        map.insert(key.to_string(), value);
        return;
    }

    let entry = map.entry(key.to_string()).or_insert(Value::Null);
    set_nested_value(entry, &segments[1..], value);
}

fn as_object_mut(value: &mut Value) -> &mut Map<String, Value> {
    if !value.is_object() {
        *value = Value::Object(Map::new());
    }
    match value {
        Value::Object(map) => map,
        _ => unreachable!(),
    }
}
