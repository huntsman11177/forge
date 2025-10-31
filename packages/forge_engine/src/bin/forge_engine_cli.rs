use clap::{Parser, Subcommand};
use forge_engine::{
    build_graphs_from_source, generate_manifest, get_renderer, read_graph, renderer_names,
    simulate_flow, AnalysisOutcome, AnalyzerService, EvalConfig, LogicError, LogicGraph,
    ManifestKind, RenderContext, RenderOptions, RiverpodAdapter,
};
use serde::Serialize;
use serde_json::{self, Value};
use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
    process,
};

#[derive(Debug, Parser)]
#[command(name = "forge_engine", about = "Forge engine CLI", version)]
struct Cli {
    /// Backwards-compatible shorthand for `parse --file`
    #[arg(long, global = true)]
    file: Option<PathBuf>,

    #[command(subcommand)]
    command: Option<Commands>,
}

fn run_simulate(
    flow_id: &str,
    graph_path: &Path,
    entry: Option<&str>,
    providers_path: Option<&Path>,
    output: Option<PathBuf>,
    max_steps: Option<usize>,
    max_trace: Option<usize>,
) -> Result<i32, String> {
    let graph = read_logic_graph(graph_path)?;
    let providers = match providers_path {
        Some(path) => Some(read_json_map(path)?),
        None => None,
    };

    let mut config = EvalConfig::default();
    if let Some(steps) = max_steps {
        config.max_steps = steps;
    }
    if let Some(trace) = max_trace {
        config.max_trace = trace;
    }

    let result = simulate_flow(&graph, flow_id, entry, providers.as_ref(), config)
        .map_err(|err| format_logic_error(err, flow_id))?;

    let summary = SimulationOutput {
        flow_id: flow_id.to_string(),
        entry: entry.map(ToString::to_string),
        result,
    };

    let json = serde_json::to_string_pretty(&summary)
        .map_err(|err| format!("Failed to serialize simulation result: {err}"))?;

    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|err| {
                format!(
                    "Failed to create output directory {}: {err}",
                    parent.display()
                )
            })?;
        }
        fs::write(&path, &json).map_err(|err| {
            format!(
                "Failed to write simulation output to {}: {err}",
                path.display()
            )
        })?;
    } else {
        println!("{json}");
    }

    Ok(0)
}

fn read_logic_graph(path: &Path) -> Result<LogicGraph, String> {
    let contents = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read logic graph {}: {err}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|err| format!("Failed to parse logic graph {}: {err}", path.display()))
}

fn read_json_map(path: &Path) -> Result<HashMap<String, Value>, String> {
    let contents = fs::read_to_string(path)
        .map_err(|err| format!("Failed to read providers {}: {err}", path.display()))?;
    serde_json::from_str(&contents)
        .map_err(|err| format!("Failed to parse providers {}: {err}", path.display()))
}

fn format_logic_error(err: LogicError, flow_id: &str) -> String {
    match err {
        LogicError::FlowNotFound(_) => format!("Flow '{flow_id}' was not found in the logic graph"),
        other => other.to_string(),
    }
}

#[derive(Debug, Serialize)]
struct SimulationOutput {
    flow_id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    entry: Option<String>,
    result: forge_engine::EvalResult,
}

#[derive(Debug, Subcommand)]
enum Commands {
    /// Parses a Dart file into Forge graphs (default behavior)
    Parse {
        #[arg(long)]
        file: PathBuf,
    },
    /// Runs the analyzer fallback and merge pipeline
    Analyze {
        #[arg(long)]
        file: PathBuf,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long, default_value_t = 0.5)]
        confidence: f32,
    },
    /// Simulates a logic flow and returns its evaluation trace
    Simulate {
        #[arg(long)]
        flow: String,
        #[arg(long)]
        graph: PathBuf,
        #[arg(long)]
        entry: Option<String>,
        #[arg(long)]
        providers: Option<PathBuf>,
        #[arg(long)]
        output: Option<PathBuf>,
        #[arg(long)]
        max_steps: Option<usize>,
        #[arg(long)]
        max_trace: Option<usize>,
    },
    /// Renders a Forge UI graph into target framework code
    Render {
        #[arg(long, short = 'f')]
        file: PathBuf,
        #[arg(long, short = 't', value_name = "FRAMEWORK")]
        framework: String,
        #[arg(long, short = 'o')]
        out_dir: Option<PathBuf>,
        #[arg(long)]
        emit_manifest: bool,
    },
}

#[derive(Debug, Serialize)]
struct AnalysisReport {
    outcomes: Vec<AnalysisOutcome>,
    total_conflicts: usize,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    match run_with_args(&args) {
        Ok(code) => process::exit(code),
        Err(err) => {
            eprintln!("{err}");
            process::exit(1);
        }
    }
}

