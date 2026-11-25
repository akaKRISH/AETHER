use aether_engine::{
    Action, ActionKind, Budget, ResourceVec, ScheduleContext, ScoreWeights, schedule_with_trace,
};
use criterion::{BenchmarkId, Criterion, criterion_group, criterion_main};
use std::hint::black_box;
use std::time::Duration;

fn scheduler(c: &mut Criterion) {
    let budget = Budget::with_quarter_reserve(ResourceVec {
        wall_ms: 20_000,
        model_tokens: 2_000,
        attempts: 12,
        ..ResourceVec::default()
    });
    let mut group = c.benchmark_group("scheduler/trace");
    for count in [3_usize, 32, 128] {
        let candidates: Vec<_> = (0..count)
            .map(|index| Action {
                id: format!("action-{index:03}"),
                kind: [
                    ActionKind::Explore,
                    ActionKind::Replay,
                    ActionKind::Challenge,
                ][index % 3],
                estimated_cost: ResourceVec {
                    wall_ms: 120 + index as u64 * 23,
                    model_tokens: index as u64 * 5,
                    attempts: 1,
                    ..ResourceVec::default()
                },
                expected_gain: (index % 7) as u16 + 1,
                uncertainty_reduction: (index % 5) as u16,
                claim_importance: (index % 11) as u16,
                eligible: index % 7 != 6,
            })
            .collect();
        assert!(
            schedule_with_trace(
                &budget,
                &candidates,
                ScoreWeights::default(),
                ScheduleContext::default(),
            )
            .is_ok()
        );
        group.bench_with_input(
            BenchmarkId::from_parameter(count),
            &candidates,
            |bencher, input| {
                bencher.iter(|| {
                    schedule_with_trace(
                        black_box(&budget),
                        black_box(input),
                        ScoreWeights::default(),
                        ScheduleContext::default(),
                    )
                    .unwrap()
                });
            },
        );
    }
    group.finish();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .without_plots()
        .sample_size(20)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2));
    targets = scheduler
}
criterion_main!(benches);
