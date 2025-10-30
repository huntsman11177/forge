# Forge Studio

Forge is a visual IDE and runtime for Flutter applications that turns UI design, logic authoring, data binding, and code generation into a single, live experience. It aims to replace fragmented design → builder → code handoffs with a unified, bidirectional workflow backed by production-quality exports.@Main.md#73-160 @Main.md#242-318

---

## Table of Contents

1. [Vision & Value Proposition](#vision--value-proposition)
2. [Target Users & Primary Use Cases](#target-users--primary-use-cases)
3. [Scope, Constraints & Non-goals](#scope-constraints--non-goals)
4. [Product Pillars](#product-pillars)
5. [End-to-End Workflow](#end-to-end-workflow)
6. [System Architecture](#system-architecture)
7. [Core Modules](#core-modules)
8. [Marketplace & Packaging](#marketplace--packaging)
9. [Validation & Quality Gates](#validation--quality-gates)
10. [Edge Cases & Risk Mitigations](#edge-cases--risk-mitigations)
11. [Competitive Positioning](#competitive-positioning)
12. [Business Model & Economics](#business-model--economics)
13. [Roadmap](#roadmap)
14. [Open Questions & Follow-ups](#open-questions--follow-ups)

---

## Vision & Value Proposition

- **Live WYSIWYG runtime** – Forge renders the actual Flutter runtime while you design, so every interaction reflects production behaviour (navigation, provider bindings, conditional UI, and data flows).@Main.md#93-160
- **Unified executable graph** – UI structure, logic nodes, data bindings, and runtime state are stored in a single graph that can be round-tripped between visual tooling and Dart code.@Main.md#93-160 @Main.md#242-318
- **Bidirectional editing** – Import existing Flutter repos, visually refactor them, and export clean Dart back into source control without breaking parity.@Main.md#242-318
- **Production confidence** – Generated code is idiomatic Flutter, validated through analyzer checks and `flutter build web`, so what you preview is what you ship.@Main.md#133-160 @Main.md#3200-3256

## Target Users & Primary Use Cases

| Persona | Goals | Forge Outcome |
|---------|-------|---------------|
| Flutter product engineers | Accelerate UI iteration, keep code ownership | Visual edits feed directly into Git, while preserving custom logic as black boxes.@Main.md#242-318 @Main.md#595-861 |
| UX engineers / designer-developer hybrids | Prototype behaviours with real data bindings | Live runtime removes mockup/implementation gap.@Main.md#73-160 |
| Agencies & startups | Bootstrap front-ends, reuse flows across clients | Install Forge packages, configure providers, and connect to any backend later.@Main.md#1222-1356 @Main.md#3200-3390 |

## Scope, Constraints & Non-goals

- **Flutter-first MVP** – Focus on Flutter UI, navigation, state management, and logic bindings; backend automation is outside scope.@Main.md#595-705
- **Backend remains external** – API services, auth, and data persistence are treated as black-box integrations that Forge references but does not generate.@Main.md#652-686
- **Future frameworks** – React/Next.js support is aspirational post-MVP; include gating criteria before committing resources.@Main.md#1251-1296 @Main.md#3771-3799
- **No Penpot fork** – Forge is a greenfield stack; lessons from Penpot/Figma inform UX but we do not inherit their architecture.@Main.md#933-1219

## Product Pillars

1. **Executable design** – The canvas is the app runtime, eliminating divergence between preview and production.@Main.md#93-160
2. **Composable graphs** – Everything from widgets to navigation to logic nodes is composable, versioned, and diffable.@Main.md#93-160 @Main.md#221-233
3. **Round-trip code ownership** – Teams can import, edit, and export without losing hand-written Dart.@Main.md#242-318 @Main.md#456-577
4. **Marketplace ecosystem** – Shareable, validated packages create a network effect of reusable, living modules.@Main.md#1222-3200

## End-to-End Workflow

1. **Import** – Clone or point Forge at a Flutter repo; engine parses UI screens, providers, navigation, and preserves custom code as black boxes.@Main.md#255-399 @Main.md#652-686
2. **Edit** – Use the canvas, logic graph, and provider panels to modify layout, flows, bindings, and conditions with instant preview.@Main.md#121-160 @Main.md#736-847
3. **Validate** – Run analyzer, tests, and builds via `forge validate` to surface compilation or runtime regressions.@Main.md#3200-3256
4. **Export/Commit** – Generate formatted Dart, write to disk, and optionally open a PR via Git integrations.@Main.md#282-305 @Main.md#748-777
5. **Package/Publish** (optional) – Bundle constructs into ForgePkg assets for reuse or sale.@Main.md#1222-3200

## System Architecture

```
Forge Studio (Flutter Web)
 ├─ Canvas Runtime (renders graph via Flutter)
 ├─ Inspector & Logic Graph Editor
 ├─ Provider Manager & DevTools bridge
 │
Forge Engine (Rust + WASM)
 ├─ Dart AST Parser & Pattern Library
 ├─ Graph Builder & Code Generator
 ├─ Validation adapters (analyzer, build)
 │
Forge CLI (Dart)
 ├─ Project orchestration, import/export
 ├─ Validation pipeline invocation
 │
Forge Marketplace Services (future)
 ├─ Package registry & validation pipeline
 ├─ Payments, ratings, dependency graph
```

This architecture supports local-first development while remaining cloud-ready for marketplace scenarios.@Main.md#1251-1296 @Main.md#3764-3799

## Core Modules

| Module | Responsibilities | Tech Stack | Status |
|--------|-------------------|------------|--------|
| **forge_studio** | Visual canvas, logic editor, provider binding, DevTools integration.@Main.md#73-160 @Main.md#736-847 | Flutter Web | MVP in progress |
| **forge_engine** | Parse Dart to ForgeGraph, generate code, maintain pattern library, expose WASM bindings.@Main.md#393-512 | Rust → WASM | MVP in progress |
| **forge_cli** | Import, export, validate, preview commands, Git hand-off.@Main.md#748-777 | Dart | Planned |
| **forge_spec** | Owns ForgeGraph schema, ForgePkg manifest, docs.@Main.md#1222-1399 | JSON/YAML | Planned |
| **Marketplace backend** | Registry, validation jobs, payments, analytics.@Main.md#315-3599 | Rust/TypeScript (TBD) | Future |

## Marketplace & Packaging

- **ForgePkg format** — Bundles UI graph, logic, provider schema, assets, docs, and previews with semantic versioning and dependency metadata.@Main.md#1251-1399 @Main.md#305-3200
- **Package lifecycle** — `forge pkg build`, `forge pkg validate`, `forge pkg publish`; validation enforces compilation, tests, security scanning, and performance benchmarks.@Main.md#1222-3200 @Main.md#3200-3256
- **Marketplace operations** — Quality badges, revenue sharing (15% baseline), creator tiers, continuous revalidation against latest Flutter SDK.@Main.md#1251-3599
- **Ecosystem flywheel** — More packages → higher reuse → stronger adoption, underpinning SaaS + marketplace revenue strategy.@Main.md#1342-1376 @Main.md#3512-3636

## Validation & Quality Gates

1. **Structural checks** – Schema validation for graphs, manifests, and assets.@Main.md#1222-3200
2. **Dependency resolution** – Semantic version solver for package trees and conflict detection.@Main.md#305-3200
3. **Compilation** – Run `flutter pub get`, `dart analyze`, and `flutter build web --release` in isolated environments.@Main.md#3200-3256
4. **Runtime smoke tests** – Headless browser executes preview flows to catch runtime errors and measure load performance.@Main.md#3200-3256
5. **Security scan** – Detect prohibited APIs, undeclared permissions, and vulnerable dependencies.@Main.md#3200-3256
6. **Continuous validation** – Re-run checks weekly to guard against SDK drift.@Main.md#3232-3256

## Edge Cases & Risk Mitigations

- **Parsing limitations** – Tiered strategy: fully parse simple widgets, recognise patterns for common async/widgets, and encapsulate complex logic as custom nodes.@Main.md#327-389
- **Visual complexity** – Prevent node spaghetti with subgraphs, reusable functions, and progressive disclosure in the editor.@Main.md#170-233
- **Merge conflicts** – Use stable IDs in graphs, semantic diffs, and visual merge tooling to reconcile concurrent edits.@Main.md#221-233 @Main.md#579-585
- **Custom code** – Preserve unparseable widgets as black boxes with editable props and metadata, ensuring developers retain control.@Main.md#456-577
- **Performance** – Provide profiling overlays, estimated frame-time warnings, and incremental rebuilds to avoid jank.@Main.md#221-233 @Main.md#1280-1296
- **Framework creep** – Gate non-Flutter support behind explicit milestones and validation of Flutter-first success.@Main.md#1251-1296 @Main.md#3771-3799

## Competitive Positioning

Forge differentiates through code ownership, bidirectional editing, and a marketplace of living modules, outperforming Figma, FlutterFlow, Webflow, and traditional IDEs on runtime fidelity and reuse.@Main.md#3393-3508

## Business Model & Economics

- **Subscriptions** – Free, Pro, Team, and Enterprise tiers cover collaboration, advanced widgets, and governance.@Main.md#3512-3548
- **Marketplace commissions** – Sliding 15%→10% fee structure with incentives for high-performing creators.@Main.md#3550-3565
- **Services** – Migration, custom widget development, training, and white-label licensing add high-margin revenue streams.@Main.md#3567-3584
- **Financial outlook** – Projected path to $180M ARR with strong LTV/CAC and payback metrics once network effects mature.@Main.md#3586-3636

## Roadmap

### Year 1 (Months 1–12)

| Quarter | Focus | Highlights |
|---------|-------|------------|
| Q1 | Core canvas & runtime | 20 core widgets, property panel, logic editor, Riverpod bindings, live preview.@Main.md#3771-3788 |
| Q2 | Bidirectional sync | Dart AST parser, pattern library, import wizard, diff visualisation, formatting preservation.@Main.md#3796-3799 |
| Q3 | Package system & marketplace prep | ForgePkg spec, CLI workflows, validation pipeline, marketplace UI & payments.@Main.md#1222-3200 @Main.md#1251-1296 |
| Q4 | Collaboration & polish | Multiplayer editing, debugging tools, AI assists, enterprise features, docs.@Main.md#1280-1296

### Beyond Year 1

- React/Next.js runtime support (post Flutter success proof).@Main.md#1251-1296
- AI-assisted graph creation and pattern detection.@Main.md#221-233 @Main.md#1280-1296
- Real-time cloud previews and large-scale marketplace operations.@Main.md#1251-1296

## Open Questions & Follow-ups

1. **Framework expansion criteria** – Define adoption or revenue milestones that trigger React/Next.js investment.@Main.md#1251-1296
2. **Semantic diff format** – Finalise how ForgeGraph changes map to Git-friendly patches.@Main.md#221-233
3. **Black-box metadata standard** – Determine minimum metadata for custom widgets to remain editable in layout flows.@Main.md#456-577
4. **Security permissions model** – Specify manifest schema for package-level capabilities (network, storage, etc.).@Main.md#1251-1399
5. **Marketplace moderation** – Establish automated + human review processes to maintain quality.@Main.md#1251-3599

---

_This README is intended to stay extensible; append detailed specs, diagrams, or decision records per module as they mature._
