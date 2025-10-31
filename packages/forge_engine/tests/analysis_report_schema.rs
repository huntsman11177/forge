use forge_engine::{
    AnalysisOutcome, AnalyzerService, PropValue, ScreenGraph, WidgetNode, ANALYSIS_REPORT_VERSION,
};
use jsonschema::JSONSchema;
use serde_json::{json, Value};
use std::collections::BTreeMap;

fn load_schema() -> JSONSchema {
    let schema_str = include_str!("../../../forge_spec/analysis_report.schema.json");
    let schema_json: Value = serde_json::from_str(schema_str).expect("valid analysis schema JSON");
    JSONSchema::compile(&schema_json).expect("analysis schema should compile")
}

fn make_simple_graph(id: &str, text: &str) -> ScreenGraph {
    let mut props = BTreeMap::new();
    props.insert(
        "value".to_string(),
        PropValue::Literal {
            value: serde_json::Value::String(text.to_string()),
        },
    );

    ScreenGraph {
        id: id.to_string(),
        root: WidgetNode {
            widget: "Text".to_string(),
            props,
            children: vec![],
        },
    }
}

fn make_outcome(confidence: f32) -> AnalysisOutcome {
    let source = r#"import 'package:flutter/widgets.dart';

class SampleScreen extends StatelessWidget {
  const SampleScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return const Text('Hello');
  }
}
"#;

    let base_graph = make_simple_graph("SampleScreen", "Hello");
    let quick_graph = base_graph.clone();
    let service = AnalyzerService::default();

    service.run(source, &base_graph, quick_graph, confidence)
}

#[test]
fn analysis_report_matches_schema_native_path() {
    let outcome = make_outcome(0.9);
    validate_outcome(&outcome);
}

#[test]
fn analysis_report_matches_schema_fallback_path() {
    let outcome = make_outcome(0.1);
    validate_outcome(&outcome);
}

fn validate_outcome(outcome: &AnalysisOutcome) {
    let validator = load_schema();
    let outcome_json = serde_json::to_value(outcome).expect("serialize outcome");

    let report_json = json!({
        "version": ANALYSIS_REPORT_VERSION,
        "outcomes": [outcome_json],
        "total_conflicts": outcome.merge.conflicts.len(),
    });

    let validation: Result<(), Vec<String>> =
        validator
            .validate(&report_json)
            .map(|_| ())
            .map_err(|errors| {
                errors
                    .map(|err| format!("{} at {}", err, err.instance_path))
                    .collect::<Vec<_>>()
            });

    if let Err(messages) = validation {
        panic!(
            "Analysis report failed schema validation: {}",
            messages.join("; ")
        );
    }
}
