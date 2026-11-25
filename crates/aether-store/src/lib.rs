#![forbid(unsafe_code)]

//! SQLite adapter for the trusted local research controller.
//! The database and its enclosing directory belong to the local operator.

mod evidence;
mod execution;
mod model;
mod read;
#[cfg(test)]
mod tests;

pub use model::*;

use rusqlite::{
    Connection, OptionalExtension, Row, Transaction, TransactionBehavior, params, types::Type,
};
use serde::Serialize;
use serde::de::DeserializeOwned;
use serde_json::{Value, json};
use sha2::{Digest, Sha256};
use std::error::Error as StdError;
use std::fmt;
use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

const SCHEMA_VERSION: u32 = 1;
pub const MAX_PAGE_SIZE: usize = 512;
const MAX_TEXT: usize = 4096;
const MAX_EVENT_BYTES: usize = 128 * 1024;

pub type Result<T> = std::result::Result<T, StoreError>;

#[derive(Debug)]
pub enum StoreError {
    Sqlite(rusqlite::Error),
    Json(serde_json::Error),
    Invalid(&'static str),
    NotFound(&'static str),
    Conflict(&'static str),
    Integrity(&'static str),
    BudgetExceeded,
    UnsupportedSchema(u32),
}

impl fmt::Display for StoreError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Sqlite(error) => write!(f, "SQLite operation failed: {error}"),
            Self::Json(error) => write!(f, "JSON operation failed: {error}"),
            Self::Invalid(reason) => write!(f, "invalid input: {reason}"),
            Self::NotFound(entity) => write!(f, "{entity} not found"),
            Self::Conflict(reason) => write!(f, "state conflict: {reason}"),
            Self::Integrity(reason) => write!(f, "integrity check failed: {reason}"),
            Self::BudgetExceeded => f.write_str("campaign budget does not admit this attempt"),
            Self::UnsupportedSchema(version) => write!(f, "unsupported schema version {version}"),
        }
    }
}

impl StdError for StoreError {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        match self {
            Self::Sqlite(error) => Some(error),
            Self::Json(error) => Some(error),
            _ => None,
        }
    }
}

impl From<rusqlite::Error> for StoreError {
    fn from(error: rusqlite::Error) -> Self {
        use rusqlite::ffi::ErrorCode;
        if error.sqlite_error_code() == Some(ErrorCode::ConstraintViolation) {
            return Self::Conflict("record conflicts with a database constraint");
        }
        Self::Sqlite(error)
    }
}

impl From<serde_json::Error> for StoreError {
    fn from(error: serde_json::Error) -> Self {
        Self::Json(error)
    }
}

pub struct Store {
    conn: Connection,
}

impl Store {
    pub fn open(path: impl AsRef<Path>) -> Result<Self> {
        Self::initialize(Connection::open(path)?)
    }

    pub fn open_in_memory() -> Result<Self> {
        Self::initialize(Connection::open_in_memory()?)
    }

    fn initialize(mut conn: Connection) -> Result<Self> {
        conn.busy_timeout(Duration::from_secs(3))?;
        conn.pragma_update(None, "foreign_keys", true)?;
        conn.pragma_update(None, "journal_mode", "DELETE")?;
        conn.pragma_update(None, "synchronous", "FULL")?;
        let tx = conn.transaction_with_behavior(TransactionBehavior::Immediate)?;
        let version: u32 = tx.pragma_query_value(None, "user_version", |row| row.get(0))?;
        match version {
            SCHEMA_VERSION => {}
            0 => {
                tx.execute_batch(include_str!("001_initial.sql"))?;
                tx.pragma_update(None, "user_version", SCHEMA_VERSION)?;
            }
            unsupported => return Err(StoreError::UnsupportedSchema(unsupported)),
        }
        tx.commit()?;
        Ok(Self { conn })
    }