fn run_with_args(args: &[String]) -> Result<i32, String> {
    let patched_args = normalize_legacy_args(args);
    let cli = Cli::try_parse_from(&patched_args).map_err(|e| e.to_string())?;

    match cli.command {
        Some(Commands::Parse { file }) => run_parse_file(&file),
        Some(Commands::Analyze {
            file,
            output,
            confidence,
        }) => run_analyze_file(&file, output, confidence),
        Some(Commands::Simulate {
            flow,
            graph,
            entry,
            providers,
            output,
            max_steps,
            max_trace,
        }) => run_simulate(
            &flow,
            &graph,
            entry.as_deref(),
            providers.as_deref(),
            output,
            max_steps,
            max_trace,
        ),
        Some(Commands::Render {
            file,
            framework,
            out_dir,
            emit_manifest,
        }) => run_render(&file, &framework, out_dir.as_deref(), emit_manifest),
        None => {
            let file = cli
                .file
                .ok_or_else(|| "Missing required argument --file or subcommand".to_string())?;
            run_parse_file(&file)
        }
    }
}

fn normalize_legacy_args(args: &[String]) -> Vec<String> {
    if args.len() > 1 {
        let first = args[1].as_str();
        let is_flag = first.starts_with('-');
        let is_help = matches!(first, "-h" | "--help" | "-V" | "--version");
        let is_known_cmd = matches!(first, "parse" | "analyze");
        if is_flag && !is_help && !is_known_cmd {
            let mut patched = Vec::with_capacity(args.len() + 1);
            patched.push(args[0].clone());
            patched.push("parse".to_string());
            patched.extend_from_slice(&args[1..]);
            return patched;
        }
    }
    args.to_vec()
}

fn run_parse_file(file: &Path) -> Result<i32, String> {
    let payload = execute_parse(file)?;
    println!("{payload}");
    Ok(0)
}

fn execute_parse(file: &Path) -> Result<String, String> {
    if !file.exists() {
        return Err(format!("Input file {} does not exist", file.display()));
    }

    let contents =
        fs::read_to_string(file).map_err(|e| format!("Failed to read {}: {e}", file.display()))?;

    let graphs = build_graphs_from_source(&contents);
    serde_json::to_string_pretty(&graphs).map_err(|e| format!("Failed to serialize graphs: {e}"))
}

fn run_analyze_file(file: &Path, output: Option<PathBuf>, confidence: f32) -> Result<i32, String> {
    let report = execute_analyze(file, confidence)?;
    let json = serde_json::to_string_pretty(&report)
        .map_err(|e| format!("Failed to serialize analysis report: {e}"))?;

    if let Some(path) = output {
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                format!(
                    "Failed to create output directory {}: {e}",
                    parent.display()
                )
            })?;
        }
        fs::write(&path, &json)
            .map_err(|e| format!("Failed to write analysis output to {}: {e}", path.display()))?;
    } else {
        println!("{json}");
    }

    if report.total_conflicts > 0 {
        eprintln!("⚠️  {} conflict(s) detected", report.total_conflicts);
        Ok(2)
    } else {
        Ok(0)
    }
}

fn execute_analyze(file: &Path, confidence: f32) -> Result<AnalysisReport, String> {
    if !file.exists() {
        return Err(format!("Input file {} does not exist", file.display()));
    }

    let contents =
        fs::read_to_string(file).map_err(|e| format!("Failed to read {}: {e}", file.display()))?;

    let graphs = build_graphs_from_source(&contents);
    if graphs.is_empty() {
        return Err("No graphs were generated from the input source.".to_string());
    }

    let service = AnalyzerService::default();
    let mut outcomes = Vec::new();

    for graph in graphs {
        let base_graph = graph.clone();
        let outcome = service.run(&contents, &base_graph, graph, confidence);
        outcomes.push(outcome);
    }

    let total_conflicts = outcomes
        .iter()
        .map(|outcome| outcome.merge.conflicts.len())
        .sum();

    Ok(AnalysisReport {
        outcomes,
        total_conflicts,
    })
}

