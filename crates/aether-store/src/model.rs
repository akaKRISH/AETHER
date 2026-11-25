use rusqlite::types::{FromSql, FromSqlError, FromSqlResult, ToSql, ToSqlOutput, ValueRef};
use serde::{Deserialize, Serialize};

macro_rules! names {
    ($name:ident { $($variant:ident => $value:literal),+ $(,)? }) => {
        #[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
        #[serde(rename_all = "snake_case")]
        pub enum $name { $($variant),+ }

        impl ToSql for $name {
            fn to_sql(&self) -> rusqlite::Result<ToSqlOutput<'_>> {
                let name = match self { $(Self::$variant => $value),+ };
                Ok(ToSqlOutput::Borrowed(ValueRef::Text(name.as_bytes())))
            }
        }

        impl FromSql for $name {
            fn column_result(value: ValueRef<'_>) -> FromSqlResult<Self> {
                match value.as_str()? {
                    $($value => Ok(Self::$variant),)+
                    _ => Err(FromSqlError::InvalidType),
                }
            }
        }
    };
}

names!(CampaignState {
    Draft => "draft", Approved => "approved", Running => "running", Paused => "paused",
    Completed => "completed", Failed => "failed", Cancelled => "cancelled",
});
names!(Action { Explore => "explore", Replay => "replay", Challenge => "challenge" });
names!(AttemptOutcome {
    Succeeded => "succeeded", Failed => "failed", Cancelled => "cancelled", Expired => "expired",
});
names!(ClaimState {
    Proposed => "proposed", Supported => "supported", Verified => "verified",
    Rejected => "rejected", Inconclusive => "inconclusive", Revoked => "revoked",
});
names!(EvidenceRelation {
    Supports => "supports", Contradicts => "contradicts", DerivedFrom => "derived_from",
    Reproduces => "reproduces", Invalidates => "invalidates",
});
names!(VerificationVerdict {
    Verified => "verified", Rejected => "rejected", Inconclusive => "inconclusive",
});

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub struct BudgetUnits {
    pub wall_ms: u64,
    pub attempts: u64,
    pub tokens: u64,
    pub micro_usd: u64,
}

impl BudgetUnits {
    pub(crate) fn checked_add(self, other: Self) -> Option<Self> {
        Some(Self {
            wall_ms: self.wall_ms.checked_add(other.wall_ms)?,
            attempts: self.attempts.checked_add(other.attempts)?,
            tokens: self.tokens.checked_add(other.tokens)?,
            micro_usd: self.micro_usd.checked_add(other.micro_usd)?,
        })
    }

    pub(crate) fn checked_sub(self, other: Self) -> Option<Self> {
        Some(Self {
            wall_ms: self.wall_ms.checked_sub(other.wall_ms)?,
            attempts: self.attempts.checked_sub(other.attempts)?,
            tokens: self.tokens.checked_sub(other.tokens)?,
            micro_usd: self.micro_usd.checked_sub(other.micro_usd)?,
        })
    }

    pub(crate) fn fits(self, cap: Self) -> bool {
        cap.checked_sub(self).is_some()
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Budget {
    pub caps: BudgetUnits,
    pub used: BudgetUnits,
    pub reserved: BudgetUnits,
    pub halted: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewCampaign {
    pub id: String,
    pub title: String,
    pub objective: String,
    pub protocol_hash: String,
    pub policy_hash: String,
    pub evaluator_hash: String,
    pub requires_replay: bool,
    pub caps: BudgetUnits,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Campaign {
    #[serde(flatten)]
    pub spec: NewCampaign,
    pub state: CampaignState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewHypothesis {
    pub id: String,
    pub campaign_id: String,
    pub statement: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewExperiment {
    pub id: String,
    pub campaign_id: String,
    pub hypothesis_id: Option<String>,
    pub task_key: String,
    pub definition_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewRun {
    pub id: String,
    pub experiment_id: String,
    pub parent_run_id: Option<String>,
    pub action: Action,
    pub seed: u64,
    pub input_hash: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewAttempt {
    pub id: String,
    pub run_id: String,
    pub lease_token: String,
    pub lease_expires_at_ms: u64,
    pub estimate: BudgetUnits,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Attempt {
    pub id: String,
    pub run_id: String,
    pub lease_expires_at_ms: u64,
    pub estimate: BudgetUnits,
    pub outcome: Option<AttemptOutcome>,
    pub usage: Option<BudgetUnits>,
    pub manifest_sha256: Option<String>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MetricInput {
    pub id: String,
    pub name: String,
    pub unit: String,
    pub evaluator_hash: String,
    pub split_hash: String,
    pub value: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Metric {
    pub attempt_id: String,
    #[serde(flatten)]
    pub data: MetricInput,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct ArtifactInput {
    pub name: String,
    pub media_type: String,
    pub bytes: Vec<u8>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Artifact {
    pub attempt_id: String,
    pub name: String,
    pub media_type: String,
    pub sha256: String,
    pub size_bytes: u64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AttemptCompletion {
    pub id: String,
    pub lease_token: String,
    pub outcome: AttemptOutcome,
    pub manifest: Vec<u8>,
    pub usage: BudgetUnits,
    pub metrics: Vec<MetricInput>,
    pub artifacts: Vec<ArtifactInput>,
    pub reason: Option<String>,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewClaim {
    pub id: String,
    pub campaign_id: String,
    pub statement: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Claim {
    #[serde(flatten)]
    pub spec: NewClaim,
    pub state: ClaimState,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "snake_case")]
pub enum EvidenceTarget {
    Metric { metric_id: String },
    Artifact { attempt_id: String, sha256: String },
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewEvidence {
    pub id: String,
    pub claim_id: String,
    pub target: EvidenceTarget,
    pub relation: EvidenceRelation,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Evidence {
    #[serde(flatten)]
    pub data: NewEvidence,
    pub invalidated: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Dependency {
    pub claim_id: String,
    pub prerequisite_id: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewReplication {
    pub id: String,
    pub original_attempt_id: String,
    pub replay_attempt_id: String,
    pub passed: bool,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewVerification {
    pub id: String,
    pub claim_id: String,
    pub verdict: VerificationVerdict,
    pub rule_hash: String,
    pub evaluator_hash: String,
    pub replay_attempt_id: Option<String>,
    pub reason: String,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct NewContradiction {
    pub id: String,
    pub claim_id: String,
    pub evidence_id: String,
    pub reason: String,
    pub invalidates: bool,
}

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct Event {
    pub campaign_id: String,
    pub sequence: u64,
    pub kind: String,
    pub occurred_at_ms: u64,
    pub detail: serde_json::Value,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct Snapshot {
    pub campaign: Campaign,
    pub budget: Budget,
    pub hypotheses: Vec<NewHypothesis>,
    pub experiments: Vec<NewExperiment>,
    pub runs: Vec<NewRun>,
    pub attempts: Vec<Attempt>,
    pub metrics: Vec<Metric>,
    pub artifacts: Vec<Artifact>,
    pub claims: Vec<Claim>,
    pub evidence: Vec<Evidence>,
    pub dependencies: Vec<Dependency>,
    pub verifications: Vec<NewVerification>,
    pub contradictions: Vec<NewContradiction>,
    pub replications: Vec<NewReplication>,
}
