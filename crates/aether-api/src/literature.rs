mod archive;
mod discovery;
mod knowledge;

pub use archive::{ArchiveWork, TextManifest};
pub use discovery::{MetadataPage, MetadataProvider};
pub use knowledge::{GraphBatch, GraphExport, Kind as GraphKind};

use std::path::Path;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use rusqlite::{Connection, OptionalExtension, TransactionBehavior, params};
use serde::{Deserialize, Serialize};
use thiserror::Error;

const MAX_RESULTS: usize = 20;
const MAX_IMPORT: usize = 100;
const MAX_CITATIONS: u32 = 5_000;

#[derive(Debug, Error)]
pub enum LiteratureError {
    #[error("use a bounded citation query of up to 256 bytes and a limit of 1–20")]
    InvalidQuery,
    #[error("invalid or oversized citation metadata")]
    InvalidCitation,
    #[error("the citation catalog has reached its 5,000-record limit")]
    CatalogLimit,
    #[error("citation identifiers match different saved works")]
    AliasConflict,
    #[error("citation ID was not found in the local catalog")]
    MissingCitation,
    #[error("metadata lookup is busy or rate-limited; retry in {retry_after_secs} seconds")]
    RateLimited { retry_after_secs: u64 },
    #[error("metadata provider is unavailable")]
    Upstream,
    #[error("metadata provider returned invalid or oversized metadata")]
    InvalidResponse,
    #[error("expected bounded UTF-8 full text with an approved license and HTTPS source")]
    InvalidText,
    #[error("literature archive capacity reached")]
    ArchiveLimit,
    #[error("literature text could not be read")]
    TextIo,
    #[error("invalid knowledge graph: {0}")]
    InvalidGraph(&'static str),
    #[error("knowledge graph node not found")]
    MissingGraphNode,
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
    #[serde(rename = "openalex")]
    OpenAlex,
    Arxiv,
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
    #[serde(default)]
    pub doi: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub arxiv_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub openalex_id: Option<String>,
    /// A different publication named by the source, whose identity remains separate.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_doi: Option<String>,
    /// Source-reported outgoing identifiers. None means coverage is unknown.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub references: Option<Vec<String>>,
    #[serde(default, skip_serializing_if = "std::ops::Not::not")]
    pub references_truncated: bool,
    pub title: String,
    #[serde(default)]
    pub authors: Vec<String>,
    #[serde(default)]
    pub container_title: Option<String>,
    #[serde(default)]
    pub published_year: Option<u16>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub topics: Vec<String>,
    /// Source-reported rights metadata. Absence leaves publication rights unknown.
    #[serde(default)]
    pub license_urls: Vec<String>,
    #[serde(default)]
    pub provenance: CitationProvenance,
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct FrozenCitation {
    pub id: String,
    title: String,
    url: String,
    year: Option<u16>,
    provenance: CitationProvenance,
}

impl Citation {
    fn normalized(mut self) -> Result<Self, LiteratureError> {
        for doi in self.doi.iter_mut().chain(&mut self.related_doi) {
            *doi = normalized_doi(doi).ok_or(LiteratureError::InvalidCitation)?;
        }
        if let Some(id) = &mut self.arxiv_id {
            *id = normalize_arxiv(id);
        }
        if let Some(id) = &mut self.openalex_id {
            *id = normalize_openalex(id);
        }
        if let Some(references) = &mut self.references {
            if references.len() > 128 {
                return Err(LiteratureError::InvalidCitation);
            }
            for id in references.iter_mut() {
                *id = paper_id(id).ok_or(LiteratureError::InvalidCitation)?;
            }
            references.sort_unstable();
            references.dedup();
        }
        if self.doi.is_none() && self.arxiv_id.is_none() && self.openalex_id.is_none() {
            return Err(LiteratureError::InvalidCitation);
        }
        if self
            .doi
            .iter()
            .chain(&self.related_doi)
            .any(|doi| !valid_doi(doi))
            || self
                .arxiv_id
                .as_deref()
                .is_some_and(|id| !valid_arxiv_id(id))
            || self
                .openalex_id
                .as_deref()
                .is_some_and(|id| !valid_openalex_id(id))
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
            || self.topics.len() > 16
            || self.topics.iter().any(|topic| !valid_text(topic, 120))
            || self.license_urls.len() > 8
            || self
                .license_urls
                .iter()
                .any(|link| !valid_license_url(link))
        {
            return Err(LiteratureError::InvalidCitation);
        }
        self.title = self.title.trim().to_owned();
        self.authors
            .iter_mut()
            .for_each(|author| *author = author.trim().to_owned());
        self.container_title = self.container_title.map(|title| title.trim().to_owned());
        self.topics
            .iter_mut()
            .for_each(|topic| *topic = topic.trim().to_owned());
        self.topics.sort_unstable();
        self.topics.dedup();
        self.license_urls.sort();
        self.license_urls.dedup();
        Ok(self)
    }

    fn aliases(&self) -> Vec<String> {
        let mut aliases = Vec::new();
        aliases.extend(self.doi.iter().cloned());
        aliases.extend(
            self.arxiv_id
                .iter()
                .map(|value| format!("arxiv:{}", arxiv_root(value))),
        );
        if let Some(id) = self
            .doi
            .as_deref()
            .and_then(|doi| doi.strip_prefix("10.48550/arxiv."))
            && valid_arxiv_id(id)
        {
            aliases.push(format!("arxiv:{}", arxiv_root(id)));
        }
        aliases.dedup();
        aliases.extend(
            self.openalex_id
                .iter()
                .map(|value| format!("openalex:{value}")),
        );
        aliases
    }

    pub fn source_url(&self) -> String {
        if let Some(doi) = &self.doi {
            format!("https://doi.org/{doi}")
        } else if let Some(id) = &self.arxiv_id {
            format!("https://arxiv.org/abs/{id}")
        } else if let Some(id) = &self.openalex_id {
            format!("https://openalex.org/{id}")
        } else {
            String::new()
        }
    }
}

fn normalized_doi(value: &str) -> Option<String> {
    let value = value.trim().to_ascii_lowercase();
    let doi = [
        "https://doi.org/",
        "http://doi.org/",
        "https://dx.doi.org/",
        "doi:",
    ]
    .into_iter()
    .find_map(|prefix| value.strip_prefix(prefix))
    .unwrap_or(&value);
    valid_doi(doi).then(|| doi.to_owned())
}

fn normalize_arxiv(value: &str) -> String {
    let value = value.trim().to_ascii_lowercase();
    ["https://arxiv.org/abs/", "http://arxiv.org/abs/", "arxiv:"]
        .into_iter()
        .find_map(|prefix| value.strip_prefix(prefix))
        .unwrap_or(&value)
        .to_owned()
}

fn normalize_openalex(value: &str) -> String {
    let value = value.trim().to_ascii_lowercase();
    ["https://openalex.org/", "openalex:"]
        .into_iter()
        .find_map(|prefix| value.strip_prefix(prefix))
        .unwrap_or(&value)
        .to_ascii_uppercase()
}

fn paper_id(value: &str) -> Option<String> {
    if value.len() > 300 {
        return None;
    }
    let arxiv = normalize_arxiv(value);
    let openalex = normalize_openalex(value);
    normalized_doi(value)
        .or_else(|| valid_arxiv_id(&arxiv).then(|| format!("arxiv:{arxiv}")))
        .or_else(|| valid_openalex_id(&openalex).then(|| format!("openalex:{openalex}")))
}

fn valid_doi(value: &str) -> bool {
    let Some((prefix, suffix)) = value.trim().split_once('/') else {
        return false;
    };
    let registration = prefix.strip_prefix("10.").unwrap_or_default();
    value.len() <= 256
        && (4..=9).contains(&registration.len())
        && registration.bytes().all(|byte| byte.is_ascii_digit())
        && !suffix.is_empty()
        && suffix
            .bytes()
            .all(|byte| byte.is_ascii_alphanumeric() || b"-._;()/:+".contains(&byte))
}

fn arxiv_root(id: &str) -> &str {
    id.rsplit_once('v')
        .filter(|(_, version)| {
            !version.is_empty() && version.bytes().all(|byte| byte.is_ascii_digit())
        })
        .map_or(id, |(root, _)| root)
}

fn valid_arxiv_id(id: &str) -> bool {
    if id.len() > 48 {
        return false;
    }
    let root = arxiv_root(id);
    if root != id && id[root.len() + 1..].starts_with('0') {
        return false;
    }
    if let Some((category, number)) = root.split_once('/') {
        !category.is_empty()
            && category.as_bytes()[0].is_ascii_alphabetic()
            && category
                .bytes()
                .all(|byte| byte.is_ascii_alphabetic() || b"-.".contains(&byte))
            && number.len() == 7
            && number.bytes().all(|byte| byte.is_ascii_digit())
    } else if let Some((month, sequence)) = root.split_once('.') {
        month.len() == 4
            && month.bytes().all(|byte| byte.is_ascii_digit())
            && (4..=5).contains(&sequence.len())
            && sequence.bytes().all(|byte| byte.is_ascii_digit())
    } else {
        false
    }
}

fn valid_openalex_id(id: &str) -> bool {
    id.strip_prefix('W').is_some_and(|digits| {
        (1..=20).contains(&digits.len()) && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
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
            "PRAGMA foreign_keys = ON;
            CREATE TABLE IF NOT EXISTS citations (
                doi TEXT PRIMARY KEY NOT NULL,
                year INTEGER,
                search_text TEXT NOT NULL,
                record TEXT NOT NULL
            ) STRICT;
            CREATE TABLE IF NOT EXISTS citation_aliases (
                alias TEXT PRIMARY KEY NOT NULL,
                work TEXT NOT NULL REFERENCES citations(doi)
            ) STRICT;
            CREATE INDEX IF NOT EXISTS citation_aliases_work ON citation_aliases(work);",
        )?;
        archive::initialize(&db)?;
        knowledge::initialize(&db)?;
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
    /// Repeated source identifiers preserve the first recorded snapshot.
    pub fn import(&mut self, citations: Vec<Citation>) -> Result<usize, LiteratureError> {
        self.import_observations(citations, None)
    }

    fn import_observations(
        &mut self,
        citations: Vec<Citation>,
        page: Option<&MetadataPage>,
    ) -> Result<usize, LiteratureError> {
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
            let aliases = citation.aliases();
            let mut existing = None;
            for alias in &aliases {
                let matched: Option<String> = tx
                    .query_row(
                        "SELECT work FROM citation_aliases WHERE alias = ?1
                     UNION SELECT doi FROM citations WHERE doi = ?1",
                        [alias],
                        |row| row.get(0),
                    )
                    .optional()?;
                if let Some(matched) = matched {
                    if existing.as_ref().is_some_and(|work| *work != matched) {
                        return Err(LiteratureError::AliasConflict);
                    }
                    existing = Some(matched);
                }
            }
            let work = existing.as_deref().unwrap_or(&aliases[0]);
            if existing.is_none() {
                inserted += 1;
                if inserted > remaining {
                    return Err(LiteratureError::CatalogLimit);
                }
                let search_text = format!(
                    "{} {} {} {} {}",
                    citation.title,
                    citation.authors.join(" "),
                    citation.container_title.as_deref().unwrap_or_default(),
                    citation.topics.join(" "),
                    citation.arxiv_id.as_deref().unwrap_or_default(),
                )
                .to_lowercase();
                tx.execute(
                    "INSERT INTO citations (doi, year, search_text, record) VALUES (?1, ?2, ?3, ?4)",
                    params![work, citation.published_year, search_text, serde_json::to_string(&citation)?],
                )?;
            }
            for alias in &aliases {
                tx.execute(
                    "INSERT INTO citation_aliases (alias, work) VALUES (?1, ?2)
                     ON CONFLICT(alias) DO NOTHING",
                    params![alias, work],
                )?;
            }
            let alias_count: u32 = tx.query_row(
                "SELECT count(*) FROM citation_aliases WHERE work = ?1",
                [work],
                |row| row.get(0),
            )?;
            if alias_count > 12 {
                return Err(LiteratureError::InvalidCitation);
            }
            archive::save_snapshot(&tx, work, &citation, page)?;
        }
        knowledge::enforce_graph_bounds(&tx)?;
        tx.commit()?;
        Ok(inserted)
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<Citation>, LiteratureError> {
        validate_query(query, limit, true)?;
        let mut statement = self.db.prepare(
            "SELECT record FROM citations AS citation
             WHERE ?1 = '' OR instr(search_text, ?1) > 0 OR instr(lower(doi), ?1) > 0
                OR EXISTS (
                    SELECT 1 FROM citation_aliases
                    WHERE work = citation.doi AND instr(lower(alias), ?1) > 0
                )
                OR EXISTS (
                    SELECT 1 FROM literature_snapshots
                    WHERE work = citation.doi AND (
                        instr(lower(json_extract(record, '$.citation.arxiv_id')), ?1) > 0 OR
                        instr(lower(json_extract(record, '$.citation.title')), ?1) > 0
                    )
                )
                OR EXISTS (
                    SELECT 1 FROM literature_texts
                    WHERE work = citation.doi AND instr(search_text, ?1) > 0
                )
             ORDER BY year DESC NULLS LAST, doi ASC LIMIT ?2",
        )?;
        statement
            .query_map(params![query.trim().to_lowercase(), limit as u16], |row| {
                row.get::<_, String>(0)
            })?
            .map(|row| serde_json::from_str::<Citation>(&row?)?.normalized())
            .collect()
    }

    pub fn freeze(&self, id: &str) -> Result<FrozenCitation, LiteratureError> {
        let (id, work) = self.resolve_work(id)?;
        let version = id
            .strip_prefix("arxiv:")
            .filter(|value| *value != arxiv_root(value));
        let record: String = self
            .db
            .query_row(
                "SELECT record FROM citations WHERE doi = ?1",
                [&work],
                |row| row.get(0),
            )
            .optional()?
            .ok_or(LiteratureError::MissingCitation)?;
        let mut citation = serde_json::from_str::<Citation>(&record)?.normalized()?;
        if let Some(requested) = version
            && citation.arxiv_id.as_deref() != Some(requested)
        {
            citation = self
                .snapshots(&work)?
                .into_iter()
                .map(|snapshot| snapshot.citation)
                .find(|citation| citation.arxiv_id.as_deref() == Some(requested))
                .ok_or(LiteratureError::MissingCitation)?;
        }
        let url = version.map_or_else(
            || citation.source_url(),
            |id| format!("https://arxiv.org/abs/{id}"),
        );
        Ok(FrozenCitation {
            id,
            title: citation.title,
            url,
            year: citation.published_year,
            provenance: citation.provenance,
        })
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

#[cfg(test)]
mod tests {
    use super::*;

    fn citation(doi: &str, title: &str) -> Citation {
        Citation {
            doi: Some(doi.to_owned()),
            arxiv_id: None,
            openalex_id: None,
            related_doi: None,
            references: None,
            references_truncated: false,
            title: title.to_owned(),
            authors: vec!["Model Researcher".into()],
            container_title: None,
            published_year: Some(2025),
            topics: Vec::new(),
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
        assert_eq!(saved[0].doi.as_deref(), Some("10.1234/a"));
        assert!(catalog.search("_", 20).unwrap().is_empty());
        drop(catalog);
        std::fs::remove_file(path).unwrap();
    }

    #[test]
    fn crossref_parser_retains_metadata_without_trusting_license_links() {
        let citations = discovery::parse_metadata(MetadataProvider::Crossref, r#"{
            "status":"ok", "message":{"items":[{
                "DOI":"10.1234/example", "title":["An example"],
                "author":[{"given":"Research","family":"Example"}],
                "published":{"date-parts":[[2025,3,4]]},
                "license":[{"URL":"javascript:bad"},{"URL":"https://creativecommons.org/licenses/by/4.0/"}],
                "reference":[{"DOI":"https://doi.org/10.1234/OTHER"}, {"unstructured":"no identifier"}]
            }]}
        }"#, 2, 42).unwrap();
        assert_eq!(citations[0].published_year, Some(2025));
        assert_eq!(citations[0].authors, ["Research Example"]);
        assert_eq!(citations[0].license_urls.len(), 1);
        assert_eq!(
            citations[0].references.as_deref().unwrap(),
            ["10.1234/other"]
        );
        assert!(citations[0].references_truncated);
        assert_eq!(citations[0].provenance.source, CitationSource::Crossref);
        assert_eq!(citations[0].provenance.observed_at_unix_seconds, 42);
    }

    #[test]
    fn openalex_topics_and_doi_aliases_are_searchable() {
        let mut catalog = CitationCatalog::open(":memory:").unwrap();
        // Model an older DOI-only record without a row in the new alias table.
        let legacy = r#"{"doi":"10.1234/example","title":"A prior work"}"#;
        catalog.db.execute(
            "INSERT INTO citations (doi, search_text, record) VALUES ('10.1234/example', 'a prior work', ?1)",
            [legacy],
        ).unwrap();
        let records = discovery::parse_metadata(
            MetadataProvider::OpenAlex,
            r#"{"results":[
            {
                "id":"https://openalex.org/W101","doi":"https://doi.org/10.1234/EXAMPLE",
                "title":"A newer snapshot","publication_year":2025,
                "referenced_works":["https://openalex.org/W102","https://example.invalid/W103"]
            },
            {
                "id":"https://openalex.org/W102","doi":null,"display_name":"A DOI-less record",
                "authorships":[{"author":{"display_name":"Fixture Author"}}],
                "primary_location":{"source":{"display_name":"Fixture Journal"}},
                "topics":[{"display_name":"Computational methods"}]
            }
        ]}"#,
            2,
            123,
        )
        .unwrap();
        assert_eq!(catalog.import(records.clone()).unwrap(), 1);
        assert_eq!(catalog.import(records).unwrap(), 0);
        let by_identifier = catalog.search("w101", 20).unwrap();
        let snapshots = catalog.snapshots("10.1234/example").unwrap();
        assert_eq!(
            snapshots[0].citation.references.as_deref().unwrap(),
            ["openalex:W102"]
        );
        assert!(snapshots[0].citation.references_truncated);
        assert_eq!(by_identifier[0].title, "A prior work");
        assert_eq!(by_identifier[0].provenance.source, CitationSource::Manual);
        let by_topic = catalog.search("computational methods", 20).unwrap();
        assert_eq!(by_topic.len(), 1);
        assert_eq!(by_topic[0].source_url(), "https://openalex.org/W102");
        assert_eq!(by_topic[0].authors, ["Fixture Author"]);
        assert_eq!(by_topic[0].provenance.source, CitationSource::OpenAlex);
        assert_eq!(by_topic[0].provenance.observed_at_unix_seconds, 123);
    }

    #[test]
    fn arxiv_keeps_first_version_and_distinguishes_journal_doi() {
        let mut catalog = CitationCatalog::open(":memory:").unwrap();
        catalog
            .import_manual(vec![citation("10.1234/journal", "Journal version")])
            .unwrap();
        let feed = r#"
            <feed xmlns="http://www.w3.org/2005/Atom" xmlns:arxiv="http://arxiv.org/schemas/atom">
              <entry><id>http://arxiv.org/abs/2501.01234v1</id>
                <title>Determinism &amp; experiments</title><published>2025-01-12T00:00:00Z</published>
                <author><name>Fixture Author</name></author>
                <category term="cs.LG" />
                <arxiv:doi>10.1234/journal</arxiv:doi>
                <link href="https://example.invalid/ignore-this-full-text.pdf" type="application/pdf" />
              </entry>
            </feed>"#;
        let records = discovery::parse_metadata(MetadataProvider::Arxiv, feed, 1, 10).unwrap();
        assert_eq!(records[0].doi, None);
        assert_eq!(records[0].related_doi.as_deref(), Some("10.1234/journal"));
        assert_eq!(records[0].authors, ["Fixture Author"]);
        assert_eq!(records[0].title, "Determinism & experiments");
        assert_eq!(catalog.import(records).unwrap(), 1);
        assert!(
            catalog.snapshots("arxiv:2501.01234").unwrap()[0]
                .citation
                .references
                .is_none()
        );
        let newer_feed = feed.replace("01234v1", "01234v2");
        let newer = discovery::parse_metadata(MetadataProvider::Arxiv, &newer_feed, 1, 11).unwrap();
        assert_eq!(catalog.import(newer).unwrap(), 0);
        let result = catalog.search("cs.lg", 20).unwrap();
        assert_eq!(result[0].source_url(), "https://arxiv.org/abs/2501.01234v1");
        assert_eq!(result[0].provenance.observed_at_unix_seconds, 10);
        assert_eq!(catalog.search("", 20).unwrap().len(), 2);
        assert_eq!(catalog.search("2501.01234v1", 20).unwrap(), result);
        assert_eq!(catalog.search("2501.01234v2", 20).unwrap(), result);
        assert_eq!(
            catalog
                .freeze("https://arxiv.org/abs/2501.01234v2")
                .unwrap()
                .url,
            "https://arxiv.org/abs/2501.01234v2"
        );
        assert!(matches!(
            catalog.freeze("arxiv:2501.01234v3"),
            Err(LiteratureError::MissingCitation)
        ));
    }

    #[test]
    fn malformed_provider_metadata_is_bounded() {
        let dtd = r#"<!DOCTYPE feed [<!ENTITY external SYSTEM "file:///example">]>
            <feed xmlns="http://www.w3.org/2005/Atom"><title>&external;</title></feed>"#;
        assert!(matches!(
            discovery::parse_metadata(MetadataProvider::Arxiv, dtd, 1, 0),
            Err(LiteratureError::InvalidResponse)
        ));
        let error_feed = r#"<feed xmlns="http://www.w3.org/2005/Atom"><entry>
            <id>http://arxiv.org/api/errors#bad_query</id></entry></feed>"#;
        assert!(matches!(
            discovery::parse_metadata(MetadataProvider::Arxiv, error_feed, 1, 0),
            Err(LiteratureError::InvalidResponse)
        ));
        let json = r#"{"results":[{"id":"https://openalex.org/W1","title":"One"},
                                  {"id":"https://openalex.org/W2","title":"Two"}]}"#;
        assert!(matches!(
            discovery::parse_metadata(MetadataProvider::OpenAlex, json, 1, 0),
            Err(LiteratureError::InvalidResponse)
        ));
    }

    #[test]
    fn conflicting_alias_bridge_cannot_rewrite_or_partially_save_catalog() {
        let mut catalog = CitationCatalog::open(":memory:").unwrap();
        let mut separate = citation("10.1234/second", "Second");
        separate.openalex_id = Some("W77".into());
        catalog
            .import_manual(vec![citation("10.1234/first", "First"), separate])
            .unwrap();
        let mut conflicting = citation("10.1234/first", "Untrusted bridge");
        conflicting.openalex_id = Some("W77".into());
        assert!(matches!(
            catalog.import(vec![citation("10.1234/third", "Third"), conflicting]),
            Err(LiteratureError::AliasConflict)
        ));
        assert_eq!(catalog.search("", 20).unwrap().len(), 2);
        assert_eq!(catalog.search("w77", 20).unwrap()[0].title, "Second");
        assert!(catalog.search("10.1234/third", 20).unwrap().is_empty());
    }
}
