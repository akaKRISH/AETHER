# Literature discovery and local provenance

AETHER searches its local SQLite catalog without network access. A separate authenticated lookup accepts a user-supplied bibliographic query for one named provider. Lookup is disabled by default. The optional save operation imports the observed records into the local catalog. Provider titles, author names and categories remain external data throughout this path.

| Provider | Normalized discovery input | Enable flag | Pacing |
|---|---|---|---|
| Crossref | Public `/works` page with bibliographic metadata, reported rights URLs and reference DOIs | `AETHER_ALLOW_CROSSREF=1` | One request per local archive at a time and at least one second of cooldown |
| OpenAlex | Keyless `/works` page with DOI when reported, work ID, bibliographic metadata, topics and referenced works | `AETHER_ALLOW_OPENALEX=1` | One request per local archive at a time and at least three seconds of cooldown |
| arXiv | Atom result page with versioned e-print ID, bibliographic metadata, categories and linked journal DOI when reported | `AETHER_ALLOW_ARXIV=1` | One request per local archive at a time and at least three seconds of cooldown |

Each flag applies to the requesting server or CLI process. OpenAlex uses the documented small keyless query budget with no account key. Requests use fixed HTTPS origins, reject redirects and cap the response at 2 MiB. SQLite persists cooldowns across processes sharing the same archive. The API returns 429 with a `Retry-After` value for local cooldowns or upstream rate limits. Share the same data directory for provider clients using AETHER. arXiv's pacing applies across clients controlled by one user. [Crossref access and limits](https://www.crossref.org/documentation/retrieve-metadata/rest-api/access-and-authentication) · [OpenAlex authentication](https://help.openalex.org/api/authentication/) · [OpenAlex search](https://help.openalex.org/api/searching/) · [arXiv API](https://info.arxiv.org/help/api/user-manual.html) · [arXiv limits](https://info.arxiv.org/help/api/tou.html)

`POST /v1/literature/lookup` takes `{"query":"reproducible computational experiments","provider":"openalex","save":false,"page":0}` plus the local bearer token. If `provider` is omitted, the API uses Crossref. Providing a zero-based page returns the provenance envelope and continuation. Omitting the page returns the original first-page array. Cache entries do not depend on `save`. Set `save: true` to add snapshots and aliases. `GET /v1/literature?query=...` searches local metadata and extracted passages. `GET /v1/literature/work?id=...` shows versions and text extraction details.

## Catalog identity

An incoming DOI links to an existing work with the same normalized DOI. OpenAlex IDs and arXiv roots are alternate keys. The original arXiv version remains in the first saved record. A journal DOI reported by arXiv is a related publication and gets its own record identity. A provider record that would join two distinct saved works causes the import to roll back. The catalog preserves the first observed metadata and provenance, including the original version and observation time.

Lookup returns bibliographic records and known identifiers. The catalog derives landing-page links from validated identifiers. The [local text intake](../../research/literature/README.md) supports cited passages under an explicit rights manifest. Automated full-text retrieval and literature-to-hypothesis links remain open.

## Historical live Windows probe

An isolated release server enabled each provider and submitted only public bibliographic queries, without credentials. Crossref returned HTTP 200 with 11 normalized records from a bibliographic query. That route uses search ranking. OpenAlex returned HTTP 200 with 12 records including work `W2626778328`. arXiv returned HTTP 200 with 12 records including version `1706.03762v7`. The process saved results in its temporary catalog, and a local title search returned 20 entries, the configured page ceiling. The probe retrieved metadata. [OpenAlex work](https://openalex.org/W2626778328) · [arXiv record](https://arxiv.org/abs/1706.03762v7)

## Code walkthrough

1. `CitationCatalog::discover` validates the query, checks the persisted cache and reserves the provider before `fetch_page` reads a bounded response. The adapter normalizes the payload and stores its observation timestamp and response digest. arXiv parsing checks the Atom namespace and disables document type declarations and external XML entities. [Archive](../../crates/aether-api/src/literature/archive.rs) · [Adapters](../../crates/aether-api/src/literature/discovery.rs)
2. `Citation::normalized` checks identifier shape and field limits. `Citation::aliases` provides the small list of exact keys for the work. `CitationCatalog::import` writes metadata and aliases in one immediate transaction. If an alias is new, a lookup by that alias can find the earlier record. [Catalog](../../crates/aether-api/src/literature.rs)
3. The HTTP handler checks the server's provider flag before any external request. The local catalog remains available when every provider flag is absent. [Handler](../../crates/aether-api/src/http.rs)

Run `cargo test --locked -p aether-api literature::` to exercise fixtures for all three formats, catalog compatibility, identity conflicts, topic search, retry delays and malformed Atom documents. These fixtures use invented example identifiers inside the tests and create no live provider traffic.

### Rebuild exercise

Start with two rows that refer to the same paper. Give the first row a DOI and the second row both that DOI and a provider ID. Write a test showing that the first title remains after both rows are imported and that searching by the provider ID still finds it. Next add a second existing DOI and an incoming row whose identifiers refer to two different saved works. Explain why an atomic rollback is safer than choosing one snapshot.
