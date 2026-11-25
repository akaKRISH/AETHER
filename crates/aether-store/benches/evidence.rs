use aether_store::{BudgetUnits, NewCampaign, NewClaim, Store, StoreError};
use criterion::{Criterion, criterion_group, criterion_main};
use serde_json::json;
use std::hint::black_box;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const CAMPAIGN: &str = "bench-evidence";

fn create_campaign(store: &mut Store) {
    let hash = "1".repeat(64);
    store
        .create_campaign(&NewCampaign {
            id: CAMPAIGN.into(),
            title: "Evidence benchmark".into(),
            objective: "Measure the local evidence control plane".into(),
            protocol_hash: hash.clone(),
            policy_hash: hash.clone(),
            evaluator_hash: hash,
            requires_replay: true,
            caps: BudgetUnits::default(),
        })
        .unwrap();
}

fn evidence(c: &mut Criterion) {
    let mut graph = Store::open_in_memory().unwrap();
    create_campaign(&mut graph);
    for index in 0..64 {
        graph
            .add_claim(&NewClaim {
                id: format!("claim-{index}"),
                campaign_id: CAMPAIGN.into(),
                statement: "Benchmark fixture".into(),
            })
            .unwrap();
        if index > 0 {
            graph
                .add_dependency(&format!("claim-{index}"), &format!("claim-{}", index - 1))
                .unwrap();
        }
    }
    assert_eq!(graph.snapshot(CAMPAIGN).unwrap().dependencies.len(), 63);
    c.bench_function("evidence/cycle_check_chain_64", |b| {
        b.iter(|| {
            assert!(matches!(
                graph.add_dependency(black_box("claim-0"), black_box("claim-63")),
                Err(StoreError::Conflict("claim dependency cycle"))
            ));
        });
    });
    c.bench_function("evidence/combined_snapshot_chain_64", |b| {
        b.iter(|| graph.snapshot(black_box(CAMPAIGN)).unwrap());
    });

    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let path = std::env::temp_dir().join(format!(
        "aether-bench-{}-{nonce}.sqlite",
        std::process::id()
    ));
    std::fs::OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&path)
        .unwrap();
    let mut disk = Store::open(&path).unwrap();
    create_campaign(&mut disk);
    c.bench_function("evidence/durable_decision_append", |b| {
        b.iter(|| {
            disk.record_decision(black_box(CAMPAIGN), json!({"action":"explore"}))
                .unwrap();
        });
    });
    drop(disk);
    std::fs::remove_file(path).unwrap();
}

criterion_group! {
    name = benches;
    config = Criterion::default()
        .without_plots()
        .sample_size(20)
        .warm_up_time(Duration::from_secs(1))
        .measurement_time(Duration::from_secs(2));
    targets = evidence
}
criterion_main!(benches);
