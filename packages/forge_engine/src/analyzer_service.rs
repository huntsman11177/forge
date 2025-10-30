use crate::{merge_screen_graphs, MergeOutcome, ScreenGraph};
use serde::Serialize;

/// Hybrid analyzer service that routes between the native parser and an
/// external analyzer based on confidence thresholds. When a fallback analyzer
/// is required, the service reconciles the quick parse graph with the analyzer
/// graph using the merge engine and returns the merged result alongside any
/// conflicts.
#[derive(Debug, Clone)]
pub struct AnalyzerService {
    confidence_threshold: f32,
}

impl AnalyzerService {
    /// Default confidence threshold used when none is supplied.
    pub const DEFAULT_CONFIDENCE_THRESHOLD: f32 = 0.7;

    /// Creates a new service with the provided threshold. Values outside the
    /// 0.0–1.0 range are clamped.
    pub fn new(confidence_threshold: f32) -> Self {
        Self {
            confidence_threshold: confidence_threshold.clamp(0.0, 1.0),
        }
    }

    /// Evaluates the provided confidence score and selects the processing
    /// strategy to follow.
    pub fn evaluate(&self, native_confidence: f32) -> AnalysisDecision {
        let strategy = if native_confidence >= self.confidence_threshold {
            AnalysisStrategy::Native
        } else {
            AnalysisStrategy::AnalyzerFallback
        };
        AnalysisDecision {
            strategy,
            native_confidence,
            threshold: self.confidence_threshold,
        }
    }

    /// Runs the hybrid analyzer flow, invoking the mocked analyzer when the
    /// fallback strategy is selected. Returns the merge outcome alongside the
    /// decision metadata.
    pub fn run(
        &self,
        source: &str,
        base_graph: &ScreenGraph,
        quick_graph: ScreenGraph,
        native_confidence: f32,
    ) -> AnalysisOutcome {
        let decision = self.evaluate(native_confidence);

        let mut quick_owned = quick_graph;
        let quick_ref = &quick_owned;
        let mut analyzer_graph_out: Option<ScreenGraph> = None;
        let mut analyzer_invoked = false;

        let merge = match decision.strategy {
            AnalysisStrategy::Native => merge_screen_graphs(base_graph, quick_ref, quick_ref),
            AnalysisStrategy::AnalyzerFallback => {
                let invocation = self.invoke_analyzer(source, quick_ref);
                analyzer_invoked = invocation.executed;
                let analyzer_graph = invocation.graph.unwrap_or_else(|| quick_ref.clone());
                let merge = merge_screen_graphs(base_graph, quick_ref, &analyzer_graph);
                analyzer_graph_out = Some(analyzer_graph);
                merge
            }
        };

        AnalysisOutcome {
            decision,
            analyzer_invoked,
            diagnostics: Vec::new(),
            quick_graph: quick_owned,
            analyzer_graph: analyzer_graph_out,
            merge,
        }
    }

    /// Mock external analyzer call. Currently returns a stub indicating that the
    /// analyzer would have been executed; this will be replaced with the real
    /// Dart analyzer integration.
    fn invoke_analyzer(&self, _source: &str, quick_graph: &ScreenGraph) -> AnalyzerInvocation {
        let mut analyzer_graph = quick_graph.clone();
        analyzer_graph.id = format!("{}__analyzer", quick_graph.id);

        AnalyzerInvocation {
            executed: true,
            graph: Some(analyzer_graph),
        }
    }
}

impl Default for AnalyzerService {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CONFIDENCE_THRESHOLD)
    }
}

/// Result of the decision phase showing which strategy should be used.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AnalysisDecision {
    pub strategy: AnalysisStrategy,
    pub native_confidence: f32,
    pub threshold: f32,
}

/// Outcome of running the hybrid analysis.
#[derive(Debug, Clone, PartialEq, Serialize)]
pub struct AnalysisOutcome {
    pub decision: AnalysisDecision,
    pub analyzer_invoked: bool,
    #[serde(skip_serializing_if = "Vec::is_empty", default)]
    pub diagnostics: Vec<String>,
    pub quick_graph: ScreenGraph,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub analyzer_graph: Option<ScreenGraph>,
    pub merge: MergeOutcome,
}

/// Strategy chosen for processing the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
pub enum AnalysisStrategy {
    Native,
    AnalyzerFallback,
}

/// Stub describing the analyzer invocation. Expands once the real analyzer is
/// wired in.
#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct AnalyzerInvocation {
    pub executed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub graph: Option<ScreenGraph>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{PropValue, WidgetNode};
    use serde_json::json;
    use std::collections::BTreeMap;

    fn make_graph(id: &str, value: &str) -> ScreenGraph {
        let mut props = BTreeMap::new();
        props.insert(
            "value".to_string(),
            PropValue::Literal {
                value: serde_json::Value::String(value.to_string()),
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

    #[test]
    fn evaluate_prefers_native_when_confidence_exceeds_threshold() {
        let service = AnalyzerService::default();
        let base = make_graph("Sample", "base");
        let quick = make_graph("Sample", "quick");
        let expected = quick.clone();

        let outcome = service.run("fn main(){}", &base, quick, 0.9);
        assert_eq!(outcome.decision.strategy, AnalysisStrategy::Native);
        assert!(!outcome.analyzer_invoked);
        assert_eq!(outcome.merge.screen, expected);
        assert!(outcome.merge.conflicts.is_empty());
    }

    #[test]
    fn evaluate_invokes_analyzer_when_confidence_is_low() {
        let service = AnalyzerService::new(0.8);
        let base = make_graph("Sample", "base");
        let quick = make_graph("Sample", "quick");
        let outcome = service.run("class Demo {}", &base, quick, 0.5);
        assert_eq!(
            outcome.decision.strategy,
            AnalysisStrategy::AnalyzerFallback
        );
        assert!(outcome.analyzer_invoked);
        assert_eq!(outcome.merge.conflicts.len(), 1);
        let conflict = &outcome.merge.conflicts[0];
        assert_eq!(conflict.path, "screen.id");
        assert_eq!(conflict.right, Some(json!("Sample__analyzer")));
    }

    #[test]
    fn threshold_is_clamped_to_valid_range() {
        let service = AnalyzerService::new(2.5);
        assert_eq!(service.confidence_threshold, 1.0);
        let low_service = AnalyzerService::new(-1.0);
        assert_eq!(low_service.confidence_threshold, 0.0);
    }
}
