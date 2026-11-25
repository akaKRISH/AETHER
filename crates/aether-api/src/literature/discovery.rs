use std::sync::Mutex;
use std::time::{Duration, Instant};

use roxmltree::{Document, Node, ParsingOptions};
use serde::{Deserialize, Serialize};

use super::{
    Citation, CitationProvenance, CitationSource, LiteratureError, observed_now, valid_doi,
    valid_license_url, valid_text, validate_query,
};

const MAX_RESPONSE_BYTES: u64 = 2 * 1024 * 1024;
const ATOM: &str = "http://www.w3.org/2005/Atom";
const ARXIV_NS: &str = "http://arxiv.org/schemas/atom";

type LookupSlot = Mutex<Option<(Instant, Duration)>>;

static CROSSREF_SLOT: LookupSlot = Mutex::new(None);
static OPENALEX_SLOT: LookupSlot = Mutex::new(None);
static ARXIV_SLOT: LookupSlot = Mutex::new(None);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataProvider {
    #[default]
    Crossref,
    #[serde(rename = "openalex")]
    OpenAlex,
    Arxiv,
}

/// Make one bounded request to a fixed metadata origin. The caller authorizes
/// the exact bibliographic query. Returned links are never requested here.
pub fn search_metadata(
    provider: MetadataProvider,
    query: &str,
    limit: usize,
) -> Result<Vec<Citation>, LiteratureError> {
    validate_query(query, limit, false)?;
    let (slot, cooldown) = match provider {
        MetadataProvider::Crossref => (&CROSSREF_SLOT, 1),
        MetadataProvider::OpenAlex => (&OPENALEX_SLOT, 3),
        MetadataProvider::Arxiv => (&ARXIV_SLOT, 3),
    };
    let mut slot = slot.try_lock().map_err(|_| LiteratureError::RateLimited {
        retry_after_secs: cooldown,
    })?;
    if let Some((started, delay)) = *slot
        && let Some(wait) = delay.checked_sub(started.elapsed())
        && !wait.is_zero()
    {
        return Err(LiteratureError::RateLimited {
            retry_after_secs: wait.as_secs().saturating_add(1),
        });
    }

    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .http_status_as_error(false)
        .timeout_global(Some(Duration::from_secs(8)))
        .build()
        .into();
    let query = query.trim();
    let count = limit.to_string();
    let request = match provider {
        MetadataProvider::Crossref => agent
            .get("https://api.crossref.org/works")
            .query("query.bibliographic", query)
            .query("rows", &count)
            .query(
                "select",
                "DOI,title,author,container-title,published,license",
            ),
        MetadataProvider::OpenAlex => agent
            .get("https://api.openalex.org/works")
            .query("search", query)
            .query("per_page", &count)
            .query(
                "select",
                "id,doi,display_name,authorships,publication_year,primary_location,topics",
            ),
        MetadataProvider::Arxiv => agent
            .get("https://export.arxiv.org/api/query")
            .query("search_query", format!("all:{query}"))
            .query("max_results", &count),
    };
    let response = request
        .header(
            "User-Agent",
            "AETHER-Research-Laboratory/0.1 (explicit bibliographic lookup)",
        )
        .call();
    *slot = Some((Instant::now(), Duration::from_secs(cooldown)));
    let mut response = response.map_err(|_| LiteratureError::Upstream)?;
    if response.status().as_u16() == 429 {
        let wait = retry_after_seconds(
            response
                .headers()
                .get("retry-after")
                .and_then(|value| value.to_str().ok()),
        );
        *slot = Some((Instant::now(), Duration::from_secs(wait)));
        return Err(LiteratureError::RateLimited {
            retry_after_secs: wait,
        });
    }
    if !response.status().is_success() {
        return Err(LiteratureError::Upstream);
    }
    let body = response
        .body_mut()
        .with_config()
        .limit(MAX_RESPONSE_BYTES)
        .read_to_string()
        .map_err(|_| LiteratureError::InvalidResponse)?;
    *slot = Some((Instant::now(), Duration::from_secs(cooldown)));
    let timestamp = observed_now(CitationSource::from(provider)).observed_at_unix_seconds;
    parse_metadata(provider, &body, limit, timestamp)
}

