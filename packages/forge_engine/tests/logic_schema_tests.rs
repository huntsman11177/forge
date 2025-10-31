use jsonschema::JSONSchema;
use serde_json::Value;
use std::fs;
use std::path::Path;

fn load_schema() -> JSONSchema {
    let schema_str = include_str!("../../../forge_spec/logic_flow_v1.json");
    let schema_json: Value =
        serde_json::from_str(schema_str).expect("valid logic flow schema JSON");
    JSONSchema::compile(&schema_json).expect("logic flow schema should compile")
}

fn validate_fixture(validator: &JSONSchema, fixture_path: &Path) {
    let contents = fs::read_to_string(fixture_path)
        .unwrap_or_else(|err| panic!("failed to read {}: {err}", fixture_path.display()));
    let json: Value = serde_json::from_str(&contents).unwrap_or_else(|err| {
        panic!(
            "fixture {} is not valid JSON: {err}",
            fixture_path.display()
        )
    });

    let result: Result<(), Vec<String>> = validator.validate(&json).map(|_| ()).map_err(|errors| {
        errors
            .map(|err| format!("{} at {}", err, err.instance_path))
            .collect::<Vec<_>>()
    });

    if let Err(messages) = result {
        panic!(
            "fixture {} failed logic schema validation: {}",
            fixture_path.display(),
            messages.join("; ")
        );
    }
}

#[test]
fn logic_fixtures_conform_to_schema() {
    let validator = load_schema();
    let fixture_dir = Path::new("fixtures/logic");

    assert!(
        fixture_dir.exists(),
        "expected fixtures/logic directory to exist"
    );

    let mut found_fixture = false;

    for entry in fs::read_dir(fixture_dir).expect("read fixtures/logic") {
        let entry = entry.expect("read fixture entry");
        let path = entry.path();
        if path.is_file() {
            found_fixture = true;
            validate_fixture(&validator, &path);
        }
    }

    assert!(found_fixture, "no logic fixtures were found to validate");
}
