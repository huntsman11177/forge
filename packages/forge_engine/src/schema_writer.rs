use crate::{
    BindingReference, BindingTarget, Flow, LogicEdge, LogicGraph, LogicNode, PropValue,
    ScreenGraph, WidgetNode,
};
use serde_json::{json, Map, Value};

/// Canonical project metadata embedded in serialized Forge documents.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SchemaProject {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
}

impl SchemaProject {
    pub fn new<S: Into<String>>(id: S, name: S) -> Self {
        Self {
            id: id.into(),
            name: name.into(),
            description: None,
        }
    }

    fn to_value(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("id".to_string(), Value::String(self.id.clone()));
        obj.insert("name".to_string(), Value::String(self.name.clone()));
        if let Some(desc) = &self.description {
            obj.insert("description".to_string(), Value::String(desc.clone()));
        }
        Value::Object(obj)
    }
}

/// High-level Forge document representation returned by the schema writer.
#[derive(Debug, Clone, PartialEq)]
pub struct SchemaDocument {
    pub version: String,
    pub project: SchemaProject,
    pub screens: Vec<ScreenGraph>,
    pub logic: Vec<LogicGraph>,
    pub metadata: Option<Value>,
}

/// Aggregated Forge graph encompassing screens, logic flows, and project metadata.
#[derive(Debug, Clone, PartialEq)]
pub struct ForgeGraph {
    pub project: SchemaProject,
    pub screens: Vec<ScreenGraph>,
    pub logic: Vec<LogicGraph>,
    pub metadata: Option<Value>,
}

impl ForgeGraph {
    pub fn new(project: SchemaProject) -> Self {
        Self {
            project,
            screens: Vec::new(),
            logic: Vec::new(),
            metadata: None,
        }
    }

    pub fn with_screens(mut self, screens: Vec<ScreenGraph>) -> Self {
        self.screens = screens;
        self
    }

    pub fn with_logic(mut self, logic: Vec<LogicGraph>) -> Self {
        self.logic = logic;
        self
    }

    pub fn with_metadata(mut self, metadata: Option<Value>) -> Self {
        self.metadata = metadata;
        self
    }
}

impl SchemaDocument {
    pub fn to_value(&self) -> Value {
        let mut root = Map::new();
        root.insert(
            "forge_schema_version".to_string(),
            Value::String(self.version.clone()),
        );
        root.insert("project".to_string(), self.project.to_value());
        root.insert(
            "screens".to_string(),
            Value::Array(
                self.screens
                    .iter()
                    .map(|screen| screen.to_schema_node())
                    .collect(),
            ),
        );
        root.insert(
            "logic".to_string(),
            Value::Array(
                self.logic
                    .iter()
                    .map(|graph| graph.to_schema_node())
                    .collect(),
            ),
        );
        if let Some(metadata) = &self.metadata {
            root.insert("metadata".to_string(), metadata.clone());
        }
        Value::Object(root)
    }

    pub fn to_string_pretty(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string_pretty(&self.to_value())
    }
}

/// Writes Forge in-memory graphs into deterministic schema documents.
pub struct SchemaWriter;

impl SchemaWriter {
    pub const SCHEMA_VERSION: &'static str = "1.0.0";

    pub fn from_screens(project: SchemaProject, screens: Vec<ScreenGraph>) -> SchemaDocument {
        let graph = ForgeGraph {
            project,
            screens,
            logic: Vec::new(),
            metadata: None,
        };
        Self::from_graph(graph)
    }

    pub fn from_graph(graph: ForgeGraph) -> SchemaDocument {
        SchemaDocument {
            version: Self::SCHEMA_VERSION.to_string(),
            project: graph.project,
            screens: graph.screens,
            logic: graph.logic,
            metadata: graph.metadata,
        }
    }

    pub fn build_document(
        project: SchemaProject,
        screens: Vec<ScreenGraph>,
        logic: Vec<LogicGraph>,
        metadata: Option<Value>,
    ) -> SchemaDocument {
        let graph = ForgeGraph {
            project,
            screens,
            logic,
            metadata,
        };
        Self::from_graph(graph)
    }

    pub fn to_string(document: &SchemaDocument) -> Result<String, serde_json::Error> {
        document.to_string_pretty()
    }
}

/// Trait implemented by graph structures that can be lowered into schema nodes.
pub trait ToSchemaNode {
    fn to_schema_node(&self) -> Value;
}

impl ToSchemaNode for ScreenGraph {
    fn to_schema_node(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("id".to_string(), Value::String(self.id.clone()));
        obj.insert("name".to_string(), Value::String(self.id.clone()));
        obj.insert("root".to_string(), self.root.to_schema_node());
        Value::Object(obj)
    }
}

impl ToSchemaNode for WidgetNode {
    fn to_schema_node(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("widget".to_string(), Value::String(self.widget.clone()));

        if !self.props.is_empty() {
            let mut props = Map::new();
            for (key, value) in &self.props {
                props.insert(key.clone(), value.to_schema_node());
            }
            obj.insert("props".to_string(), Value::Object(props));
        } else {
            obj.insert("props".to_string(), Value::Object(Map::new()));
        }

        let children = self
            .children
            .iter()
            .map(|child| child.to_schema_node())
            .collect();
        obj.insert("children".to_string(), Value::Array(children));

        Value::Object(obj)
    }
}