impl From<MetadataProvider> for CitationSource {
    fn from(provider: MetadataProvider) -> Self {
        match provider {
            MetadataProvider::Crossref => Self::Crossref,
            MetadataProvider::OpenAlex => Self::OpenAlex,
            MetadataProvider::Arxiv => Self::Arxiv,
        }
    }
}

pub(super) fn parse_metadata(
    provider: MetadataProvider,
    body: &str,
    limit: usize,
    observed_at_unix_seconds: u64,
) -> Result<Vec<Citation>, LiteratureError> {
    if body.len() as u64 > MAX_RESPONSE_BYTES {
        return Err(LiteratureError::InvalidResponse);
    }
    let provenance = CitationProvenance {
        source: provider.into(),
        observed_at_unix_seconds,
    };
    match provider {
        MetadataProvider::Crossref => crossref_citations(decode_json(body)?, limit, provenance),
        MetadataProvider::OpenAlex => openalex_citations(decode_json(body)?, limit, provenance),
        MetadataProvider::Arxiv => arxiv_citations(body, limit, provenance),
    }
}

fn decode_json<T: serde::de::DeserializeOwned>(body: &str) -> Result<T, LiteratureError> {
    serde_json::from_str(body).map_err(|_| LiteratureError::InvalidResponse)
}

