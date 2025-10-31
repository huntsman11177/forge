use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::{Map, Value};
use std::collections::HashMap;

pub type LogicNodeId = String;
pub type FlowId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "camelCase")]
pub enum BuiltinLogicNodeKind {
    EventEntry,
    ActionSetState,
    ActionEmitEvent,
    Condition,
    Delay,
    HttpRequest,
    Transform,
    Return,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogicNode {
    pub id: LogicNodeId,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub kind: Option<BuiltinLogicNodeKind>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub custom_kind: Option<String>,
    #[serde(default = "default_props")]
    pub props: Value,
    #[serde(default)]
    pub inputs: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct LogicEdge {
    pub from_node: LogicNodeId,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub from_port: Option<String>,
    pub to_node: LogicNodeId,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub to_port: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Flow {
    pub id: FlowId,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub name: Option<String>,
    #[serde(default)]
    pub nodes: Vec<LogicNode>,
    #[serde(default)]
    pub edges: Vec<LogicEdge>,
    #[serde(default)]
    pub entry_nodes: Vec<LogicNodeId>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct LogicGraph {
    #[serde(default)]
    pub flows: Vec<Flow>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub metadata: Option<Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
#[serde(untagged)]
pub enum ExprValue {
    Null,
    Bool(bool),
    Number(f64),
    String(String),
    Object(Map<String, Value>),
    Array(Vec<Value>),
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvalTraceEntry {
    pub timestamp: DateTime<Utc>,
    pub node_id: LogicNodeId,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub node_kind: Option<BuiltinLogicNodeKind>,
    #[serde(skip_serializing_if = "Option::is_none", default)]
    pub custom_kind: Option<String>,
    pub input: Value,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub output: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub error: Option<String>,
    pub duration_ms: u128,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct EvalResult {
    pub success: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[serde(default)]
    pub return_value: Option<Value>,
    #[serde(default)]
    pub traces: Vec<EvalTraceEntry>,
    #[serde(default)]
    pub diagnostics: Vec<String>,
    #[serde(default)]
    pub provider_state: HashMap<String, Value>,
}

fn default_props() -> Value {
    Value::Object(Map::new())
}
