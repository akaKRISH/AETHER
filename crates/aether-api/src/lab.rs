use aether_engine::{
    Action, ActionKind, Budget, Decision, ReservationOutcome, ResourceVec, ScheduleContext,
    ScoreWeights, StopReason, schedule_with_trace,
};
use aether_runner::{MAX_OUTPUT, Output, SortAlgorithm, Task, evaluate, sha256, supervise};
use aether_store::{
    Action as StoredAction, ArtifactInput, AttemptCompletion, AttemptOutcome, BudgetUnits,
    Campaign, CampaignState, EvidenceRelation, EvidenceTarget, MetricInput, NewAttempt,
    NewCampaign, NewClaim, NewContradiction, NewEvidence, NewExperiment, NewHypothesis,
    NewReplication, NewRun, NewVerification, Store, VerificationVerdict,
};
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};
use std::collections::{BTreeMap, BTreeSet};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use thiserror::Error;

const ACTION_LIMIT_MS: u64 = 2_000;
const POLICY: &str = "aether-reviewed-sort-ecar/v1";
static STAGE_SEQUENCE: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Error)]
pub enum Error {
    #[error("{0}")]
    Input(String),
    #[error("campaign or artifact was not found")]
    Missing,
    #[error("{0}")]
    Conflict(String),
    #[error("{0}")]
    Integrity(String),
    #[error("storage: {0}")]
    Storage(String),
    #[error("scheduling: {0}")]
    Schedule(String),
    #[error("I/O: {0}")]
    Io(#[from] io::Error),
    #[error("JSON: {0}")]
    Json(#[from] serde_json::Error),
}

type Result<T> = std::result::Result<T, Error>;

fn storage<T, E: std::fmt::Display>(result: std::result::Result<T, E>) -> Result<T> {
    result.map_err(|error| Error::Storage(error.to_string()))
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct PlanRequest {
    pub id: String,
    pub title: String,
    pub objective: String,
    pub seed: u32,
    pub items: u16,
    pub algorithm: SortAlgorithm,
    pub attempts: u64,
    pub wall_ms: u64,
}

impl PlanRequest {
    pub fn example(id: String, seed: u32) -> Self {
        Self {
            id,
            title: "Seeded sorting campaign".into(),
            objective: "Evaluate a reviewed sorting candidate against an independent reference, replay, and challenge".into(),
            seed,
            items: 128,
            algorithm: SortAlgorithm::Insertion,
            attempts: 5,
            wall_ms: 12_000,
        }
    }

    fn validate(&self) -> Result<()> {
        if !valid_id(&self.id)
            || self.title.trim().is_empty()
            || self.title.len() > 120
            || self.objective.trim().is_empty()
            || self.objective.len() > 512
            || !(1..=128).contains(&self.items)
            || !(1..=12).contains(&self.attempts)
            || !(2_000..=30_000).contains(&self.wall_ms)
        {
            return Err(Error::Input("campaign requires an ASCII id (1–32 lowercase letters/digits/hyphens), bounded objective, 1–128 items, 1–12 attempts and 2000–30000 ms".into()));
        }
        Ok(())
    }

    fn task(&self, challenge: bool) -> Task {
        Task::Sort {
            seed: if challenge {
                self.seed ^ 0x854a_6817
            } else {
                self.seed
            },
            items: self.items,
            algorithm: self.algorithm,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct FrozenPlan {
    version: String,
    evaluator_hash: String,
    policy_hash: String,
    plan: PlanRequest,
}

pub fn valid_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= 32
        && id.as_bytes()[0].is_ascii_alphanumeric()
        && id
            .bytes()
            .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || byte == b'-')
}

pub fn valid_digest(hash: &str) -> bool {
    hash.len() == 64
        && hash
            .bytes()
            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
}

#[derive(Debug, Clone)]
pub struct Lab {
    root: PathBuf,
    executable: PathBuf,
    evaluator_hash: String,
}

impl Lab {
    pub fn open(root: PathBuf) -> Result<Self> {
        fs::create_dir_all(root.join("protocols"))?;
        fs::create_dir_all(root.join("supervisors"))?;
        let root = root.canonicalize()?;
        let executable = std::env::current_exe()?;
        let evaluator_hash = sha256(&fs::read(&executable)?);
        let lab = Self {
            root,
            executable,
            evaluator_hash,
        };
        lab.store()?;
        lab.seed_catalog()?;
        Ok(lab)
    }

    fn store(&self) -> Result<Store> {
        storage(Store::open(self.root.join("laboratory.sqlite3")))
    }

    fn get(&self, store: &Store, id: &str) -> Result<Campaign> {
        if !valid_id(id) {
            return Err(Error::Input("invalid campaign id".into()));
        }
        storage(store.campaign(id))?.ok_or(Error::Missing)
    }

    pub fn campaigns(&self) -> Result<Vec<Campaign>> {
        storage(self.store()?.campaigns())
    }

    pub fn campaigns_page(&self, after: Option<&str>, limit: usize) -> Result<Vec<Campaign>> {
        if after.is_some_and(|cursor| !valid_id(cursor)) || !(1..=512).contains(&limit) {
            return Err(Error::Input("invalid campaign cursor or page size".into()));
        }
        storage(self.store()?.campaigns_page(after, limit))
    }

    pub fn recover(&self, id: &str) -> Result<usize> {
        let _supervisor = self.supervisor_lock(id)?;
        let mut store = self.store()?;
        let campaign = self.get(&store, id)?;
        if !matches!(
            campaign.state,
            CampaignState::Running | CampaignState::Paused
        ) {
            return Err(Error::Conflict(
                "only interrupted running or paused campaigns can be reconciled".into(),
            ));
        }
        let expired = storage(store.recover_expired_in(id))?;
        let snapshot = storage(store.snapshot(id))?;
        if snapshot
            .attempts
            .iter()
            .any(|attempt| attempt.outcome.is_none())
        {
            return Err(Error::Conflict("worker lease has not expired yet".into()));
        }
        storage(store.record_decision(id, json!({"recovery": "interrupted supervisor; evidence retained", "expired_leases": expired})))?;
        storage(store.transition_campaign(id, CampaignState::Failed))?;
        Ok(expired)
    }

    fn supervisor_lock(&self, id: &str) -> Result<File> {
        if !valid_id(id) {
            return Err(Error::Input("invalid campaign id".into()));
        }
        let path = self.root.join("supervisors").join(format!("{id}.lock"));
        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(false)
            .open(path)?;
        file.try_lock().map_err(|error| {
            Error::Conflict(format!("cannot acquire campaign supervisor lock: {error}"))
        })?;
        Ok(file)
    }

    fn catalog(&self) -> Result<crate::literature::CitationCatalog> {
        storage(crate::literature::CitationCatalog::open(
            self.root.join("literature.sqlite3"),
        ))
    }

    fn seed_catalog(&self) -> Result<()> {
        let sources = [
            (
                "10.21203/rs.3.rs-11035327/v1",
                "CAVEAT: Reliability-aware active verification under correlated errors for multi-agent LLM reasoning",
                2026,
                "Research Square",
            ),
            (
                "10.48550/arXiv.2610.18449",
                "Autonomous Science Bench: Evaluating AI Agents for Autonomous Scientific Discovery",
                2026,
                "arXiv",
            ),
            (
                "10.48550/arXiv.2504.01848",
                "PaperBench: Evaluating AI's Ability to Replicate AI Research",
                2025,
                "arXiv",
            ),
        ];
        let records = sources
            .iter()
            .map(|(doi, title, year, venue)| crate::literature::Citation {
                doi: (*doi).into(),
                title: (*title).into(),
                authors: Vec::new(),
                published_year: Some(*year),
                container_title: Some((*venue).into()),
                license_urls: Vec::new(),
                provenance: crate::literature::CitationProvenance::default(),
            })
            .collect();
        self.import_citations(records)?;
        Ok(())
    }

    pub fn citations(&self, query: &str) -> Result<Vec<crate::literature::Citation>> {
        storage(self.catalog()?.search(query, 20))
    }

    pub fn import_citations(&self, citations: Vec<crate::literature::Citation>) -> Result<usize> {
        storage(self.catalog()?.import_manual(citations))
    }

    pub fn create(&self, plan: &PlanRequest) -> Result<Campaign> {
        plan.validate()?;
        let frozen = FrozenPlan {
            version: "aether.protocol/1".into(),
            evaluator_hash: self.evaluator_hash.clone(),
            policy_hash: sha256(POLICY.as_bytes()),
            plan: plan.clone(),
        };
        let bytes = serde_json::to_vec(&frozen)?;
        let digest = sha256(&bytes);
        self.publish_protocol(&digest, &bytes)?;
        let mut store = self.store()?;
        let campaign = NewCampaign {
            id: plan.id.clone(),
            title: plan.title.clone(),
            objective: plan.objective.clone(),
            protocol_hash: digest,
            policy_hash: frozen.policy_hash,
            evaluator_hash: frozen.evaluator_hash,
            requires_replay: true,
            caps: BudgetUnits {
                wall_ms: plan.wall_ms,
                attempts: plan.attempts,
                ..BudgetUnits::default()
            },
        };
        if let Some(existing) = storage(store.campaign(&plan.id))? {
            return if existing.spec == campaign {
                Ok(existing)
            } else {
                Err(Error::Conflict(
                    "campaign id already belongs to another frozen plan".into(),
                ))
            };
        }
        storage(store.create_campaign(&campaign))?;
        self.get(&store, &plan.id)
    }

    fn publish_protocol(&self, digest: &str, bytes: &[u8]) -> Result<()> {
        let directory = self.root.join("protocols");
        let path = directory.join(format!("{digest}.json"));
        if path.try_exists()? {
            return if self.protocol_bytes(digest)? == bytes {
                Ok(())
            } else {
                Err(Error::Integrity("protocol digest conflict".into()))
            };
        }
        let temporary = directory.join(format!(
            ".stage-{}-{}",
            std::process::id(),
            STAGE_SEQUENCE.fetch_add(1, Ordering::Relaxed)
        ));
        let mut file = OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(&temporary)?;
        let publication = (|| -> io::Result<()> {
            file.write_all(bytes)?;
            file.sync_all()?;
            fs::hard_link(&temporary, &path)?;
            Ok(())
        })();
        drop(file);
        let cleanup = fs::remove_file(&temporary);
        match publication {
            Ok(()) => cleanup.map_err(Error::from),
            Err(error)
                if error.kind() == io::ErrorKind::AlreadyExists
                    && self.protocol_bytes(digest)? == bytes =>
            {
                cleanup.map_err(Error::from)
            }
            Err(error) => Err(error.into()),
        }
    }

    fn protocol_bytes(&self, digest: &str) -> Result<Vec<u8>> {
        if !valid_digest(digest) {
            return Err(Error::Integrity("invalid stored protocol digest".into()));
        }
        let path = self.root.join("protocols").join(format!("{digest}.json"));
        if !fs::symlink_metadata(&path)?.file_type().is_file() {
            return Err(Error::Integrity("invalid protocol file type".into()));
        }
        let mut bytes = Vec::new();
        File::open(path)?.take(4097).read_to_end(&mut bytes)?;
        if bytes.len() > 4096 || sha256(&bytes) != digest {
            return Err(Error::Integrity("frozen protocol bytes changed".into()));
        }
        Ok(bytes)
    }

    pub fn approve(&self, id: &str, digest: &str) -> Result<Campaign> {
        let mut store = self.store()?;
        let campaign = self.get(&store, id)?;
        if campaign.spec.protocol_hash != digest {
            return Err(Error::Conflict(
                "approval hash does not match frozen protocol".into(),
            ));
        }
        self.protocol_bytes(digest)?;
        storage(store.approve_campaign(id, digest))?;
        self.get(&store, id)
    }

    pub fn snapshot(&self, id: &str) -> Result<Value> {
        let store = self.store()?;
        self.get(&store, id)?;
        let mut snapshot = serde_json::to_value(storage(store.snapshot(id))?)?;
        snapshot["events"] = serde_json::to_value(storage(store.events(id))?)?;
        Ok(snapshot)
    }

    pub fn events(&self, id: &str, after: u64) -> Result<Vec<Value>> {
        let store = self.store()?;
        self.get(&store, id)?;
        Ok(storage(store.events_page(id, after, 64))?
            .into_iter()
            .map(serde_json::to_value)
            .collect::<std::result::Result<Vec<_>, _>>()?)
    }

    pub fn artifact(&self, hash: &str) -> Result<Vec<u8>> {
        if !valid_digest(hash) {
            return Err(Error::Input("invalid artifact SHA-256".into()));
        }
        let bytes = storage(self.store()?.artifact(hash))?.ok_or(Error::Missing)?;
        if sha256(&bytes) != hash {
            return Err(Error::Integrity("stored artifact bytes changed".into()));
        }
        Ok(bytes)
    }

    pub fn report(&self, id: &str) -> Result<Value> {
        let store = self.store()?;
        let campaign = self.get(&store, id)?;
        let protocol: Value =
            serde_json::from_slice(&self.protocol_bytes(&campaign.spec.protocol_hash)?)?;
        let data = self.snapshot(id)?;
        let mut digests = BTreeSet::new();
        let refs = data
            .get("artifacts")
            .and_then(Value::as_array)
            .into_iter()
            .flatten();
        for artifact in refs {
            let hash = artifact["sha256"]
                .as_str()
                .ok_or_else(|| Error::Integrity("invalid artifact metadata".into()))?;
            digests.insert(hash);
        }
        for attempt in data["attempts"].as_array().into_iter().flatten() {
            if let Some(hash) = attempt["manifest_sha256"].as_str() {
                digests.insert(hash);
            }
        }
        let mut blobs = BTreeMap::new();
        let mut total = 0usize;
        for hash in digests {
            let bytes = self.artifact(hash)?;
            total = total.saturating_add(bytes.len());
            if total > 4_194_304 {
                return Err(Error::Integrity(
                    "report exceeds the local evidence bundle size limit".into(),
                ));
            }
            blobs.insert(hash, bytes);
        }
        Ok(json!({
            "version": "aether.campaign-report/1",
            "scope": "local development experiment on reviewed synthetic sorting inputs",
            "protocol": protocol,
            "protocol_hash": campaign.spec.protocol_hash,
            "evidence": data,
            "blobs_by_sha256": blobs,
            "limitations": ["The frozen comparison covers the original and one challenge seed.", "Replay uses a fresh process and the same pinned executable.", "No held-out scientific study is implied."]
        }))
    }

    pub fn cancel_idle(&self, id: &str) -> Result<Campaign> {
        let mut store = self.store()?;
        let current = self.get(&store, id)?;
        if current.state == CampaignState::Cancelled {
            return Ok(current);
        }
        if !matches!(
            current.state,
            CampaignState::Draft | CampaignState::Approved | CampaignState::Paused
        ) {
            return Err(Error::Conflict(
                "campaign is not cancellable without its current supervisor".into(),
            ));
        }
        storage(store.transition_campaign(id, CampaignState::Cancelled))?;
        self.get(&store, id)
    }

    pub fn run(&self, id: &str, cancelled: &AtomicBool) -> Result<Value> {
        let _supervisor = self.supervisor_lock(id)?;
        let mut store = self.store()?;
        let campaign = self.get(&store, id)?;
        if campaign.state != CampaignState::Approved {
            return Err(Error::Conflict(
                "campaign must be approved and ready".into(),
            ));
        }
        if campaign.spec.evaluator_hash != self.evaluator_hash
            || campaign.spec.policy_hash != sha256(POLICY.as_bytes())
        {
            return Err(Error::Conflict(
                "frozen campaign pins a different evaluator or policy build".into(),
            ));
        }
        let frozen: FrozenPlan =
            serde_json::from_slice(&self.protocol_bytes(&campaign.spec.protocol_hash)?)?;
        if frozen.version != "aether.protocol/1"
            || frozen.evaluator_hash != campaign.spec.evaluator_hash
            || frozen.policy_hash != campaign.spec.policy_hash
            || self.evaluator_hash != sha256(&fs::read(&self.executable)?)
        {
            return Err(Error::Integrity(
                "protocol or executable does not match the approved build".into(),
            ));
        }
        let plan = frozen.plan;
        plan.validate()?;
        if plan.id != id {
            return Err(Error::Integrity("stored protocol campaign mismatch".into()));
        }
        storage(store.transition_campaign(id, CampaignState::Running))?;
        match self.run_approved(&mut store, &campaign, &plan, cancelled) {
            Ok(state) => storage(store.transition_campaign(id, state))?,
            Err(error) => {
                storage(store.record_decision(id, json!({"failure": error.to_string()})))?;
                storage(store.transition_campaign(id, CampaignState::Failed))?;
                return Err(error);
            }
        }
        self.report(id)
    }

    fn run_approved(
        &self,
        store: &mut Store,
        campaign: &Campaign,
        plan: &PlanRequest,
        cancelled: &AtomicBool,
    ) -> Result<CampaignState> {
        let id = &plan.id;
        let main_claim = format!("{id}-correct");
        let hypothesis = format!("{id}-hypothesis");
        let experiment = format!("{id}-sort");
        let statement = format!(
            "The {:?} candidate exactly sorts {} generated integers on seeds {} and {} and replays the original output.",
            plan.algorithm,
            plan.items,
            plan.seed,
            plan.seed ^ 0x854a_6817
        );
        storage(store.add_hypothesis(&NewHypothesis {
            id: hypothesis.clone(),
            campaign_id: id.clone(),
            statement: statement.clone(),
        }))?;
        storage(store.add_claim(&NewClaim {
            id: main_claim.clone(),
            campaign_id: id.clone(),
            statement,
        }))?;
        let followup = format!("{id}-baseline");
        storage(store.add_claim(&NewClaim {
            id: followup.clone(),
            campaign_id: id.clone(),
            statement:
                "A follow-up study can use this protocol to compare alternative variants.".into(),
        }))?;
        storage(store.add_dependency(&followup, &main_claim))?;
        storage(store.add_experiment(&NewExperiment {
            id: experiment.clone(),
            campaign_id: id.clone(),
            hypothesis_id: Some(hypothesis),
            task_key: "seeded-sort-v1".into(),
            definition_hash: campaign.spec.protocol_hash.clone(),
        }))?;
        let mut budget = Budget::with_quarter_reserve(ResourceVec {
            wall_ms: plan.wall_ms,
            attempts: plan.attempts,
            output_bytes: plan.attempts * MAX_OUTPUT,
            ..ResourceVec::default()
        });
        let estimate = ResourceVec {
            wall_ms: ACTION_LIMIT_MS,
            attempts: 1,
            output_bytes: MAX_OUTPUT,
            ..ResourceVec::default()
        };
        let mut accepted: [Option<(String, Output)>; 3] = [None, None, None];
        let mut failed = false;

        loop {
            let objective_met = accepted.iter().all(Option::is_some);
            let graph = storage(store.snapshot(id))?;
            let mut dependent = BTreeSet::from([main_claim.as_str()]);
            loop {
                let before = dependent.len();
                for edge in &graph.dependencies {
                    if dependent.contains(edge.prerequisite_id.as_str()) {
                        dependent.insert(edge.claim_id.as_str());
                    }
                }
                if dependent.len() == before {
                    break;
                }
            }
            let importance = (dependent.len().saturating_sub(1)).min(12) as u16;
            let candidates = [
                candidate(
                    "explore",
                    ActionKind::Explore,
                    !failed && accepted[0].is_none(),
                    estimate,
                    [9, 2, 4],
                ),
                candidate(
                    "replay",
                    ActionKind::Replay,
                    !failed && accepted[0].is_some() && accepted[1].is_none(),
                    estimate,
                    [5, 10, 7 + importance],
                ),
                candidate(
                    "challenge",
                    ActionKind::Challenge,
                    !failed && accepted[0].is_some() && accepted[2].is_none(),
                    estimate,
                    [5, 8, 7 + importance],
                ),
            ];
            let trace = schedule_with_trace(
                &budget,
                &candidates,
                ScoreWeights::default(),
                ScheduleContext {
                    cancelled: cancelled.load(Ordering::Acquire),
                    objective_met,
                },
            )
            .map_err(|error| Error::Schedule(error.to_string()))?;
            storage(store.record_decision(id, json!({"policy": POLICY, "used": budget.used(), "remaining": budget.remaining(), "dependent_claims": importance, "candidates": candidates, "trace": trace})))?;
            let Decision::Run { action_id, kind } = trace.decision else {
                let Decision::Stop(reason) = trace.decision else {
                    unreachable!()
                };
                let verdict = if objective_met && reason != StopReason::Cancelled {
                    VerificationVerdict::Verified
                } else if failed {
                    VerificationVerdict::Rejected
                } else {
                    VerificationVerdict::Inconclusive
                };
                storage(store.record_verification(&NewVerification {
                    id: format!("{id}-verdict"), claim_id: main_claim,
                    verdict, rule_hash: campaign.spec.protocol_hash.clone(), evaluator_hash: campaign.spec.evaluator_hash.clone(),
                    replay_attempt_id: accepted[1].as_ref().map(|(attempt, _)| attempt.clone()),
                    reason: format!("Scheduler stopped with {reason:?}; frozen predicates evaluated against stored evidence"),
                }))?;
                return Ok(if reason == StopReason::Cancelled {
                    CampaignState::Cancelled
                } else {
                    CampaignState::Completed
                });
            };

            let (index, action) = match kind {
                ActionKind::Explore => (0, StoredAction::Explore),
                ActionKind::Replay => (1, StoredAction::Replay),
                ActionKind::Challenge => (2, StoredAction::Challenge),
            };
            let run_id = format!("{id}-{action_id}");
            let attempt = format!("{run_id}-1");
            if budget
                .try_reserve(&attempt, kind, estimate)
                .map_err(|error| Error::Schedule(error.to_string()))?
                != ReservationOutcome::Created
            {
                return Err(Error::Conflict("attempt already recorded".into()));
            }
            let task = plan.task(index == 2);
            let input_bytes = serde_json::to_vec(&task)?;
            let input_hash = sha256(&input_bytes);
            storage(store.add_run(&NewRun {
                id: run_id.clone(),
                experiment_id: experiment.clone(),
                input_hash: input_hash.clone(),
                parent_run_id: (index != 0).then(|| format!("{id}-explore")),
                action,
                seed: u64::from(if index == 2 {
                    plan.seed ^ 0x854a_6817
                } else {
                    plan.seed
                }),
            }))?;
            let started_at = now_ms()?;
            let lease_token =
                sha256(format!("{attempt}-{started_at}-{}", std::process::id()).as_bytes());
            let protected = if kind == ActionKind::Explore {
                units(budget.unconsumed_verification_reserve())
            } else {
                BudgetUnits::default()
            };
            storage(store.start_attempt_with_reserve(
                &NewAttempt {
                    id: attempt.clone(),
                    run_id,
                    lease_token: lease_token.clone(),
                    lease_expires_at_ms: started_at.saturating_add(15_000),
                    estimate: units(estimate),
                },
                protected,
            ))?;
            let started = Instant::now();
            // Reserve margin for process polling and cleanup. An actual overrun still fails closed.
            let observation = supervise(
                &self.executable,
                &task,
                Duration::from_millis(ACTION_LIMIT_MS - 200),
                cancelled,
            );
            let (actual_wall, output_bytes) = observation.as_ref().map_or_else(
                |_| (elapsed_ms(started), 0),
                |receipt| (receipt.wall_ms, receipt.output_bytes),
            );
            let actual = ResourceVec {
                wall_ms: actual_wall,
                attempts: 1,
                output_bytes,
                ..ResourceVec::default()
            };
            let trusted = observation
                .as_ref()
                .ok()
                .and_then(|receipt| evaluate(&task, &receipt.output).ok());
            let replica_matches = index != 1
                || observation
                    .as_ref()
                    .ok()
                    .zip(accepted[0].as_ref())
                    .is_some_and(|(receipt, (_, original))| &receipt.output == original);
            let trusted_failure = observation.is_ok() && (trusted.is_none() || !replica_matches);
            let observed = observation.as_ref().ok();
            let reason = observation
                .as_ref()
                .err()
                .map(ToString::to_string)
                .or_else(|| {
                    trusted_failure
                        .then(|| "output disagrees with the frozen reference or replay".to_owned())
                })
                .or_else(|| {
                    (actual_wall > ACTION_LIMIT_MS)
                        .then(|| "worker settlement exceeded wall-time reservation".to_owned())
                });
            let outcome = if cancelled.load(Ordering::Acquire) {
                AttemptOutcome::Cancelled
            } else if reason.is_some() {
                AttemptOutcome::Failed
            } else {
                AttemptOutcome::Succeeded
            };
            let reason = reason.or_else(|| {
                (outcome == AttemptOutcome::Cancelled).then(|| "cancelled by supervisor".into())
            });
            let metric_id = format!("{attempt}-correct");
            let metrics = trusted
                .filter(|_| outcome == AttemptOutcome::Succeeded)
                .map_or_else(Vec::new, |(correct, total)| {
                    vec![MetricInput {
                        id: metric_id.clone(),
                        name: "exact_reference_match".into(),
                        unit: "fraction".into(),
                        evaluator_hash: campaign.spec.evaluator_hash.clone(),
                        split_hash: input_hash.clone(),
                        value: correct as f64 / total as f64,
                    }]
                });
            let artifacts = observed.map_or_else(Vec::new, |receipt| {
                vec![ArtifactInput {
                    name: "worker-receipt.json".into(),
                    media_type: "application/json".into(),
                    bytes: receipt.artifact.clone(),
                }]
            });
            storage(store.complete_attempt(&AttemptCompletion {
                id: attempt.clone(), lease_token, outcome,
                manifest: serde_json::to_vec(&json!({
                    "version": "aether.lab-manifest/1", "attempt": attempt, "campaign": id, "task": task,
                    "protocol_hash": campaign.spec.protocol_hash, "input_hash": input_hash,
                    "evaluator_hash": campaign.spec.evaluator_hash, "usage": actual,
                    "worker": observed, "trusted_counts": trusted, "replica_matches": replica_matches, "reason": reason,
                }))?,
                usage: units(actual), metrics, artifacts, reason,
            }))?;
            budget
                .settle(&attempt, actual)
                .map_err(|error| Error::Schedule(error.to_string()))?;
            if outcome != AttemptOutcome::Succeeded {
                failed = trusted_failure;
                if failed {
                    if let Some(receipt) = observed {
                        let evidence_id = format!("{attempt}-contrary");
                        storage(store.link_evidence(&NewEvidence {
                            id: evidence_id.clone(),
                            claim_id: main_claim.clone(),
                            target: EvidenceTarget::Artifact {
                                attempt_id: attempt.clone(),
                                sha256: receipt.artifact_sha256.clone(),
                            },
                            relation: EvidenceRelation::Contradicts,
                        }))?;
                        storage(store.record_contradiction(&NewContradiction {
                            id: format!("{attempt}-contradiction"), claim_id: main_claim.clone(),
                            evidence_id, reason: "The returned output failed the frozen reference or replay check".into(), invalidates: false,
                        }))?;
                    }
                    continue;
                }
                if outcome == AttemptOutcome::Cancelled {
                    continue;
                }
                return Err(Error::Schedule(
                    "worker failed before usable evidence was produced".into(),
                ));
            }
            let receipt = observation.map_err(|error| Error::Integrity(error.to_string()))?;
            storage(store.link_evidence(&NewEvidence {
                id: format!("{attempt}-metric-edge"),
                claim_id: main_claim.clone(),
                target: EvidenceTarget::Metric { metric_id },
                relation: EvidenceRelation::Supports,
            }))?;
            storage(store.link_evidence(&NewEvidence {
                id: format!("{attempt}-artifact-edge"),
                claim_id: main_claim.clone(),
                target: EvidenceTarget::Artifact {
                    attempt_id: attempt.clone(),
                    sha256: receipt.artifact_sha256,
                },
                relation: if index == 1 {
                    EvidenceRelation::Reproduces
                } else {
                    EvidenceRelation::Supports
                },
            }))?;
            if index == 1 {
                let (original, _) = accepted[0]
                    .as_ref()
                    .ok_or_else(|| Error::Integrity("missing original attempt".into()))?;
                storage(store.record_replication(&NewReplication {
                    id: format!("{id}-replication"),
                    original_attempt_id: original.clone(),
                    replay_attempt_id: attempt.clone(),
                    passed: true,
                    reason:
                        "Fresh process output matched the original and trusted reference".into(),
                }))?;
            }
            accepted[index] = Some((attempt, receipt.output));
        }
    }
}

fn units(resource: ResourceVec) -> BudgetUnits {
    BudgetUnits {
        wall_ms: resource.wall_ms,
        attempts: resource.attempts,
        tokens: resource.model_tokens,
        micro_usd: resource.paid_microusd,
    }
}

fn candidate(
    id: &str,
    kind: ActionKind,
    eligible: bool,
    estimated_cost: ResourceVec,
    estimates: [u16; 3],
) -> Action {
    Action {
        id: id.into(),
        kind,
        eligible,
        estimated_cost,
        expected_gain: estimates[0],
        uncertainty_reduction: estimates[1],
        claim_importance: estimates[2],
    }
}

fn now_ms() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|time| u64::try_from(time.as_millis()).ok())
        .ok_or_else(|| Error::Integrity("clock cannot produce a lease timestamp".into()))
}

fn elapsed_ms(started: Instant) -> u64 {
    u64::try_from(started.elapsed().as_nanos().div_ceil(1_000_000)).unwrap_or(u64::MAX)
}

pub fn read_json_file<T: serde::de::DeserializeOwned>(path: &Path) -> Result<T> {
    let mut bytes = Vec::new();
    File::open(path)?.take(65_537).read_to_end(&mut bytes)?;
    if bytes.len() > 65_536 {
        return Err(Error::Input("JSON config exceeds 64 KiB".into()));
    }
    Ok(serde_json::from_slice(&bytes)?)
}