fn retry_after_seconds(header: Option<&str>) -> u64 {
    header
        .and_then(|value| value.trim().parse::<u64>().ok())
        .unwrap_or(60)
        .max(3)
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
    provenance: CitationProvenance,
) -> Result<Vec<Citation>, LiteratureError> {
    if envelope.status != "ok" || envelope.message.items.len() > limit {
        return Err(LiteratureError::InvalidResponse);
    }
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
                doi: Some(item.doi),
                arxiv_id: None,
                openalex_id: None,
                related_doi: None,
                topics: Vec::new(),
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

#[derive(Deserialize)]
struct OpenAlexEnvelope {
    results: Vec<OpenAlexWork>,
}

#[derive(Deserialize)]
struct OpenAlexWork {
    id: String,
    doi: Option<String>,
    #[serde(alias = "display_name")]
    title: Option<String>,
    #[serde(default)]
    authorships: Vec<OpenAlexAuthorship>,
    publication_year: Option<u16>,
    primary_location: Option<OpenAlexLocation>,
    #[serde(default)]
    topics: Vec<OpenAlexLabel>,
}

#[derive(Deserialize)]
struct OpenAlexAuthorship {
    author: OpenAlexLabel,
}

#[derive(Deserialize)]
struct OpenAlexLocation {
    source: Option<OpenAlexLabel>,
}

#[derive(Deserialize)]
struct OpenAlexLabel {
    display_name: Option<String>,
}

fn openalex_citations(
    envelope: OpenAlexEnvelope,
    limit: usize,
    provenance: CitationProvenance,
) -> Result<Vec<Citation>, LiteratureError> {
    if envelope.results.len() > limit {
        return Err(LiteratureError::InvalidResponse);
    }
    Ok(envelope
        .results
        .into_iter()
        .filter_map(|item| {
            let id = item.id.strip_prefix("https://openalex.org/")?;
            let doi = item.doi.and_then(|doi| {
                let normalized = doi.strip_prefix("https://doi.org/").unwrap_or(&doi);
                valid_doi(normalized).then(|| normalized.to_owned())
            });
            Citation {
                doi,
                arxiv_id: None,
                openalex_id: Some(id.to_owned()),
                related_doi: None,
                title: item.title?,
                authors: item
                    .authorships
                    .into_iter()
                    .filter_map(|entry| entry.author.display_name)
                    .filter(|name| valid_text(name, 160))
                    .take(32)
                    .collect(),
                container_title: item
                    .primary_location
                    .and_then(|location| location.source?.display_name)
                    .filter(|title| valid_text(title, 256)),
                published_year: item.publication_year,
                topics: item
                    .topics
                    .into_iter()
                    .filter_map(|topic| topic.display_name)
                    .filter(|label| valid_text(label, 120))
                    .take(16)
                    .collect(),
                license_urls: Vec::new(),
                provenance: provenance.clone(),
            }
            .normalized()
            .ok()
        })
        .collect())
}

fn arxiv_citations(
    body: &str,
    limit: usize,
    provenance: CitationProvenance,
) -> Result<Vec<Citation>, LiteratureError> {
    let options = ParsingOptions {
        nodes_limit: 25_000,
        ..ParsingOptions::default()
    };
    let document = Document::parse_with_options(body, options)
        .map_err(|_| LiteratureError::InvalidResponse)?;
    let feed = document.root_element();
    if !feed.has_tag_name((ATOM, "feed")) {
        return Err(LiteratureError::InvalidResponse);
    }
    let entries: Vec<_> = feed
        .children()
        .filter(|node| node.has_tag_name((ATOM, "entry")))
        .take(limit.saturating_add(1))
        .collect();
    if entries.len() > limit {
        return Err(LiteratureError::InvalidResponse);
    }
    let mut records = Vec::new();
    for entry in entries {
        let raw_id = xml_child_text(entry, ATOM, "id").ok_or(LiteratureError::InvalidResponse)?;
        let id = raw_id
            .strip_prefix("https://arxiv.org/abs/")
            .or_else(|| raw_id.strip_prefix("http://arxiv.org/abs/"))
            .ok_or(LiteratureError::InvalidResponse)?;
        if let Some(record) = arxiv_record(entry, id, &provenance) {
            records.push(record);
        }
    }
    Ok(records)
}

fn arxiv_record(
    entry: Node<'_, '_>,
    id: &str,
    provenance: &CitationProvenance,
) -> Option<Citation> {
    let title = xml_child_text(entry, ATOM, "title")?;
    let year =
        xml_child_text(entry, ATOM, "published").and_then(|date| date.get(..4)?.parse().ok());
    Citation {
        doi: None,
        arxiv_id: Some(id.to_owned()),
        openalex_id: None,
        related_doi: xml_child_text(entry, ARXIV_NS, "doi")
            .filter(|doi| valid_doi(doi))
            .map(str::to_owned),
        title: title.split_whitespace().collect::<Vec<_>>().join(" "),
        authors: entry
            .children()
            .filter(|node| node.has_tag_name((ATOM, "author")))
            .filter_map(|node| xml_child_text(node, ATOM, "name"))
            .filter(|name| valid_text(name, 160))
            .take(32)
            .map(str::to_owned)
            .collect(),
        container_title: Some("arXiv".into()),
        published_year: year,
        topics: entry
            .children()
            .filter(|node| node.has_tag_name((ATOM, "category")))
            .filter_map(|node| node.attribute("term"))
            .filter(|topic| valid_text(topic, 120))
            .take(16)
            .map(str::to_owned)
            .collect(),
        license_urls: Vec::new(),
        provenance: provenance.clone(),
    }
    .normalized()
    .ok()
}

fn xml_child_text<'a>(node: Node<'a, '_>, ns: &str, name: &str) -> Option<&'a str> {
    node.children()
        .find(|child| child.has_tag_name((ns, name)))?
        .text()
}

#[cfg(test)]
mod tests {
    use super::retry_after_seconds;

    #[test]
    fn retry_after_uses_provider_delay_or_a_conservative_fallback() {
        assert_eq!(retry_after_seconds(Some("120")), 120);
        assert_eq!(retry_after_seconds(Some("86400")), 86400);
        assert_eq!(retry_after_seconds(Some("0")), 3);
        assert_eq!(retry_after_seconds(None), 60);
    }
}