    fn write(&mut self, work: impl FnOnce(&Transaction<'_>) -> Result<()>) -> Result<()> {
        let tx = self
            .conn
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        work(&tx)?;
        tx.commit()?;
        Ok(())
    }

    pub fn create_campaign(&mut self, campaign: &NewCampaign) -> Result<()> {
        validate_id(&campaign.id)?;
        validate_text(&campaign.title)?;
        validate_text(&campaign.objective)?;
        validate_hash(&campaign.protocol_hash)?;
        validate_hash(&campaign.policy_hash)?;
        validate_hash(&campaign.evaluator_hash)?;
        self.write(|tx| {
            tx.execute(
                "INSERT INTO campaign VALUES (?, ?, ?, ?, ?, ?, ?, 'draft')",
                params![
                    campaign.id,
                    campaign.title,
                    campaign.objective,
                    campaign.protocol_hash,
                    campaign.policy_hash,
                    campaign.evaluator_hash,
                    campaign.requires_replay,
                ],
            )?;
            tx.execute(
                "INSERT INTO budget (campaign_id, caps, used, reserved) VALUES (?, ?, ?, ?)",
                params![
                    campaign.id,
                    serde_json::to_string(&campaign.caps)?,
                    serde_json::to_string(&BudgetUnits::default())?,
                    serde_json::to_string(&BudgetUnits::default())?,
                ],
            )?;
            append(
                tx,
                &campaign.id,
                "campaign.created",
                json!({"protocol_hash": campaign.protocol_hash}),
            )
        })
    }

    pub fn approve_campaign(&mut self, campaign_id: &str, protocol_hash: &str) -> Result<()> {
        validate_id(campaign_id)?;
        validate_hash(protocol_hash)?;
        self.write(|tx| {
            let (state, frozen): (CampaignState, String) = tx
                .query_row(
                    "SELECT state, protocol_hash FROM campaign WHERE id = ?",
                    [campaign_id],
                    |row| Ok((row.get(0)?, row.get(1)?)),
                )
                .optional()?
                .ok_or(StoreError::NotFound("campaign"))?;
            if frozen != protocol_hash {
                return Err(StoreError::Conflict(
                    "approval must match the frozen protocol",
                ));
            }
            if state == CampaignState::Approved {
                return Ok(());
            }
            if state != CampaignState::Draft {
                return Err(StoreError::Conflict("only a draft can be approved"));
            }
            set_campaign_state(tx, campaign_id, CampaignState::Approved)?;
            append(
                tx,
                campaign_id,
                "protocol.approved",
                json!({"protocol_hash": protocol_hash}),
            )
        })
    }

    pub fn transition_campaign(&mut self, campaign_id: &str, state: CampaignState) -> Result<()> {
        use CampaignState::*;
        validate_id(campaign_id)?;
        self.write(|tx| {
            let from = campaign_state(tx, campaign_id)?;
            if !matches!(
                (from, state),
                (Draft | Approved, Cancelled)
                    | (Approved | Paused, Running)
                    | (Running, Paused | Completed | Failed | Cancelled)
                    | (Paused, Failed | Cancelled)
            ) {
                return Err(StoreError::Conflict("campaign transition is unavailable"));
            }
            if matches!(state, Completed | Failed | Cancelled) {
                let active: bool = tx.query_row(
                    "SELECT EXISTS(SELECT 1 FROM attempt a JOIN run r ON r.id = a.run_id
                     JOIN experiment e ON e.id = r.experiment_id
                     WHERE e.campaign_id = ? AND a.outcome IS NULL)",
                    [campaign_id],
                    |row| row.get(0),
                )?;
                if active {
                    return Err(StoreError::Conflict(
                        "settle active attempts before ending the campaign",
                    ));
                }
            }
            if state == Running && Self::read_budget(tx, campaign_id)?.halted {
                return Err(StoreError::BudgetExceeded);
            }
            set_campaign_state(tx, campaign_id, state)?;
            append(
                tx,
                campaign_id,
                "campaign.state_changed",
                json!({"from": from, "to": state}),
            )
        })
    }

    pub fn add_hypothesis(&mut self, hypothesis: &NewHypothesis) -> Result<()> {
        validate_id(&hypothesis.id)?;
        validate_id(&hypothesis.campaign_id)?;
        validate_text(&hypothesis.statement)?;
        self.write(|tx| {
            ensure_campaign_open(tx, &hypothesis.campaign_id)?;
            tx.execute(
                "INSERT INTO hypothesis VALUES (?, ?, ?)",
                params![hypothesis.id, hypothesis.campaign_id, hypothesis.statement],
            )?;
            append(
                tx,
                &hypothesis.campaign_id,
                "hypothesis.proposed",
                json!({"id": hypothesis.id}),
            )
        })
    }

    pub fn add_experiment(&mut self, experiment: &NewExperiment) -> Result<()> {
        validate_id(&experiment.id)?;
        validate_id(&experiment.campaign_id)?;
        validate_id(&experiment.task_key)?;
        validate_hash(&experiment.definition_hash)?;
        self.write(|tx| {
            ensure_campaign_open(tx, &experiment.campaign_id)?;
            if let Some(hypothesis) = experiment.hypothesis_id.as_deref() {
                validate_id(hypothesis)?;
                let campaign: String = tx
                    .query_row(
                        "SELECT campaign_id FROM hypothesis WHERE id = ?",
                        [hypothesis],
                        |row| row.get(0),
                    )
                    .optional()?
                    .ok_or(StoreError::NotFound("hypothesis"))?;
                require_campaign(&experiment.campaign_id, &campaign)?;
            }
            tx.execute(
                "INSERT INTO experiment VALUES (?, ?, ?, ?, ?)",
                params![
                    experiment.id,
                    experiment.campaign_id,
                    experiment.hypothesis_id,
                    experiment.task_key,
                    experiment.definition_hash,
                ],
            )?;
            append(
                tx,
                &experiment.campaign_id,
                "experiment.created",
                json!({"id": experiment.id}),
            )
        })
    }

