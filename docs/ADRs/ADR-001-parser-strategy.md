# ADR-001: Parser Strategy — Hybrid Rust Quick-Parser + Dart Analyzer Fallback
Date: 2025-10-30
Status: Accepted
Deciders: Subrata Saha
Technical leads consulted: (add collaborators here)

## Context
Forge must import existing Flutter code reliably and quickly to uphold the core value proposition: **bidirectional sync between visual design (FSL) and production Dart code**.

Parsing Flutter/Dart presents tradeoffs:

- The **official Dart Analyzer** (from the Dart SDK) is authoritative, always up to date, and supports complete semantic analysis (types, imports, control flow). However, embedding it directly is heavyweight (VM/JIT requirements, process orchestration) and can be slow for interactive UIs or large codebases.
- A **custom Rust quick-parser** is lightweight, embeddable, and extremely fast. It can cover ~80–95% of common Flutter UI patterns (widget trees, provider usage, basic control flow) and integrates cleanly with Forge's Rust core. Yet it risks mis-parsing edge cases and needs maintenance as the Dart language evolves.

## Decision
Adopt a **hybrid parser strategy** combining the speed of a Rust quick-parser with the accuracy of the Dart Analyzer fallback.

1. **Quick Parser (Primary Path):** Rust-based parser provides the optimistic, interactive experience. Responsibilities:
   - Parse widget trees returned from `build()` methods.
   - Recognize common Flutter widgets and props.
   - Detect state management patterns (e.g., `final fooProvider = StateProvider(...)`, `ref.watch(fooProvider)`).
   - Produce a lightweight AST/FSL representation with confidence scoring.

2. **Fallback Parser (Authoritative Path):** Invoke the official Dart Analyzer when confidence falls below threshold or for final verification. Runs as an isolated process (e.g., language server, containerized analyzer) and produces the authoritative AST + semantic information.

3. **Confidence Threshold & Routing:** Quick parser returns `ParseResult { graph, confidence: 0.0-1.0, diagnostics }`.
   - If `confidence >= 0.92`: accept quick parser output.
   - If `0.6 <= confidence < 0.92`: show "verify" UI, optionally schedule Dart Analyzer in background.
   - If `confidence < 0.6`: automatically run Dart Analyzer fallback and use its output.

4. **Deterministic Merge:** Both parsers emit FSL. When the fallback result arrives, perform a three-way merge (`base`, `quick`, `analyzer`). If conflicts remain, surface them in the merge UI.

## Consequences
**Pros**
- Fast interactive import path (quick parser) for most files.
- Accurate authoritative parsing for edge cases (Dart Analyzer) without sacrificing UX.
- Balanced performance profile; analyzer runs less frequently.
- Aligns with Forge's Rust/WASM engine design.

**Cons**
- Two parser stacks increase implementation complexity.
- Requires robust merge logic and confidence heuristics.
- Needs infrastructure to execute Dart Analyzer cross-platform.

## Integration Pseudocode

### High-Level Flow
```
user -> importProject(paths)
  -> scanFiles(paths)
     -> for each file:
          parseQuick(file) -> result.confidence
          if result.confidence >= 0.92:
              acceptQuickResult(file)
          else:
              scheduleDartAnalyzer(file)   // background
              acceptQuickResultTemporarily(file)
              when analyzerDone(file):
                  mergeResults(base, quick, analyzer)
                  if merge.conflict:
                      showConflictUI(file)
                  else:
                      replaceTempWithAnalyzerResult(file)
```

### Quick Parser Interface (Rust)
```rust
pub struct ParseResult {
    pub forge_graph: ForgeGraph,
    pub confidence: f32,
    pub diagnostics: Vec<Diagnostic>,
}

pub trait QuickParser {
    fn parse_file(&self, path: &Path) -> ParseResult;
}
```

### Dart Analyzer Invocation
```
# Spawn analyzer in isolated process / container / language server
$ dart analyze --format=json path/to/project
# or communicate via gRPC/HTTP with a long-running analyzer service
```

### Merge Function (Conceptual)
```rust
fn merge_graphs(base: &ForgeGraph, quick: &ForgeGraph, analyzer: &ForgeGraph) -> MergeResult {
    // Three-way merge with property-level granularity.
    // Prefer analyzer output for structural conflicts.
    // Auto-merge non-conflicting property updates.
    // Surface unresolved conflicts to the UI for manual resolution.
}
```

## Acceptance Criteria
- Quick parser ingests a 100-file Flutter project in <3 seconds (cold start) and yields confidence ≥0.92 for ≥80% of screens.
- Dart Analyzer fallback finishes within 30 seconds for 100 files in CI environments.
- Three-way merge yields zero incorrect structural merges on ≥95% of audited test cases (remaining 5% surfaced for manual resolution).
- Import UI clearly flags nodes with confidence <0.92 and provides a one-click "Run authoritative analysis" action.
- Automated tests cover quick parser, analyzer output, and merge correctness.

## Test Plan
- Unit tests: 200 handcrafted Dart snippets covering widget patterns, providers, navigation, control flow.
- Corpus tests: 50 open-source Flutter apps to measure confidence accuracy and log misparses.
- Integration tests: Import → edit → export → reimport cycle verifying AST + code parity.
- Performance benchmarks: Measure cold/warm parse times for 100/500 file projects.
- Merge tests: Simulated concurrent edits to confirm deterministic resolution.

## Operational Considerations
- Provide `--force-analyze` flag (CLI/UI) to bypass quick parser when deterministic output is required.
- Run Dart Analyzer in sandboxed environment (container or language server) to avoid host pollution.
- Cache analyzer outputs/AST fingerprints under `.forge/cache/` to accelerate re-imports.
- Collect optional telemetry (opt-in) on parser confidence vs. analyzer usage to guide tuning.

## Rollout Plan
1. Implement Quick Parser (Rust) covering top 80% patterns.
2. Implement merge engine + conflict resolution UI.
3. Integrate Dart Analyzer fallback pipeline.
4. Run corpus tests; calibrate confidence thresholds (start at 0.92, adjust as needed).
5. Release as opt-in alpha feature; gather logs and iterate.

## Alternatives Considered
- **Dart Analyzer only:** rejected due to UX latency and embedding complexity.
- **Rust-only parser:** rejected due to long-term maintenance risk and language drift.
- **Third-party parsing services:** rejected for privacy/local-first philosophy.

## Related ADRs
- ADR-002: Conflict Resolution Strategy (planned)
- ADR-003: Schema Versioning & Migration (planned)

## Implementation Checklist
- [ ] Implement `QuickParser` interface in Rust.
- [ ] Implement `AnalyzerService` wrapper for Dart Analyzer orchestration.
- [ ] Build three-way merge engine with property-level diffs.
- [ ] Update import UI with confidence overlays and analyzer trigger.
- [ ] Add CI benchmarks and corpus tests backing the acceptance criteria.
