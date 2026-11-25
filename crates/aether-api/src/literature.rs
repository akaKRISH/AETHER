use std::path::Path;
use std::sync::Mutex;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_RESULTS: usize = 20;
const MAX_IMPORT: usize = 100;
const MAX_CITATIONS: u32 = 5_000;
const CROSSREF_WORKS: &str = "https://api.crossref.org/works";
static CROSSREF_SLOT: Mutex<Option<Instant>> = Mutex::new(None);

#[derive(Debug, Error)]
pub enum LiteratureError {
    #[error("use a bounded citation query of up to 256 bytes and a limit of 1–20")]
    InvalidQuery,
    #[error("invalid or oversized citation metadata")]
    InvalidCitation,
    #[error("the citation catalog has reached its 5,000-record limit")]
    CatalogLimit,
    #[error("Crossref lookup is busy or rate-limited; retry in {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },
    #[error("Crossref is unavailable")]
    Upstream,
    #[error("Crossref returned invalid or oversized metadata")]
    InvalidResponse,
    #[error("citation storage: {0}")]
    Storage(#[from] rusqlite::Error),
    #[error("citation JSON: {0}")]
    Json(#[from] serde_json::Error),
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CitationSource {
    #[default]
    Manual,
    Crossref,
}

#[derive(Debug, Default, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CitationProvenance {
    pub source: CitationSource,
    pub observed_at_unix_seconds: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Citation {
    pub doi: String,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub container_title: Option<String>,
    #[serde(default)]
    pub published_year: Option<u16>,
    /// Source-reported rights metadata. Absence leaves publication rights unknown.
    #[serde(default)]
    pub license_urls: Vec<String>,
    #[serde(default)]
    pub provenance: CitationProvenance,
}

impl Citation {
    fn normalized(mut self) -> Result<Self, LiteratureError> {
        let Some((prefix, suffix)) = self.doi.trim().split_once('/') else {
            return Err(LiteratureError::InvalidCitation);
        };
        let registration = prefix.strip_prefix("10.").unwrap_or_default();
        if self.doi.len() > 256
            || !(4..=9).contains(&registration.len())
            || !registration.bytes().all(|byte| byte.is_ascii_digit())
            || suffix.is_empty()
            || !suffix
                .bytes()
                .all(|byte| byte.is_ascii_alphanumeric() || b"-._;()/:+".contains(&byte))
            || !valid_text(&self.title, 1024)
            || self.authors.len() > 32
            || self.authors.iter().any(|author| !valid_text(author, 160))
            || self
                .container_title
                .as_ref()
                .is_some_and(|venue| !valid_text(venue, 256))
            || self
                .published_year
                .is_some_and(|year| !(1000..=9999).contains(&year))
            || self.license_urls.len() > 8
            || self
                .license_urls
                .iter()
                .any(|link| !valid_license_url(link))
        {
            return Err(LiteratureError::InvalidCitation);
        }
        self.doi = self.doi.trim().to_ascii_lowercase();
        self.title = self.title.trim().to_owned();
        self.authors
            .iter_mut()
            .for_each(|author| *author = author.trim().to_owned());
        self.container_title = self.container_title.map(|title| title.trim().to_owned());
        self.license_urls.sort();
        self.license_urls.dedup();
        Ok(self)
    }
}

fn valid_text(value: &str, max: usize) -> bool {
    value.len() <= max && !value.trim().is_empty() && !value.chars().any(char::is_control)
}

fn valid_license_url(value: &str) -> bool {
    value.len() <= 512
        && value.parse::<ureq::http::Uri>().is_ok_and(|uri| {
            matches!(uri.scheme_str(), Some("https" | "http"))
                && uri.authority().is_some_and(|authority| {
                    !authority.host().is_empty() && !authority.as_str().contains('@')
                })
        })
}

fn observed_now(source: CitationSource) -> CitationProvenance {
    CitationProvenance {
        source,
        observed_at_unix_seconds: SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs(),
    }
}

pub struct CitationCatalog {
    db: Connection,
}

impl CitationCatalog {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, LiteratureError> {
        let db = Connection::open(path)?;
        db.busy_timeout(Duration::from_secs(3))?;
        db.execute_batch(
            "CREATE TABLE IF NOT EXISTS citations (
                doi TEXT PRIMARY KEY NOT NULL,
                year INTEGER,
                search_text TEXT NOT NULL,
                record TEXT NOT NULL
            ) STRICT;",
        )?;
        Ok(Self { db })
    }

    /// Import a user-supplied batch as manual metadata regardless of received provenance.
    pub fn import_manual(&mut self, citations: Vec<Citation>) -> Result<usize, LiteratureError> {
        let provenance = observed_now(CitationSource::Manual);
        self.import(
            citations
                .into_iter()
                .map(|citation| Citation {
                    provenance: provenance.clone(),
                    ..citation
                })
                .collect(),
        )
    }

    /// Import API-observed records after the caller explicitly requests saving them.
    /// DOI is unique, so repeated imports preserve the first snapshot.
    pub fn import(&mut self, citations: Vec<Citation>) -> Result<usize, LiteratureError> {
        if citations.len() > MAX_IMPORT {
            return Err(LiteratureError::InvalidCitation);
        }
        let batch = citations
            .into_iter()
            .map(Citation::normalized)
            .collect::<Result<Vec<_>, _>>()?;
        let tx = self
            .db
            .transaction_with_behavior(TransactionBehavior::Immediate)?;
        let count: u32 = tx.query_row("SELECT count(*) FROM citations", [], |row| row.get(0))?;
        let remaining = MAX_CITATIONS.saturating_sub(count) as usize;
        let mut inserted = 0;
        for citation in batch {
            let search_text = format!(
                "{} {} {} {}",
                citation.doi,
                citation.title,
                citation.authors.join(" "),
                citation.container_title.as_deref().unwrap_or_default(),
            )
            .to_lowercase();
            inserted += tx.execute(
                "INSERT INTO citations (doi, year, search_text, record) VALUES (?1, ?2, ?3, ?4)
                 ON CONFLICT(doi) DO NOTHING",
                params![
                    citation.doi,
                    citation.published_year,
                    search_text,
                    serde_json::to_string(&citation)?
                ],
            )?;
            if inserted > remaining {
                return Err(LiteratureError::CatalogLimit);
            }
        }
        tx.commit()?;
        Ok(inserted)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Citation>, LiteratureError> {
        validate_query(query, limit, true)?;
        let mut statement = self.db.prepare(
            "SELECT record FROM citations
             WHERE ?1 = '' OR instr(search_text, ?1) > 0
             ORDER BY year DESC NULLS LAST, doi ASC LIMIT ?2",
        )?;
        statement
            .query_map(params![query.trim().to_lowercase(), limit as u16], |row| {
                row.get::<_, String>(0)
            })?
            .map(|row| Ok(serde_json::from_str(&row?)?))
            .collect()
    }
}

fn validate_query(query: &str, limit: usize, allow_empty: bool) -> Result<(), LiteratureError> {
    if !(1..=MAX_RESULTS).contains(&limit)
        || query.len() > 256
        || query.chars().any(char::is_control)
        || (!allow_empty && query.trim().is_empty())
    {
        return Err(LiteratureError::InvalidQuery);
    }
    Ok(())
}

/// One explicit metadata request to the public Crossref endpoint. No article
/// URL, abstract, attachment or content download is part of this request.
pub fn search_crossref(query: &str, limit: usize) -> Result<Vec<Citation>, LiteratureError> {
    validate_query(query, limit, false)?;
    let mut slot = CROSSREF_SLOT
        .try_lock()
        .map_err(|_| LiteratureError::RateLimited {
            retry_after_secs: 1,
        })?;
    if let Some(wait) = slot.and_then(|next| next.checked_duration_since(Instant::now())) {
        return Err(LiteratureError::RateLimited {
            retry_after_secs: wait.as_secs().max(1),
        });
    }
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(8)))
        .build()
        .into();
    let response = agent
        .get(CROSSREF_WORKS)
        .header(
            "User-Agent",
            "AETHER-Research-Laboratory/0.1 (explicit bibliographic lookup)",
        )
        .query("query.bibliographic", query.trim())
        .query("rows", limit.to_string())
        .query(
            "select",
            "DOI,title,author,container-title,published,license",
        )
        .call();
    *slot = Some(Instant::now() + Duration::from_secs(1));
    let mut response = response.map_err(|_| LiteratureError::Upstream)?;
    if response.status().as_u16() == 429 {
        let wait = retry_after_seconds(
            response
                .headers()
                .get("retry-after")
                .and_then(|value| value.to_str().ok()),
        );
        *slot = Some(Instant::now() + Duration::from_secs(wait));
        return Err(LiteratureError::RateLimited {
            retry_after_secs: wait,
        });
    }
    if !response.status().is_success() {
        return Err(LiteratureError::Upstream);
    }
    let envelope: CrossrefEnvelope = response
        .body_mut()
        .with_config()
        .limit(2 * 1024 * 1024)
        .read_json()
        .map_err(|_| LiteratureError::InvalidResponse)?;
    crossref_citations(envelope, limit)
}

fn retry_after_seconds(header: Option<&str>) -> u64 {
    // Numeric Retry-After is typical. A date or absent header gets a
    // conservative delay. A cap keeps the process-local timer representable.
    header
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(60)
        .clamp(1, 3600)
}

#[derive(Deserialize)]
struct CrossrefEnvelope {
    status: String,
    message: CrossrefMessage,
}

#[derive(Deserialize)]
struct CrossrefMessage {
    items: Vec<CrossrefItem>,
}

#[derive(Deserialize)]
struct CrossrefItem {
    #[serde(rename = "DOI")]
    doi: String,
    #[serde(default)]
    title: Vec<String>,
    #[serde(default)]
    author: Vec<CrossrefAuthor>,
    #[serde(rename = "container-title", default)]
    container_title: Vec<String>,
    published: Option<CrossrefDate>,
    #[serde(default)]
    license: Vec<CrossrefLicense>,
}

#[derive(Deserialize)]
struct CrossrefAuthor {
    given: Option<String>,
    family: Option<String>,
    name: Option<String>,
}

#[derive(Deserialize)]
struct CrossrefDate {
    #[serde(rename = "date-parts")]
    parts: Vec<Vec<Option<u16>>>,
}

#[derive(Deserialize)]
struct CrossrefLicense {
    #[serde(rename = "URL")]
    url: String,
}

fn crossref_citations(
    envelope: CrossrefEnvelope,
    limit: usize,
) -> Result<Vec<Citation>, LiteratureError> {
    if envelope.status != "ok" || envelope.message.items.len() > limit {
        return Err(LiteratureError::InvalidResponse);
    }
    let provenance = observed_now(CitationSource::Crossref);
    Ok(envelope
        .message
        .items
        .into_iter()
        .filter_map(|item| {
            let title = item
                .title
                .into_iter()
                .find(|value| valid_text(value, 1024))?;
            let authors = item
                .author
                .into_iter()
                .map(|author| {
                    author.name.unwrap_or_else(|| {
                        format!(
                            "{} {}",
                            author.given.unwrap_or_default(),
                            author.family.unwrap_or_default()
                        )
                    })
                })
                .filter(|author| valid_text(author, 160))
                .take(32)
                .collect();
            let container_title = item
                .container_title
                .into_iter()
                .find(|value| valid_text(value, 256));
            let published_year = item
                .published
                .and_then(|date| date.parts.first()?.first().copied().flatten());
            let license_urls = item
                .license
                .into_iter()
                .map(|entry| entry.url)
                .filter(|link| valid_license_url(link))
                .take(8)
                .collect();
            Citation {
                doi: item.doi,
                title,
                authors,
                container_title,
                published_year,
                license_urls,
                provenance: provenance.clone(),
            }
            .normalized()
            .ok()
        })
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn citation(doi: &str, title: &str) -> Citation {
        Citation {
            doi: doi.to_owned(),
            title: title.to_owned(),
            authors: vec!["Model Researcher".into()],
            container_title: None,
            published_year: Some(2025),
            license_urls: Vec::new(),
            provenance: observed_now(CitationSource::Crossref),
        }
    }

    #[test]
    fn durable_catalog_is_transactional_and_searches_literals() {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!(
            "aether-literature-{}-{nonce}.sqlite3",
            std::process::id()
        ));
        let mut catalog = CitationCatalog::open(&path).unwrap();
        assert!(
            catalog
                .import_manual(vec![
                    citation("10.1234/a", "valid"),
                    citation("bad-doi", "invalid")
                ])
                .is_err()
        );
        assert!(catalog.search("", 20).unwrap().is_empty());
        let record = citation("10.1234/A", "Repeatability 100%");
        assert_eq!(catalog.import_manual(vec![record.clone()]).unwrap(), 1);
        assert_eq!(catalog.import(vec![record]).unwrap(), 0);
        drop(catalog);
        let catalog = CitationCatalog::open(&path).unwrap();
        let saved = catalog.search("ABILITY 100%", 20).unwrap();
        assert_eq!(saved.len(), 1);
        assert_eq!(saved[0].provenance.source, CitationSource::Manual);
        assert_eq!(saved[0].doi, "10.1234/a");
        assert!(catalog.search("_", 20).unwrap().is_empty());
        drop(catalog);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn crossref_parser_retains_metadata_without_trusting_license_links() {
        let envelope = serde_json::from_str(r#"{
            "status":"ok", "message":{"items":[{
                "DOI":"10.1234/example", "title":["An example"],
                "author":[{"given":"Research","family":"Example"}],
                "published":{"date-parts":[[2025,3,4]]},
                "license":[{"URL":"javascript:bad"},{"URL":"https://creativecommons.org/licenses/by/4.0/"}]
            }]}
        }"#).unwrap();
        let citations = crossref_citations(envelope, 2).unwrap();
        assert_eq!(citations[0].published_year, Some(2025));
        assert_eq!(citations[0].authors, ["Research Example"]);
        assert_eq!(citations[0].license_urls.len(), 1);
        assert_eq!(citations[0].provenance.source, CitationSource::Crossref);
        assert_eq!(retry_after_seconds(Some("120")), 120);
        assert_eq!(retry_after_seconds(None), 60);
    }
}
