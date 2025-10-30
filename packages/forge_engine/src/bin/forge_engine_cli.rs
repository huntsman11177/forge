use clap::{Parser, Subcommand};
use forge_engine::{build_graphs_from_source, AnalysisOutcome, AnalyzerService};
use serde::Serialize;
use std::{
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
