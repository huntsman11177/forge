use forge_engine::{read_graph, ForgeGraph, LogicGraph, SchemaProject, SchemaWriter, ScreenGraph};
use serde_json::Value;
use std::{error::Error, fs, path::PathBuf};

fn fixture_path(relative: &str) -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("fixtures")
        .join(relative)
}

fn load_screen(relative: &str) -> ScreenGraph {
    let path = fixture_path(relative);
    read_graph(&path).expect("screen fixture should parse")
}

fn load_logic(relative: &str) -> LogicGraph {
    let path = fixture_path(relative);
    let contents = fs::read_to_string(&path).expect("logic fixture should exist");
    serde_json::from_str(&contents).expect("logic fixture should parse")
}

fn extract_screens(value: &Value) -> Result<Vec<ScreenGraph>, Box<dyn Error>> {
    let mut screens = Vec::new();
    if let Some(array) = value.get("screens").and_then(|v| v.as_array()) {
        for item in array {
            let screen: ScreenGraph = serde_json::from_value(item.clone())?;
            screens.push(screen);
        }
    }
    Ok(screens)
}

fn extract_logic(value: &Value) -> Result<Vec<LogicGraph>, Box<dyn Error>> {
    let mut graphs = Vec::new();
    if let Some(array) = value.get("logic").and_then(|v| v.as_array()) {
        for item in array {
            let graph: LogicGraph = serde_json::from_value(item.clone())?;
            graphs.push(graph);
        }
    }
    Ok(graphs)
}

#[test]
fn export_roundtrip_preserves_fixtures() -> Result<(), Box<dyn Error>> {
    let screens = vec![
        load_screen("ui/flutter_basic.json"),
        load_screen("ui/react_basic.json"),
    ];
    let logic = vec![load_logic("logic/branching_flow.json")];

    let project = SchemaProject::new("roundtrip", "Roundtrip Project");
    let graph = ForgeGraph::new(project.clone())
        .with_screens(screens.clone())
        .with_logic(logic.clone());

    let document = SchemaWriter::from_graph(graph);
    let serialized = document.to_string_pretty()?;
    let value: Value = serde_json::from_str(&serialized)?;

    assert_eq!(value["forge_schema_version"], "1.0.0");
    assert_eq!(value["project"]["id"], project.id);
    assert_eq!(value["project"]["name"], project.name);

    let reparsed_screens = extract_screens(&value)?;
    assert_eq!(
        reparsed_screens, screens,
        "Screen graphs changed after export"
    );

    let reparsed_logic = extract_logic(&value)?;
    assert_eq!(reparsed_logic, logic, "Logic graphs changed after export");

    Ok(())
}
