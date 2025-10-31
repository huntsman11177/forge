use std::collections::HashMap;

use criterion::{criterion_group, criterion_main, BenchmarkId, Criterion};
use forge_engine::{simulate_flow, EvalConfig, LogicGraph};
use serde_json::{json, Value};

fn load_graph(name: &str) -> LogicGraph {
    let raw = match name {
        "basic" => include_str!("../fixtures/logic/basic_flow.json"),
        "branching" => include_str!("../fixtures/logic/branching_flow.json"),
        other => panic!("unknown fixture {other}"),
    };
    serde_json::from_str(raw).expect("valid logic graph fixture")
}

fn providers_for(name: &str) -> HashMap<String, Value> {
    match name {
        "basic" => HashMap::new(),
        "branching" => {
            let mut map = HashMap::new();
            map.insert("data".to_string(), json!({"value": 5.0}));
            map.insert("flags".to_string(), json!({"isEnabled": true}));
            map
        }
        _ => HashMap::new(),
    }
}

fn bench_logic_flows(c: &mut Criterion) {
    let mut group = c.benchmark_group("logic_flow_simulation");
    let cases = ["basic", "branching"];

    for case in cases {
        let graph = load_graph(case);
        let providers = providers_for(case);
        group.bench_with_input(BenchmarkId::from_parameter(case), &graph, |b, g| {
            b.iter(|| {
                let result = simulate_flow(
                    g,
                    match case {
                        "basic" => "flow.basic",
                        "branching" => "flow.branching",
                        _ => unreachable!(),
                    },
                    None,
                    Some(&providers),
                    EvalConfig::default(),
                )
                .expect("simulation succeeds");
                criterion::black_box(result);
            });
        });
    }

    group.finish();
}

criterion_group!(benches, bench_logic_flows);
criterion_main!(benches);