    pub fn record_decision(&mut self, campaign_id: &str, detail: Value) -> Result<()> {
        validate_id(campaign_id)?;
        if !detail.is_object() {
            return Err(StoreError::Invalid("decision detail must be an object"));
        }
        self.write(|tx| {
            campaign_state(tx, campaign_id)?;
            append(tx, campaign_id, "scheduler.decision", detail)
        })
    }
}

fn append(tx: &Transaction<'_>, campaign_id: &str, kind: &str, detail: Value) -> Result<()> {
    let detail = serde_json::to_string(&detail)?;
    if detail.len() > MAX_EVENT_BYTES {
        return Err(StoreError::Invalid("event detail exceeds its byte limit"));
    }
    tx.execute(
        "INSERT INTO event (campaign_id, sequence, kind, occurred_at_ms, detail)
         VALUES (?1, (SELECT COALESCE(MAX(sequence), 0) + 1 FROM event WHERE campaign_id = ?1), ?2, ?3, ?4)",
        params![campaign_id, kind, sqlite_integer(now_ms()?)?, detail],
    )?;
    Ok(())
}

pub fn artifact_sha256(content: &[u8]) -> String {
    format!("{:x}", Sha256::digest(content))
}

fn validate_id(value: &str) -> Result<()> {
    if value.is_empty()
        || value.len() > 128
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || matches!(byte, b'-' | b'_' | b'.'))
    {
        return Err(StoreError::Invalid("identifier"));
    }
    Ok(())
}

fn validate_text(value: &str) -> Result<()> {
    if value.trim().is_empty() || value.len() > MAX_TEXT || value.contains('\0') {
        return Err(StoreError::Invalid(
            "text is empty, too long or contains NUL",
        ));
    }
    Ok(())
}

fn validate_hash(value: &str) -> Result<()> {
    if value.len() != 64
        || !value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
    {
        return Err(StoreError::Invalid("expected lowercase SHA-256 hex"));
    }
    Ok(())
}

fn now_ms() -> Result<u64> {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()
        .and_then(|time| u64::try_from(time.as_millis()).ok())
        .ok_or(StoreError::Integrity(
            "clock is outside the supported UTC range",
        ))
}

fn sqlite_integer(value: u64) -> Result<i64> {
    i64::try_from(value).map_err(|_| StoreError::Invalid("quantity exceeds SQLite integer range"))
}

fn row_u64(row: &Row<'_>, index: usize) -> rusqlite::Result<u64> {
    let value: i64 = row.get(index)?;
    u64::try_from(value).map_err(|_| rusqlite::Error::IntegralValueOutOfRange(index, value))
}

fn object_json<T: Serialize>(value: T) -> Result<String> {
    Ok(serde_json::to_string(&value)?)
}

fn row_json<T: DeserializeOwned>(row: &Row<'_>, index: usize) -> rusqlite::Result<T> {
    let content: String = row.get(index)?;
    serde_json::from_str(&content).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(index, Type::Text, Box::new(error))
    })
}

fn campaign_state(conn: &Connection, campaign_id: &str) -> Result<CampaignState> {
    conn.query_row(
        "SELECT state FROM campaign WHERE id = ?",
        [campaign_id],
        |row| row.get(0),
    )
    .optional()?
    .ok_or(StoreError::NotFound("campaign"))
}

fn set_campaign_state(conn: &Connection, campaign_id: &str, state: CampaignState) -> Result<()> {
    conn.execute(
        "UPDATE campaign SET state = ? WHERE id = ?",
        params![state, campaign_id],
    )?;
    Ok(())
}

fn ensure_campaign_open(conn: &Connection, campaign_id: &str) -> Result<()> {
    if matches!(
        campaign_state(conn, campaign_id)?,
        CampaignState::Completed | CampaignState::Failed | CampaignState::Cancelled
    ) {
        return Err(StoreError::Conflict("campaign is closed"));
    }
    Ok(())
}

fn require_campaign(expected: &str, actual: &str) -> Result<()> {
    if expected != actual {
        return Err(StoreError::Conflict("cross-campaign reference"));
    }
    Ok(())
}
