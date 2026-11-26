use std::fs::File;
use std::io::Read;
use std::path::Path;

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize, de::DeserializeOwned};

use super::discovery::{PAGE_LIMIT, fetch_page};
use super::{
    Citation, CitationCatalog, CitationSource, LiteratureError, MetadataPage, MetadataProvider,
    arxiv_root, observed_now, paper_id, valid_license_url, validate_query,
};
use aether_store::artifact_sha256 as sha256;

const PAGE_SIZE: usize = 12;
const CACHE_SECONDS: u64 = 60 * 60;
const MAX_TEXT_BYTES: u64 = 512 * 1024;

pub(super) fn initialize(db: &Connection) -> Result<(), LiteratureError> {
    db.execute_batch(
        "CREATE TABLE IF NOT EXISTS literature_snapshots (
            work TEXT NOT NULL REFERENCES citations(doi), digest TEXT NOT NULL,
            record TEXT NOT NULL, PRIMARY KEY(work, digest)
        ) STRICT;
        CREATE TABLE IF NOT EXISTS literature_pages (
            provider TEXT NOT NULL, query TEXT NOT NULL, page INTEGER NOT NULL,
            observed INTEGER NOT NULL, record TEXT NOT NULL, PRIMARY KEY(provider, query, page)
        ) STRICT;
        CREATE TABLE IF NOT EXISTS literature_cooldown (
            provider TEXT PRIMARY KEY NOT NULL, ready_at INTEGER NOT NULL
        ) STRICT;
        CREATE TABLE IF NOT EXISTS literature_texts (
            work TEXT NOT NULL REFERENCES citations(doi), digest TEXT NOT NULL, body BLOB NOT NULL,
            search_text TEXT NOT NULL, record TEXT NOT NULL, PRIMARY KEY(work, digest)
        ) STRICT;",
    )?;
    Ok(())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataSnapshot {
    sha256: String,
    pub(super) citation: Citation,
    acquisition: Option<Acquisition>,
}

#[derive(Debug, Serialize, Deserialize)]
struct Acquisition {
    endpoint: String,
    query: String,
    page: u16,
    page_size: usize,
    response_sha256: String,
}

pub(super) fn save_snapshot(
    db: &Connection,
    work: &str,
    citation: &Citation,
    page: Option<&MetadataPage>,
) -> Result<(), LiteratureError> {
    let mut identity = citation.clone();
    identity.provenance.observed_at_unix_seconds = 0;
    let digest = sha256(&serde_json::to_vec(&identity)?);
    let snapshot = MetadataSnapshot {
        sha256: digest.clone(),
        citation: citation.clone(),
        acquisition: page.map(|page| Acquisition {
            endpoint: page.endpoint.clone(),
            query: page.query.clone(),
            page: page.page,
            page_size: page.page_size,
            response_sha256: page.response_sha256.clone(),
        }),
    };
    let inserted = db.execute(
        "INSERT OR IGNORE INTO literature_snapshots VALUES (?1, ?2, ?3)",
        params![work, digest, serde_json::to_string(&snapshot)?],
    )?;
    if inserted > 0 && count_for(db, "literature_snapshots", work)? > 24 {
        return Err(LiteratureError::ArchiveLimit);
    }
    Ok(())
}

fn count_for(db: &Connection, table: &str, work: &str) -> Result<u32, LiteratureError> {
    Ok(db.query_row(
        &format!("SELECT count(*) FROM {table} WHERE work = ?1"),
        [work],
        |row| row.get(0),
    )?)
}

