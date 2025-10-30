use criterion::{criterion_group, criterion_main, Criterion};
use forge_engine::build_graphs_from_source;

const SAMPLE_SOURCE: &str = r#"
import 'package:flutter/widgets.dart';

class SampleScreen extends StatelessWidget {
  const SampleScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Scaffold(
      appBar: AppBar(title: const Text('Bench Title')),
      body: ListView(
        children: [
          const SizedBox(height: 16),
          Row(
            children: const [
              Expanded(child: Text('Balance')),
              SizedBox(width: 12),
              Expanded(child: Text('Value')),
            ],
          ),
          Padding(
            padding: const EdgeInsets.all(12),
            child: Column(
              children: const [
                Text('Hello'),
                SizedBox(height: 8),
                Text('World'),
              ],
            ),
          ),
        ],
      ),
    );
  }
}
"#;

fn build_benchmark_samples(count: usize) -> Vec<String> {
    (0..count)
        .map(|idx| SAMPLE_SOURCE.replace("SampleScreen", &format!("SampleScreen{idx}")))
        .collect()
}

fn benchmark_parse_graphs(c: &mut Criterion) {
    let samples = build_benchmark_samples(100);
    c.bench_function("parse_graphs_100_samples", |b| {
        b.iter(|| {
            for sample in &samples {
                let graphs = build_graphs_from_source(sample);
                criterion::black_box(&graphs);
            }
        })
    });
}

criterion_group!(benches, benchmark_parse_graphs);
criterion_main!(benches);
