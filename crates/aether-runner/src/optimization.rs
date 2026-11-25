use super::{Error, MAX_ITEMS, data};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Interval {
    pub start: usize,
    pub end_exclusive: usize,
    pub sum: i32,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
pub struct OptimizationComparison {
    pub reference_sum: i32,
    pub baseline_sum: i32,
    pub candidate_sum: i32,
    pub baseline_is_optimal: bool,
}

pub fn optimization_input(seed: u32, items: u16) -> Result<Vec<i16>, Error> {
    if !(1..=MAX_ITEMS).contains(&items) {
        return Err(Error::Limits);
    }
    Ok(data(seed, items))
}

pub(crate) fn solve(values: &[i16]) -> (Interval, Interval) {
    let first = Interval {
        start: 0,
        end_exclusive: 1,
        sum: i32::from(values[0]),
    };
    let mut baseline = first.clone();
    let mut candidate = first.clone();
    let mut ending = first;
    for (index, value) in values.iter().copied().enumerate().skip(1) {
        let value = i32::from(value);
        if value > baseline.sum {
            baseline = Interval {
                start: index,
                end_exclusive: index + 1,
                sum: value,
            };
        }
        if ending.sum < 0 {
            ending.start = index;
            ending.sum = value;
        } else {
            ending.sum += value;
        }
        ending.end_exclusive = index + 1;
        if ending.sum > candidate.sum {
            candidate = ending.clone();
        }
    }
    (baseline, candidate)
}

/// Verify both submitted intervals and compute the exact nonempty optimum independently.
pub(crate) fn compare(
    values: &[i16],
    baseline: &Interval,
    candidate: &Interval,
) -> Result<OptimizationComparison, Error> {
    let valid_sum = |interval: &Interval| {
        values
            .get(interval.start..interval.end_exclusive)
            .filter(|slice| !slice.is_empty())
            .map(|slice| slice.iter().map(|value| i32::from(*value)).sum())
            == Some(interval.sum)
    };
    let largest_item = values.iter().copied().max().map(i32::from);
    if !valid_sum(baseline)
        || baseline.end_exclusive - baseline.start != 1
        || Some(baseline.sum) != largest_item
        || !valid_sum(candidate)
    {
        return Err(Error::Evidence);
    }
    // ponytail: enumerate intervals in O(n²) at n <= 128 for an independent
    // reference. Revisit the reference algorithm if the input cap grows.
    let mut reference_sum = i32::MIN;
    for start in 0..values.len() {
        let mut sum = 0;
        for value in &values[start..] {
            sum += i32::from(*value);
            reference_sum = reference_sum.max(sum);
        }
    }
    Ok(OptimizationComparison {
        reference_sum,
        baseline_sum: baseline.sum,
        candidate_sum: candidate.sum,
        baseline_is_optimal: baseline.sum == reference_sum,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn enumerated_reference_exposes_baseline_regret() {
        for (values, optimum, baseline_optimal) in [
            (vec![2, 3, -4, 2], 5, false),
            (vec![-5, -2, -3], -2, true),
            (vec![0, 0], 0, true),
            (vec![-4], -4, true),
        ] {
            let (baseline, candidate) = solve(&values);
            let comparison = compare(&values, &baseline, &candidate).unwrap();
            assert_eq!(comparison.reference_sum, optimum);
            assert_eq!(comparison.candidate_sum, optimum);
            assert_eq!(comparison.baseline_is_optimal, baseline_optimal);
        }
    }

    #[test]
    fn reference_rejects_fabricated_sums_and_invalid_ranges() {
        let values = [2, 3, -4, 2];
        let (baseline, mut candidate) = solve(&values);
        candidate.sum += 1;
        assert!(matches!(
            compare(&values, &baseline, &candidate),
            Err(Error::Evidence)
        ));
        candidate.start = usize::MAX;
        assert!(matches!(
            compare(&values, &baseline, &candidate),
            Err(Error::Evidence)
        ));
    }
}