fn run_render(
    file: &Path,
    framework: &str,
    out_dir: Option<&Path>,
    emit_manifest: bool,
) -> Result<i32, String> {
    let graph = read_graph(file).map_err(|err| err.to_string())?;

    let descriptor = get_renderer(framework).ok_or_else(|| {
        format!(
            "Unsupported framework '{framework}'. Supported frameworks: {}",
            format_supported_frameworks()
        )
    })?;

    let renderer = descriptor.instantiate();
    let state_adapter = RiverpodAdapter::new();
    let options = RenderOptions {
        pretty: true,
        include_comments: false,
        dialect: descriptor.dialect,
    };
    let ctx = RenderContext::new(0, &state_adapter, &options);

    let unit = renderer
        .render_tree(&graph.root, &ctx)
        .map_err(|err| err.message.clone())?;

    let output_code = unit.code;
    let dependencies = unit.dependencies;

    if let Some(dir) = out_dir {
        fs::create_dir_all(dir)
            .map_err(|err| format!("Failed to create output directory {}: {err}", dir.display()))?;
        let output_path = dir.join(format!("main.{}", descriptor.file_extension));
        fs::write(&output_path, &output_code)
            .map_err(|err| format!("Failed to write {}: {err}", output_path.display()))?;
        println!(
            "Rendered {} source to {}",
            descriptor.name,
            output_path.display()
        );

        if !dependencies.is_empty() {
            let deps_path = dir.join("dependencies.json");
            let deps_json = serde_json::to_string_pretty(&dependencies)
                .map_err(|err| format!("Failed to serialize dependencies: {err}"))?;
            fs::write(&deps_path, deps_json)
                .map_err(|err| format!("Failed to write {}: {err}", deps_path.display()))?;
            println!(
                "Saved renderer dependencies to {}. Install these packages in your project.",
                deps_path.display()
            );
        }

        if emit_manifest {
            if let Some(kind) = descriptor.manifest_kind {
                if let Some(manifest) = generate_manifest(kind, &dependencies) {
                    let manifest_path = dir.join(manifest.file_name);
                    fs::write(&manifest_path, manifest.contents).map_err(|err| {
                        format!("Failed to write {}: {err}", manifest_path.display())
                    })?;
                    println!(
                        "Wrote {} manifest to {}",
                        descriptor.name,
                        manifest_path.display()
                    );
                }
            } else {
                eprintln!(
                    "Manifest emission requested, but renderer '{}' does not define a manifest format.",
                    descriptor.name
                );
            }
        }
    } else {
        println!("{}", output_code);
        if !dependencies.is_empty() {
            eprintln!(
                "\nDependencies required for {}:\n{}\nInstall these packages before running your app.",
                descriptor.name,
                format_dependencies(&dependencies)
            );
        }

        if emit_manifest {
            if let Some(kind) = descriptor.manifest_kind {
                if let Some(manifest) = generate_manifest(kind, &dependencies) {
                    println!(
                        "\n--- {} ({}) ---\n{}",
                        manifest.file_name, descriptor.name, manifest.contents
                    );
                }
            } else {
                eprintln!(
                    "Manifest emission requested, but renderer '{}' does not define a manifest format.",
                    descriptor.name
                );
            }
        }
    }

    Ok(0)
}

fn format_supported_frameworks() -> String {
    let mut names = renderer_names();
    names.sort_unstable();
    names.join(", ")
}

fn format_dependencies(deps: &HashMap<String, String>) -> String {
    let mut entries: Vec<_> = deps.iter().collect();
    entries.sort_by(|a, b| a.0.cmp(b.0));
    entries
        .into_iter()
        .map(|(name, version)| format!("  - {name}@{version}"))
        .collect::<Vec<_>>()
        .join("\n")
}

// Re-export types needed by the binary when compiled separately.
pub use forge_engine::WidgetNode;

#[cfg(test)]
mod tests {
    use super::*;
    use forge_engine::ScreenGraph;

    #[test]
    fn parse_requires_file() {
        let args = vec!["cli".to_string()];
        let err = run_with_args(&args).unwrap_err();
        assert!(err.contains("--file"));
    }

    #[test]
    fn parse_reads_value() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        std::fs::write(
            temp_file.path(),
            r#"import 'package:flutter/widgets.dart';

class SimpleScreen extends StatelessWidget {
  const SimpleScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Text('Hi');
  }
}
"#,
        )
        .unwrap();

        let args = vec![
            "cli".to_string(),
            "--file".to_string(),
            temp_file.path().to_string_lossy().into_owned(),
        ];

        let exit_code = run_with_args(&args).unwrap();
        assert_eq!(exit_code, 0);
    }

    #[test]
    fn round_trip_serializes_graphs() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("demo.dart");
        std::fs::write(
            &file_path,
            r#"import 'package:flutter/widgets.dart';

class SimpleScreen extends StatelessWidget {
  const SimpleScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Text('Hi');
  }
}
"#,
        )
        .unwrap();

        let payload = execute_parse(&file_path).unwrap();
        let graphs: Vec<ScreenGraph> = serde_json::from_str(&payload).unwrap();
        assert_eq!(graphs.len(), 1);
        let serialized = serde_json::to_string(&graphs).unwrap();
        let deserialized: Vec<ScreenGraph> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(graphs, deserialized);
    }

    #[test]
    fn analyze_reports_conflicts() {
        let temp_dir = tempfile::tempdir().unwrap();
        let file_path = temp_dir.path().join("conflict.dart");
        std::fs::write(
            &file_path,
            r#"import 'package:flutter/widgets.dart';

class SampleScreen extends StatelessWidget {
  const SampleScreen({super.key});

  @override
  Widget build(BuildContext context) {
    return Text('Hello');
  }
}
"#,
        )
        .unwrap();

        let report = execute_analyze(&file_path, 0.1).unwrap();
        assert_eq!(report.outcomes.len(), 1);
        assert_eq!(report.total_conflicts, 1);
    }
}
