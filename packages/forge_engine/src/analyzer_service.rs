/// Hybrid analyzer service that routes between the native parser and an
/// external analyzer based on confidence thresholds.
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
    /// fallback strategy is selected. Returns both the decision and whether the
    /// analyzer was triggered.
    pub fn run(&self, source: &str, native_confidence: f32) -> AnalysisOutcome {
        let decision = self.evaluate(native_confidence);
        let analyzer_invoked = match decision.strategy {
            AnalysisStrategy::Native => false,
            AnalysisStrategy::AnalyzerFallback => self.invoke_analyzer(source).executed,
        };

        AnalysisOutcome {
            decision,
            analyzer_invoked,
        }
    }

    /// Mock external analyzer call. Currently returns a stub indicating that the
    /// analyzer would have been executed; this will be replaced with the real
    /// Dart analyzer integration.
    fn invoke_analyzer(&self, _source: &str) -> AnalyzerInvocation {
        AnalyzerInvocation { executed: true }
    }
}

impl Default for AnalyzerService {
    fn default() -> Self {
        Self::new(Self::DEFAULT_CONFIDENCE_THRESHOLD)
    }
}

/// Result of the decision phase showing which strategy should be used.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisDecision {
    pub strategy: AnalysisStrategy,
    pub native_confidence: f32,
    pub threshold: f32,
}

/// Outcome of running the hybrid analysis.
#[derive(Debug, Clone, PartialEq)]
pub struct AnalysisOutcome {
    pub decision: AnalysisDecision,
    pub analyzer_invoked: bool,
}

/// Strategy chosen for processing the source file.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AnalysisStrategy {
    Native,
    AnalyzerFallback,
}

/// Stub describing the analyzer invocation. Expands once the real analyzer is
/// wired in.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnalyzerInvocation {
    pub executed: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evaluate_prefers_native_when_confidence_exceeds_threshold() {
        let service = AnalyzerService::default();
        let outcome = service.run("fn main(){}", 0.9);
        assert_eq!(outcome.decision.strategy, AnalysisStrategy::Native);
        assert!(!outcome.analyzer_invoked);
    }

    #[test]
    fn evaluate_invokes_analyzer_when_confidence_is_low() {
        let service = AnalyzerService::new(0.8);
        let outcome = service.run("class Demo {}", 0.5);
        assert_eq!(outcome.decision.strategy, AnalysisStrategy::AnalyzerFallback);
        assert!(outcome.analyzer_invoked);
    }

    #[test]
    fn threshold_is_clamped_to_valid_range() {
        let service = AnalyzerService::new(2.5);
        assert_eq!(service.confidence_threshold, 1.0);
        let low_service = AnalyzerService::new(-1.0);
        assert_eq!(low_service.confidence_threshold, 0.0);
    }
}
