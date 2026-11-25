#![forbid(unsafe_code)]

//! Deterministic evaluation of fixed binary-classification inputs.

use std::fmt;

pub const FIXTURE_VERSION: &str = "classification-v1";
pub const EVALUATOR_VERSION: &str = "binary-accuracy-v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Accuracy {
    pub correct: usize,
    pub total: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvalError {
    Empty,
    LengthMismatch { labels: usize, predictions: usize },
    InvalidLabel { index: usize, label: u8 },
    InvalidPrediction { index: usize, prediction: u8 },
}

impl fmt::Display for EvalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Empty => write!(f, "classification requires at least one observation"),
            Self::LengthMismatch {
                labels,
                predictions,
            } => {
                write!(
                    f,
                    "{labels} labels and {predictions} predictions have different lengths"
                )
            }
            Self::InvalidLabel { index, label } => {
                write!(f, "label at index {index} must be 0 or 1, got {label}")
            }
            Self::InvalidPrediction { index, prediction } => {
                write!(
                    f,
                    "prediction at index {index} must be 0 or 1, got {prediction}"
                )
            }
        }
    }
}

impl std::error::Error for EvalError {}

/// Evaluate paired binary labels and predictions, returning exact counts.
///
/// The slices must have equal nonzero length and consist of 0 and 1.
pub fn evaluate(labels: &[u8], predictions: &[u8]) -> Result<Accuracy, EvalError> {
    if labels.len() != predictions.len() {
        return Err(EvalError::LengthMismatch {
            labels: labels.len(),
            predictions: predictions.len(),
        });
    }
    if labels.is_empty() {
        return Err(EvalError::Empty);
    }

    let mut correct = 0;
    for (index, (&label, &prediction)) in labels.iter().zip(predictions).enumerate() {
        if label > 1 {
            return Err(EvalError::InvalidLabel { index, label });
        }
        if prediction > 1 {
            return Err(EvalError::InvalidPrediction { index, prediction });
        }
        if label == prediction {
            correct += 1;
        }
    }

    Ok(Accuracy {
        correct,
        total: labels.len(),
    })
}

/// Return the immutable teaching fixture as (ground truth, predictions).
pub fn fixture() -> (&'static [u8], &'static [u8]) {
    (&[0, 1, 1, 0], &[0, 1, 0, 0])
}

#[cfg(test)]
mod tests {
    use super::{Accuracy, EvalError, evaluate, fixture};

    #[test]
    fn counts_fixture_and_boundary_accuracies() {
        let (labels, predictions) = fixture();
        assert_eq!(
            evaluate(labels, predictions),
            Ok(Accuracy {
                correct: 3,
                total: 4
            })
        );
        assert_eq!(
            evaluate(&[0, 1], &[0, 1]),
            Ok(Accuracy {
                correct: 2,
                total: 2
            })
        );
        assert_eq!(
            evaluate(&[0, 1], &[1, 0]),
            Ok(Accuracy {
                correct: 0,
                total: 2
            })
        );
    }

    #[test]
    fn rejects_unpaired_empty_and_non_binary_inputs() {
        assert_eq!(evaluate(&[], &[]), Err(EvalError::Empty));
        assert_eq!(
            evaluate(&[1], &[]),
            Err(EvalError::LengthMismatch {
                labels: 1,
                predictions: 0
            })
        );
        assert_eq!(
            evaluate(&[2], &[1]),
            Err(EvalError::InvalidLabel { index: 0, label: 2 })
        );
        assert_eq!(
            evaluate(&[1], &[2]),
            Err(EvalError::InvalidPrediction {
                index: 0,
                prediction: 2
            })
        );
    }
}
