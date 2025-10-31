use forge_engine::{
    BuiltinLogicNodeKind, Flow, ForgeGraph, LogicEdge, LogicGraph, LogicNode, PropValue,
    SchemaDocument, SchemaProject, SchemaWriter, ScreenGraph, WidgetNode,
};
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn make_screen(id: &str) -> ScreenGraph {
    let mut props = BTreeMap::new();
    props.insert(
        "title".to_string(),
        PropValue::Literal {
            value: Value::String("Forge".into()),
        },
    );

    let child_props = BTreeMap::from([(
        "value".into(),
        PropValue::Expression {
            expression: "context.title".into(),
        },
    )]);

    ScreenGraph {
        id: id.to_string(),
        root: WidgetNode {
            widget: "AppBar".into(),
            props,
            children: vec![WidgetNode {
                widget: "Text".into(),
                props: child_props,
                children: Vec::new(),
            }],
        },
    }
}

fn make_logic_graph() -> LogicGraph {
    let nodes = vec![
        LogicNode {
            id: "start".into(),
            name: Some("Start".into()),
            kind: Some(BuiltinLogicNodeKind::EventEntry),
            custom_kind: None,
            props: json!({}),
            inputs: Vec::new(),
            outputs: vec!["then".into()],
            metadata: None,
        },
        LogicNode {
            id: "return".into(),
            name: Some("Return".into()),
            kind: Some(BuiltinLogicNodeKind::Return),
            custom_kind: None,
            props: json!({ "value": "42" }),
            inputs: vec!["value".into()],
            outputs: Vec::new(),
            metadata: None,
        },
    ];

    let edges = vec![LogicEdge {
        from_node: "start".into(),
        from_port: Some("then".into()),
        to_node: "return".into(),
        to_port: Some("value".into()),
        metadata: None,
    }];

    LogicGraph {
        flows: vec![Flow {
            id: "flow.main".into(),
            name: Some("Main".into()),
            nodes,
            edges,
            entry_nodes: vec!["start".into()],
            metadata: None,
        }],
        metadata: None,
    }
}

fn make_project(id: &str, name: &str) -> SchemaProject {
    SchemaProject::new(id, name)
}

fn make_document(
    project: SchemaProject,
    screens: Vec<ScreenGraph>,
    logic: Vec<LogicGraph>,
) -> SchemaDocument {
    SchemaWriter::build_document(project, screens, logic, None)
}

#[test]
fn schema_writer_serializes_basic_screen_graph() {
    let project = make_project("proj-1", "Forge Demo");
    let screen = make_screen("home_screen");
    let document = make_document(project, vec![screen.clone()], vec![]);

    let output = document
        .to_string_pretty()
        .expect("schema writer to string");
    let value: Value = serde_json::from_str(&output).expect("valid json");

    assert_eq!(value["forge_schema_version"], "1.0.0");
    assert_eq!(value["project"]["id"], "proj-1");
    assert_eq!(value["project"]["name"], "Forge Demo");
    assert_eq!(value["screens"].as_array().unwrap().len(), 1);
    assert_eq!(value["screens"][0]["id"], "home_screen");
    assert_eq!(value["screens"][0]["root"]["widget"], "AppBar");
    assert_eq!(
        value["screens"][0]["root"]["children"][0]["props"]["value"]["type"],
        "expression"
    );
}

#[test]
fn schema_writer_includes_logic_graphs() {
    let project = make_project("proj-logic", "Forge Logic");
    let screen = make_screen("home_screen");
    let logic = make_logic_graph();

    let document = make_document(project, vec![screen], vec![logic]);
    let output = document.to_string_pretty().expect("serialize logic graph");
    let value: Value = serde_json::from_str(&output).expect("valid json");

    assert_eq!(value["logic"].as_array().unwrap().len(), 1);
    assert_eq!(value["logic"][0]["flows"].as_array().unwrap().len(), 1);
    assert_eq!(
        value["logic"][0]["flows"][0]["nodes"]
            .as_array()
            .unwrap()
            .len(),
        2
    );
    assert_eq!(
        value["logic"][0]["flows"][0]["edges"]
            .as_array()
            .unwrap()
            .len(),
        1
    );
}

#[test]
fn schema_writer_builds_from_forge_graph() {
    let project = make_project("proj-agg", "Forge Aggregator");
    let screen = make_screen("landing");
    let graph = ForgeGraph::new(project.clone())
        .with_screens(vec![screen])
        .with_logic(vec![]);

    let document = SchemaWriter::from_graph(graph);
    assert_eq!(document.project.id, project.id);
    assert_eq!(document.screens.len(), 1);
    assert!(document.logic.is_empty());

    let value: Value = serde_json::from_str(
        &document
            .to_string_pretty()
            .expect("serialize aggregator document"),
    )
    .expect("json value");
    assert_eq!(value["screens"].as_array().unwrap().len(), 1);
    assert_eq!(value["logic"].as_array().unwrap().len(), 0);
}