impl ToSchemaNode for PropValue {
    fn to_schema_node(&self) -> Value {
        match self {
            PropValue::Literal { value } => {
                let mut obj = Map::new();
                obj.insert("type".to_string(), Value::String("literal".into()));
                obj.insert("value".to_string(), value.clone());
                Value::Object(obj)
            }
            PropValue::Expression { expression } => {
                let mut obj = Map::new();
                obj.insert("type".to_string(), Value::String("expression".into()));
                obj.insert("expression".to_string(), Value::String(expression.clone()));
                Value::Object(obj)
            }
            PropValue::Binding { binding } => binding.to_schema_node(),
        }
    }
}

impl ToSchemaNode for BindingReference {
    fn to_schema_node(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("type".to_string(), Value::String("binding".into()));
        obj.insert(
            "target".to_string(),
            Value::String(binding_target_to_str(&self.target).into()),
        );
        obj.insert("ref".to_string(), Value::String(self.reference.clone()));

        if let Some(provider) = &self.provider_id {
            obj.insert("provider".to_string(), Value::String(provider.clone()));
        }
        if let Some(path) = &self.path {
            obj.insert("path".to_string(), Value::String(path.clone()));
        }
        if let Some(ty) = &self.type_hint {
            obj.insert("type_hint".to_string(), Value::String(ty.clone()));
        }

        Value::Object(obj)
    }
}

impl ToSchemaNode for LogicGraph {
    fn to_schema_node(&self) -> Value {
        let mut obj = Map::new();
        obj.insert(
            "flows".to_string(),
            Value::Array(
                self.flows
                    .iter()
                    .map(|flow| flow.to_schema_node())
                    .collect(),
            ),
        );
        if let Some(metadata) = &self.metadata {
            obj.insert("metadata".to_string(), metadata.clone());
        }
        Value::Object(obj)
    }
}

impl ToSchemaNode for Flow {
    fn to_schema_node(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("id".to_string(), Value::String(self.id.clone()));
        if let Some(name) = &self.name {
            obj.insert("name".to_string(), Value::String(name.clone()));
        }
        obj.insert(
            "nodes".to_string(),
            Value::Array(
                self.nodes
                    .iter()
                    .map(|node| node.to_schema_node())
                    .collect(),
            ),
        );
        obj.insert(
            "edges".to_string(),
            Value::Array(
                self.edges
                    .iter()
                    .map(|edge| edge.to_schema_node())
                    .collect(),
            ),
        );
        obj.insert(
            "entry_nodes".to_string(),
            Value::Array(self.entry_nodes.iter().map(|id| json!(id)).collect()),
        );
        if let Some(metadata) = &self.metadata {
            obj.insert("metadata".to_string(), metadata.clone());
        }
        Value::Object(obj)
    }
}

impl ToSchemaNode for LogicNode {
    fn to_schema_node(&self) -> Value {
        let mut obj = Map::new();
        obj.insert("id".to_string(), Value::String(self.id.clone()));
        if let Some(name) = &self.name {
            obj.insert("name".to_string(), Value::String(name.clone()));
        }
        if let Some(kind) = &self.kind {
            obj.insert(
                "kind".to_string(),
                serde_json::to_value(kind).unwrap_or(Value::Null),
            );
        }
        if let Some(custom) = &self.custom_kind {
            obj.insert("custom_kind".to_string(), Value::String(custom.clone()));
        }
        obj.insert("props".to_string(), self.props.clone());
        obj.insert(
            "inputs".to_string(),
            Value::Array(self.inputs.iter().map(|v| json!(v)).collect()),
        );
        obj.insert(
            "outputs".to_string(),
            Value::Array(self.outputs.iter().map(|v| json!(v)).collect()),
        );
        if let Some(metadata) = &self.metadata {
            obj.insert("metadata".to_string(), metadata.clone());
        }
        Value::Object(obj)
    }
}

impl ToSchemaNode for LogicEdge {
    fn to_schema_node(&self) -> Value {
        let mut obj = Map::new();
        obj.insert(
            "from_node".to_string(),
            Value::String(self.from_node.clone()),
        );
        if let Some(port) = &self.from_port {
            obj.insert("from_port".to_string(), Value::String(port.clone()));
        }
        obj.insert("to_node".to_string(), Value::String(self.to_node.clone()));
        if let Some(port) = &self.to_port {
            obj.insert("to_port".to_string(), Value::String(port.clone()));
        }
        if let Some(metadata) = &self.metadata {
            obj.insert("metadata".to_string(), metadata.clone());
        }
        Value::Object(obj)
    }
}

fn binding_target_to_str(target: &BindingTarget) -> &'static str {
    match target {
        BindingTarget::Provider => "provider",
        BindingTarget::Widget => "widget",
        BindingTarget::Logic => "logic",
        BindingTarget::External => "external",
    }
}
