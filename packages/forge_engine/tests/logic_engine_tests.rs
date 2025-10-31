use forge_engine::{
    simulate_flow, BuiltinLogicNodeKind, EvalConfig, Flow, LogicEdge, LogicGraph, LogicNode,
};
use serde_json::json;
use std::collections::HashMap;

fn make_flow() -> LogicGraph {
    let nodes = vec![
        LogicNode {
            id: "start".into(),
            name: Some("Start".into()),
            kind: Some(BuiltinLogicNodeKind::EventEntry),
            custom_kind: None,
            props: json!({}),
            inputs: vec![],
            outputs: vec!["then".into()],
            metadata: None,
        },
        LogicNode {
            id: "transform".into(),
            name: Some("Transform".into()),
            kind: Some(BuiltinLogicNodeKind::Transform),
            custom_kind: None,
            props: json!({ "expression": "providers.data.value + 1" }),
            inputs: vec!["input".into()],
            outputs: vec!["out".into()],
            metadata: None,
        },
        LogicNode {
            id: "return".into(),
            name: Some("Return".into()),
            kind: Some(BuiltinLogicNodeKind::Return),
            custom_kind: None,
            props: json!({}),
            inputs: vec!["value".into()],
            outputs: vec![],
            metadata: None,
        },
    ];

    let edges = vec![
        LogicEdge {
            from_node: "start".into(),
            from_port: Some("then".into()),
            to_node: "transform".into(),
            to_port: Some("input".into()),
            metadata: None,
        },
        LogicEdge {
            from_node: "transform".into(),
            from_port: Some("out".into()),
            to_node: "return".into(),
            to_port: Some("value".into()),
            metadata: None,
        },
    ];

    LogicGraph {
        flows: vec![Flow {
            id: "flow.basic".into(),
            name: Some("Basic".into()),
            nodes,
            edges,
            entry_nodes: vec!["start".into()],
            metadata: None,
        }],
        metadata: None,
    }
}

#[test]
fn simple_flow_returns_incremented_value() {
    let flow = make_flow();
    let mut providers = HashMap::new();
    providers.insert("data".to_string(), json!({"value": 1.0}));
    let result = simulate_flow(
        &flow,
        "flow.basic",
        None,
        Some(&providers),
        EvalConfig::default(),
    )
    .expect("simulation");

    assert!(result.success);
    assert_eq!(result.return_value, Some(json!(2.0)));
    assert_eq!(result.traces.len(), 3);
}
