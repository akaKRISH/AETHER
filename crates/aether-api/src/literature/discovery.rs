use std::time::Duration;

use roxmltree::{Document, Node, ParsingOptions};
use serde::{Deserialize, Serialize};

use super::{
    Citation, CitationProvenance, CitationSource, LiteratureError, normalized_doi, observed_now,
    paper_id, valid_doi, valid_license_url, valid_text, validate_query,
};

const MAX_RESPONSE_BYTES: u64 = 2 * 1024 * 1024;
const ATOM: &str = "http://www.w3.org/2005/Atom";
const ARXIV_NS: &str = "http://arxiv.org/schemas/atom";

pub(super) const PAGE_LIMIT: u16 = 50;

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum MetadataProvider {
    #[default]
    Crossref,
    #[serde(rename = "openalex")]
    OpenAlex,
    Arxiv,
}

impl MetadataProvider {
    pub(super) fn settings(self) -> (&'static str, &'static str, u64) {
        match self {
            Self::Crossref => ("crossref", "https://api.crossref.org/works", 1),
            Self::OpenAlex => ("openalex", "https://api.openalex.org/works", 3),
            Self::Arxiv => ("arxiv", "https://export.arxiv.org/api/query", 3),
        }
    }

    pub fn enabled(self) -> bool {
        let flag = match self {
            Self::Crossref => "AETHER_ALLOW_CROSSREF",
            Self::OpenAlex => "AETHER_ALLOW_OPENALEX",
            Self::Arxiv => "AETHER_ALLOW_ARXIV",
        };
        std::env::var(flag).as_deref() == Ok("1")
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct MetadataPage {
    pub provider: MetadataProvider,
    pub endpoint: String,
    pub query: String,
    pub page: u16,
    pub page_size: usize,
    pub response_sha256: String,
    pub observed_at_unix_seconds: u64,
    pub next_page: Option<u16>,
    pub capped: bool,
    pub received: usize,
    pub records: Vec<Citation>,
    #[serde(default)]
    pub cached: bool,
}

/// Make one bounded request to a fixed metadata origin. The caller authorizes
/// the exact bibliographic query. Returned links are never requested here.
pub(super) fn fetch_page(
    provider: MetadataProvider,
    query: &str,
    limit: usize,
    page: u16,
) -> Result<MetadataPage, LiteratureError> {
    validate_query(query, limit, false)?;
    let (_, endpoint, _) = provider.settings();
    let agent: ureq::Agent = ureq::Agent::config_builder()
        .https_only(true)
        .max_redirects(0)
        .http_status_as_error(false)
        .proxy(None)
        .timeout_global(Some(Duration::from_secs(8)))
        .build()
        .into();
    let query = query.trim();
    let count = limit.to_string();
    let offset = usize::from(page) * limit;
    let request = match provider {
        MetadataProvider::Crossref => agent
            .get(endpoint)
            .query("query.bibliographic", query)
            .query("rows", &count)
            .query("offset", offset.to_string())
            .query(
                "select",
                "DOI,title,author,container-title,published,license,reference",
            ),
        MetadataProvider::OpenAlex => agent
            .get(endpoint)
            .query("search", query)
            .query("per_page", &count)
            .query("page", (page + 1).to_string())
            .query(
                "select",
                "id,doi,display_name,authorships,publication_year,primary_location,topics,referenced_works",
            ),
        MetadataProvider::Arxiv => agent
            .get(endpoint)
            .query("search_query", format!("all:{query}"))
            .query("start", offset.to_string())
            .query("max_results", &count),
    };
    let response = request
        .header(
            "User-Agent",
            "AETHER-Research-Laboratory/0.1 (explicit bibliographic lookup)",
        )
        .call();
    let mut response = response.map_err(|_| LiteratureError::Upstream)?;
    if response.status().as_u16() == 429 {
        let wait = retry_after_seconds(
            response
                .headers()
                .get("retry-after")
                .and_then(|value| value.to_str().ok()),
        );
        return Err(LiteratureError::RateLimited {
            retry_after_secs: wait,
        });
    }
    if !response.status().is_success() {
        return Err(LiteratureError::Upstream);
    }
    let bytes = response
        .body_mut()
        .with_config()
        .limit(MAX_RESPONSE_BYTES)
        .read_to_vec()
        .map_err(|_| LiteratureError::InvalidResponse)?;
    let body = std::str::from_utf8(&bytes).map_err(|_| LiteratureError::InvalidResponse)?;
    let timestamp = observed_now(CitationSource::from(provider)).observed_at_unix_seconds;
    decoded_page(provider, body, limit, timestamp, query, page)
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

#[cfg(test)]
pub(super) fn parse_metadata(
    provider: MetadataProvider,
    body: &str,
    limit: usize,
    observed_at_unix_seconds: u64,
) -> Result<Vec<Citation>, LiteratureError> {
    Ok(decoded_page(
        provider,
        body,
        limit,
        observed_at_unix_seconds,
        "fixture",
        0,
    )?
    .records)
}

pub(super) fn decoded_page(
    provider: MetadataProvider,
    body: &str,
    limit: usize,
    observed_at_unix_seconds: u64,
    query: &str,
    page: u16,
) -> Result<MetadataPage, LiteratureError> {
    if body.len() as u64 > MAX_RESPONSE_BYTES {
        return Err(LiteratureError::InvalidResponse);
    }
    let provenance = CitationProvenance {
        source: provider.into(),
        observed_at_unix_seconds,
    };
    let (records, received) = match provider {
        MetadataProvider::Crossref => {
            let envelope: CrossrefEnvelope = decode_json(body)?;
            let received = envelope.message.items.len();
            (crossref_citations(envelope, limit, provenance)?, received)
        }
        MetadataProvider::OpenAlex => {
            let envelope: OpenAlexEnvelope = decode_json(body)?;
            let received = envelope.results.len();
            (openalex_citations(envelope, limit, provenance)?, received)
        }
        MetadataProvider::Arxiv => arxiv_citations(body, limit, provenance)?,
    };
    Ok(MetadataPage {
        provider,
        endpoint: provider.settings().1.into(),
        query: query.into(),
        page,
        page_size: limit,
        response_sha256: aether_store::artifact_sha256(body.as_bytes()),
        observed_at_unix_seconds,
        received,
        records,
        cached: false,
        next_page: (received == limit && page + 1 < PAGE_LIMIT).then_some(page + 1),
        capped: received == limit && page + 1 == PAGE_LIMIT,
    })
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
    reference: Option<Vec<CrossrefReference>>,
}

#[derive(Deserialize)]
struct CrossrefReference {
    #[serde(rename = "DOI")]
    doi: Option<String>,
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
            let (references, references_truncated) =
                bounded_references(item.reference.map(|list| {
                    list.into_iter()
                        .map(|reference| reference.doi.and_then(|doi| normalized_doi(&doi)))
                }));
            Citation {
                doi: Some(item.doi),
                arxiv_id: None,
                openalex_id: None,
                related_doi: None,
                references,
                references_truncated,
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
    referenced_works: Option<Vec<String>>,
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
            let doi = item.doi.and_then(|doi| normalized_doi(&doi));
            let (references, references_truncated) =
                bounded_references(item.referenced_works.map(|list| {
                    list.into_iter()
                        .map(|id| paper_id(&id).filter(|id| id.starts_with("openalex:")))
                }));
            Citation {
                doi,
                arxiv_id: None,
                openalex_id: Some(id.to_owned()),
                related_doi: None,
                references,
                references_truncated,
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
) -> Result<(Vec<Citation>, usize), LiteratureError> {
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
    let received = entries.len();
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
    Ok((records, received))
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
        references: None,
        references_truncated: false,
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

fn bounded_references(
    ids: Option<impl Iterator<Item = Option<String>>>,
) -> (Option<Vec<String>>, bool) {
    let Some(ids) = ids else { return (None, false) };
    let mut incomplete = false;
    let mut saved = Vec::new();
    for id in ids {
        match id {
            Some(id) if saved.len() < 128 => saved.push(id),
            _ => incomplete = true,
        }
    }
    (Some(saved), incomplete)
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
