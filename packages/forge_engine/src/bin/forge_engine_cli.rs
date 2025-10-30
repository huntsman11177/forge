use std::{env, fs, path::PathBuf, process};

use forge_engine::build_graphs_from_source;

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        process::exit(1);
    }
}

fn run() -> Result<(), String> {
    let args: Vec<String> = env::args().collect();
    let file_path = parse_args(&args)?;

    let contents = fs::read_to_string(&file_path)
        .map_err(|e| format!("Failed to read {}: {e}", file_path.display()))?;

    let graphs = build_graphs_from_source(&contents);
    let payload = serde_json::to_string_pretty(&graphs)
        .map_err(|e| format!("Failed to serialize graphs: {e}"))?;

    println!("{payload}");
    Ok(())
}

fn parse_args(args: &[String]) -> Result<PathBuf, String> {
    let mut file: Option<PathBuf> = None;
    let mut index = 1;
    while index < args.len() {
        match args[index].as_str() {
            "--file" => {
                index += 1;
                if index >= args.len() {
                    return Err("--file requires a value".to_string());
                }
                file = Some(PathBuf::from(&args[index]));
            }
            other => {
                return Err(format!("Unknown argument: {other}"));
            }
        }
        index += 1;
    }

    let Some(path) = file else {
        return Err("Missing required argument --file".to_string());
    };

    if !path.exists() {
        return Err(format!("Input file {} does not exist", path.display()));
    }

    Ok(path)
}

// Re-export types needed by the binary when compiled separately.
pub use forge_engine::WidgetNode;

#[cfg(test)]
mod tests {
    use super::*;
    use forge_engine::ScreenGraph;

    #[test]
    fn parse_args_requires_file() {
        let args = vec!["cli".to_string()];
        assert!(parse_args(&args).is_err());
    }

    #[test]
    fn parse_args_reads_value() {
        let temp_file = tempfile::NamedTempFile::new().unwrap();
        let path = temp_file.path().to_path_buf();
        let args = vec![
            "cli".to_string(),
            "--file".to_string(),
            path.to_string_lossy().into_owned(),
        ];
        let result = parse_args(&args).unwrap();
        assert_eq!(result, path);
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

        let contents = fs::read_to_string(&file_path).unwrap();
        let graphs = build_graphs_from_source(&contents);
        assert_eq!(graphs.len(), 1);
        let serialized = serde_json::to_string(&graphs).unwrap();
        let deserialized: Vec<ScreenGraph> = serde_json::from_str(&serialized).unwrap();
        assert_eq!(graphs, deserialized);
    }
}