fn now() -> u64 {
    observed_now(CitationSource::Manual).observed_at_unix_seconds
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TextManifest {
    paper_id: String,
    source_url: String,
    license: TextLicense,
    rights_confirmed: bool,
}

#[derive(Debug, Serialize, Deserialize)]
enum TextLicense {
    #[serde(rename = "CC-BY-4.0")]
    CcBy,
    #[serde(rename = "CC0-1.0")]
    CcZero,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
enum PassageKind {
    ResearchQuestion,
    Method,
    Claim,
    Result,
    Limitation,
}

impl PassageKind {
    const ALL: [Self; 5] = [
        Self::ResearchQuestion,
        Self::Method,
        Self::Claim,
        Self::Result,
        Self::Limitation,
    ];

    fn heading(line: &str) -> Option<Self> {
        match line
            .trim_matches('#')
            .trim()
            .trim_end_matches(':')
            .to_ascii_lowercase()
            .as_str()
        {
            "research question" | "research questions" | "objective" | "objectives" => {
                Some(Self::ResearchQuestion)
            }
            "method" | "methods" | "methodology" => Some(Self::Method),
            "claim" | "claims" => Some(Self::Claim),
            "result" | "results" | "findings" => Some(Self::Result),
            "limitation" | "limitations" => Some(Self::Limitation),
            _ => None,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct Passage {
    kind: PassageKind,
    start_byte: usize,
    end_byte: usize,
    quote: String,
}

#[derive(Debug, Serialize)]
struct TextRecord {
    id: String,
    source: TextManifest,
    source_sha256: String,
    source_bytes: usize,
    observed_at_unix_seconds: u64,
    extraction: &'static str,
    status: &'static str,
    missing_sections: Vec<PassageKind>,
    extraction_truncated: bool,
    passages: Vec<Passage>,
}

#[derive(Debug, Serialize)]
pub struct ArchiveWork {
    id: String,
    citation: Citation,
    snapshots: Vec<MetadataSnapshot>,
    text_status: &'static str,
    documents: Vec<serde_json::Value>,
}

fn stored_records<T: DeserializeOwned>(
    db: &Connection,
    statement: &str,
    work: &str,
) -> Result<Vec<T>, LiteratureError> {
    db.prepare(statement)?
        .query_map([work], |row| row.get::<_, String>(0))?
        .map(|row| Ok(serde_json::from_str(&row?)?))
        .collect()
}

impl CitationCatalog {
    pub fn discover(
        &mut self,
        provider: MetadataProvider,
        query: &str,
        page: u16,
        save: bool,
    ) -> Result<MetadataPage, LiteratureError> {
        self.discover_with(provider, query, page, save, fetch_page)
    }

    fn discover_with(
        &mut self,
        provider: MetadataProvider,
        query: &str,
        page: u16,
        save: bool,
        fetch: impl FnOnce(MetadataProvider, &str, usize, u16) -> Result<MetadataPage, LiteratureError>,
    ) -> Result<MetadataPage, LiteratureError> {
        validate_query(query, PAGE_SIZE, false)?;
        if page >= PAGE_LIMIT {
            return Err(LiteratureError::InvalidQuery);
        }
        let query = query.trim();
        let (name, _, cooldown) = provider.settings();
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let cached: Option<String> = tx.query_row(
            "SELECT record FROM literature_pages WHERE provider = ?1 AND query = ?2 AND page = ?3 AND observed >= ?4",
            params![name, query, page, now().saturating_sub(CACHE_SECONDS) as i64], |row| row.get(0),
        ).optional()?;
        let result = if let Some(cached) = cached {
            tx.commit()?;
            let mut cached: MetadataPage = serde_json::from_str(&cached)?;
            cached.cached = true;
            cached
        } else {
            let ready_at: Option<i64> = tx
                .query_row(
                    "SELECT ready_at FROM literature_cooldown WHERE provider = ?1",
                    [name],
                    |row| row.get(0),
                )
                .optional()?;
            let timestamp = now();
            let delay = u64::try_from(ready_at.unwrap_or(0))
                .unwrap_or(0)
                .saturating_sub(timestamp);
            if delay != 0 {
                return Err(LiteratureError::RateLimited {
                    retry_after_secs: delay,
                });
            }
            // Cover the eight-second request even if the process exits before settling.
            set_cooldown(&tx, name, timestamp + 10 + cooldown)?;
            tx.commit()?;
            let fetched = fetch(provider, query, PAGE_SIZE, page);
            let delay = match &fetched {
                Err(LiteratureError::RateLimited { retry_after_secs }) => *retry_after_secs,
                _ => cooldown,
            };
            let tx = self
                .db
                .transaction_with_behavior(TransactionBehavior::Immediate)?;
            set_cooldown(&tx, name, now().saturating_add(delay).saturating_add(1))?;
            if let Ok(result) = &fetched {
                tx.execute(
                    "INSERT INTO literature_pages VALUES (?1, ?2, ?3, ?4, ?5)
                     ON CONFLICT(provider, query, page) DO UPDATE SET observed = excluded.observed, record = excluded.record",
                    params![
                        name,
                        query,
                        page,
                        result.observed_at_unix_seconds.min(i64::MAX as u64) as i64,
                        serde_json::to_string(result)?
                    ],
                )?;
                tx.execute(
                    "DELETE FROM literature_pages WHERE rowid IN (
                        SELECT rowid FROM literature_pages ORDER BY observed DESC, rowid DESC LIMIT -1 OFFSET 256
                    )", [],
                )?;
            }
            tx.commit()?;
            fetched?
        };
        if save {
            self.import_observations(result.records.clone(), Some(&result))?;
        }
        Ok(result)
    }

    pub(super) fn resolve_work(&self, id: &str) -> Result<(String, String), LiteratureError> {
        let id = paper_id(id).ok_or(LiteratureError::InvalidCitation)?;
        let alias = arxiv_alias(&id);
        let work = self
            .db
            .query_row(
                "SELECT doi FROM citations WHERE doi = COALESCE(
                (SELECT work FROM citation_aliases WHERE alias = ?1), ?1)",
                [alias],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(LiteratureError::MissingCitation)?;
        Ok((id, work))
    }

    pub fn archive_work(&self, id: &str) -> Result<ArchiveWork, LiteratureError> {
        let (id, work) = self.resolve_work(id)?;
        if id != arxiv_alias(&id) {
            self.freeze(&id)?;
        }
        let record: String = self.db.query_row(
            "SELECT record FROM citations WHERE doi = ?1",
            [&work],
            |row| row.get(0),
        )?;
        let snapshots = self.snapshots(&work)?;
        let documents = stored_records(
            &self.db,
            "SELECT record FROM literature_texts WHERE work = ?1 ORDER BY digest",
            &work,
        )?;
        Ok(ArchiveWork {
            id: work,
            citation: serde_json::from_str(&record)?,
            snapshots,
            text_status: if documents.is_empty() {
                "unavailable"
            } else {
                "available"
            },
            documents,
        })
    }

    pub(super) fn snapshots(&self, work: &str) -> Result<Vec<MetadataSnapshot>, LiteratureError> {
        stored_records(
            &self.db,
            "SELECT record FROM literature_snapshots WHERE work = ?1 ORDER BY rowid",
            work,
        )
    }

    pub fn ingest_text(
        &mut self,
        mut source: TextManifest,
        path: &Path,
    ) -> Result<serde_json::Value, LiteratureError> {
        let (id, work) = self.resolve_work(&source.paper_id)?;
        if !source.rights_confirmed
            || !source.source_url.starts_with("https://")
            || !valid_license_url(&source.source_url)
            || (id.starts_with("arxiv:") && id == arxiv_alias(&id))
        {
            return Err(LiteratureError::InvalidText);
        }
        self.freeze(&id)?;
        source.paper_id = id;
        let mut raw = Vec::new();
        File::open(path)
            .and_then(|file| file.take(MAX_TEXT_BYTES + 1).read_to_end(&mut raw))
            .map_err(|_| LiteratureError::TextIo)?;
        let body = std::str::from_utf8(&raw).map_err(|_| LiteratureError::InvalidText)?;
        if raw.len() as u64 > MAX_TEXT_BYTES
            || body.trim().is_empty()
            || body
                .chars()
                .any(|c| c.is_control() && !matches!(c, '\t' | '\n' | '\r'))
        {
            return Err(LiteratureError::InvalidText);
        }
        let source_sha256 = sha256(&raw);
        let id = sha256(&serde_json::to_vec(&(&source, &source_sha256))?);
        let (passages, extraction_truncated) = extract(body);
        let missing_sections = PassageKind::ALL
            .into_iter()
            .filter(|kind| !passages.iter().any(|passage| passage.kind == *kind))
            .collect();
        let search_text = passages
            .iter()
            .map(|p| p.quote.to_lowercase())
            .collect::<Vec<_>>()
            .join("\n");
        let record = TextRecord {
            id,
            source,
            source_sha256,
            source_bytes: raw.len(),
            observed_at_unix_seconds: now(),
            extraction: "explicit_sections_v1",
            status: "extracted_unverified",
            missing_sections,
            extraction_truncated,
            passages,
        };
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let inserted = tx.execute(
            "INSERT OR IGNORE INTO literature_texts VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                work,
                record.id,
                raw,
                search_text,
                serde_json::to_string(&record)?
            ],
        )?;
        if inserted > 0
            && (count_for(&tx, "literature_texts", &work)? > 8
                || tx.query_row("SELECT count(*) FROM literature_texts", [], |row| {
                    row.get::<_, u32>(0)
                })? > 512)
        {
            return Err(LiteratureError::ArchiveLimit);
        }
        let stored: String = tx.query_row(
            "SELECT record FROM literature_texts WHERE work = ?1 AND digest = ?2",
            params![work, record.id],
            |row| row.get(0),
        )?;
        tx.commit()?;
        Ok(serde_json::from_str(&stored)?)
    }

    pub fn source_text(
        &self,
        id: &str,
        digest: &str,
    ) -> Result<serde_json::Value, LiteratureError> {
        let (_, work) = self.resolve_work(id)?;
        let (raw, record): (Vec<u8>, String) = self
            .db
            .query_row(
                "SELECT body, record FROM literature_texts WHERE work = ?1 AND digest = ?2",
                params![work, digest],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .optional()?
            .ok_or(LiteratureError::MissingCitation)?;
        let record: serde_json::Value = serde_json::from_str(&record)?;
        if record["source_sha256"].as_str() != Some(&sha256(&raw)) {
            return Err(LiteratureError::InvalidText);
        }
        let source = std::str::from_utf8(&raw).map_err(|_| LiteratureError::InvalidText)?;
        Ok(serde_json::json!({ "record": record, "text": source }))
    }
}

pub(super) fn arxiv_alias(id: &str) -> String {
    id.strip_prefix("arxiv:")
        .map_or_else(|| id.into(), |value| format!("arxiv:{}", arxiv_root(value)))
}

fn set_cooldown(db: &Connection, name: &str, ready_at: u64) -> Result<(), LiteratureError> {
    db.execute(
        "INSERT INTO literature_cooldown VALUES (?1, ?2)
         ON CONFLICT(provider) DO UPDATE SET ready_at = excluded.ready_at",
        params![name, ready_at.min(i64::MAX as u64) as i64],
    )?;
    Ok(())
}

fn extract(body: &str) -> (Vec<Passage>, bool) {
    let mut offset = 0;
    let mut kind = None;
    let mut start = None;
    let mut passages = Vec::new();
    let mut truncated = false;
    for line in body.split_inclusive('\n').chain([""]) {
        let trimmed = line.trim();
        let heading = PassageKind::heading(trimmed);
        let boundary = heading.is_some() || trimmed.starts_with('#');
        if boundary || trimmed.is_empty() {
            if let Some((kind, begin)) = start.take() {
                let quote = body[begin..offset].trim_end();
                if passages.len() == 64 || quote.len() > 4096 {
                    truncated = true;
                } else {
                    passages.push(Passage {
                        kind,
                        start_byte: begin,
                        end_byte: begin + quote.len(),
                        quote: quote.into(),
                    });
                }
            }
            if boundary {
                kind = heading;
            }
        } else if let Some(kind) = kind {
            start.get_or_insert((kind, offset + line.len() - line.trim_start().len()));
        }
        offset += line.len();
    }
    (passages, truncated)
}

#[cfg(test)]
mod tests {
    use super::super::discovery::decoded_page;
    use super::*;

    #[test]
    fn page_cache_and_throttle_survive_reopen_and_allow_resume() {
        let path = std::env::temp_dir().join(format!(
            "aether-apo-cache-{}-{}.db",
            std::process::id(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        ));
        let provider = MetadataProvider::Crossref;
        let mut catalog = CitationCatalog::open(&path).unwrap();
        let body = serde_json::json!({"status":"ok","message":{"items": (0..12).map(|i|
            serde_json::json!({"DOI":format!("10.9999/apo-{i}"), "title":[format!("Fixture {i}")]})
        ).collect::<Vec<_>>()}})
        .to_string();
        let first = catalog
            .discover_with(provider, " reproducibility ", 0, false, |p, q, n, i| {
                decoded_page(p, &body, n, now(), q, i)
            })
            .unwrap();
        assert_eq!(first.next_page, Some(1));
        assert!(catalog.search("", 20).unwrap().is_empty());
        set_cooldown(&catalog.db, "crossref", now() + 60).unwrap();
        drop(catalog);
        let mut catalog = CitationCatalog::open(&path).unwrap();
        let cached = catalog
            .discover_with(provider, "reproducibility", 0, true, |_, _, _, _| {
                panic!("the cached request must not fetch again")
            })
            .unwrap();
        assert!(cached.cached);
        assert_eq!(cached.response_sha256, first.response_sha256);
        assert_eq!(catalog.search("", 20).unwrap().len(), 12);
        assert!(
            catalog.snapshots("10.9999/apo-0").unwrap()[0]
                .acquisition
                .is_some()
        );
        let next = first.next_page.unwrap();
        assert!(matches!(
            catalog.discover_with(
                provider,
                "reproducibility",
                next,
                true,
                |_, _, _, _| panic!("the persistent cooldown must stop the request")
            ),
            Err(LiteratureError::RateLimited { .. })
        ));
        set_cooldown(&catalog.db, "crossref", 0).unwrap();
        let end = catalog
            .discover_with(provider, "reproducibility", next, true, |p, q, n, i| {
                assert_eq!(i, 1);
                decoded_page(
                    p,
                    r#"{"status":"ok","message":{"items":[]}}"#,
                    n,
                    now(),
                    q,
                    i,
                )
            })
            .unwrap();
        assert_eq!(end.next_page, None);
        drop(catalog);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn extraction_preserves_utf8_crlf_and_reports_truncation() {
        let input = "## Results\r\n  café\r\ncontinued\r\n\r\n## References\r\nignored";
        let (passages, truncated) = extract(input);
        assert!(!truncated);
        assert_eq!(passages.len(), 1);
        let passage = &passages[0];
        assert_eq!(passage.quote, "café\r\ncontinued");
        assert_eq!(&input[passage.start_byte..passage.end_byte], passage.quote);
        assert!(extract(&format!("## Methods\n{}", "x".repeat(4097))).1);
    }
}
