# Forge progress to date

## Engine module landscape
- Central `lib.rs` exposes the analyzer, renderers (Angular, Flutter, React), logic engine, manifest generator, merge engine, plugin sandbox, and state adapters, providing a consolidated API surface for Dart tooling and consumers.
- Contract types such as `WidgetNode`, `ScreenGraph`, and `ParsedScreen` are in place, enabling structured graph import/export and stateless widget parsing.
- File discovery utilities (`discover_dart_files`) and JSON graph loading (`read_graph`) support downstream analysis and rendering pipelines.

## Logic engine simulation
- `simulate_flow` drives breadth-first execution with configurable step and trace limits, wiring provider state updates and return values.
- Built-in node kinds currently supported include `EventEntry`, `Transform`, `Condition`, `Return`, and `ActionSetState`, each with targeted handlers.
- Expression evaluation is delegated to the shared expression parser/evaluator, ensuring consistency across logic and renderer integrations.

## Analyzer orchestration
- `AnalyzerService` evaluates native confidence against a configurable threshold to decide between native parsing and fallback analyzer usage.
- The service merges analyzer output with the quick parse graph via the merge engine, reporting conflicts and analyzer invocation metadata.
- CLI consumers receive serialized outcomes, facilitating diagnostics and integration with higher-level tooling.

## CLI workflow
- The Dart CLI entrypoint wires `forge` subcommands for import, render, analyze, simulate, and version reporting.
- `AnalyzeCommand` validates inputs, forwards to the Rust engine binary, surfaces stdout/stderr, and prints conflict summaries derived from the merged report.
- Import pipeline utilities (`ParserUtils`) wrap analyzer AST parsing to classify stateless/stateful widget classes, map state objects back to widgets, and generate stable screen identifiers for reverse import flows.
- New complex Flutter fixtures (`advanced_screen.dart`, `advanced_screen_v2.dart`) exercise nested layouts, form controls, and animation-ready patterns, expanding coverage for CLI importer testing.

## Manifest & dependency emission
- Engine render invocations persist a `dependencies.json` manifest alongside generated sources when `--out-dir` is supplied, capturing runtime packages per renderer.
- Optional `--emit-manifest` triggers `generate_manifest`, producing framework-specific manifests (`pubspec.yaml` for Flutter, `package.json` for React/Angular) using renderer dependency metadata.
- Descriptor metadata in `renderer_registry.rs` links each renderer to its manifest kind, ensuring consistent CLI ergonomics across frameworks.

## CI & benchmarking status
- Criterion suites exist for parser throughput, multi-framework codegen, and logic runtime simulation, covering core engine hot paths.
- Baseline metrics captured in `packages/forge_engine/benches/criterion_baseline.json` with `scripts/export_benchmarks.py`, enabling deterministic regression checks.
- `scripts/compare_benchmarks.py` reports regressions beyond configurable thresholds and fails the build when triggered.
- CI workflow now runs `cargo bench`, exports summaries, and invokes the comparison step to gate merges on performance health.
