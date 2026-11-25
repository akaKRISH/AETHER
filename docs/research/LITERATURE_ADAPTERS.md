# Literature discovery and local provenance

AETHER searches its local SQLite catalog without network access. A separate authenticated lookup accepts a user-supplied bibliographic query for one named provider. Lookup is disabled by default. The optional save operation imports the observed records into the local catalog. Provider titles, author names and categories remain external data throughout this path.

| Provider | Normalized discovery input | Enable flag | Pacing |
|---|---|---|---|
| Crossref | First public `/works` page, with DOI, title, authors, year, journal and reported rights URLs | `AETHER_ALLOW_CROSSREF=1` | One request at a time and a one-second cooldown |
| OpenAlex | First keyless `/works` page, with DOI when reported, OpenAlex work ID, title, authors, year, venue and topics | `AETHER_ALLOW_OPENALEX=1` | One request at a time and a three-second cooldown |
| arXiv | First Atom result page, with versioned e-print ID, title, authors, publication year, categories and linked journal DOI when reported | `AETHER_ALLOW_ARXIV=1` | One request at a time and a three-second cooldown |

Each flag applies to the server process. OpenAlex uses the documented small keyless query budget with no account key. Requests use fixed HTTPS origins, reject redirects and cap the response at 2 MiB. The HTTP API returns 429 with a `Retry-After` value when a lookup slot is busy or the provider returns a rate limit. Cooldowns are local to the process. arXiv's pacing applies across clients controlled by one user. [Crossref access and limits](https://www.crossref.org/documentation/retrieve-metadata/rest-api/access-and-authentication) · [OpenAlex authentication](https://help.openalex.org/api/authentication/) · [OpenAlex search](https://help.openalex.org/api/searching/) · [arXiv API](https://info.arxiv.org/help/api/user-manual.html) · [arXiv limits](https://info.arxiv.org/help/api/tou.html)

`POST /v1/literature/lookup` takes `{"query":"reproducible computational experiments","provider":"openalex","save":false}` plus the local bearer token. Allowed providers are `crossref`, `openalex` and `arxiv`. If `provider` is omitted, the API uses Crossref. Saving requires `save: true`. `GET /v1/literature?query=...` searches the local index by title, authors, venue, topic or known identity.

## Catalog identity

An incoming DOI links to an existing work with the same normalized DOI. OpenAlex IDs and arXiv roots are alternate keys. The original arXiv version remains in the first saved record. A journal DOI reported by arXiv is a related publication and gets its own record identity. A provider record that would join two distinct saved works causes the import to roll back. The catalog preserves the first observed metadata and provenance, including the original version and observation time.

Lookup returns bibliographic records and known identifiers. The catalog derives landing-page links from validated identifiers. Reading papers and deciding what a citation supports are separate research steps. Literature-to-hypothesis links, full texts and live-provider checks remain open.

## Code walkthrough

1. `search_metadata` validates the query, rate-limits the selected provider and reads a bounded response. `parse_metadata` normalizes the provider payload with the observation timestamp. arXiv parsing checks the Atom namespace and disables document type declarations and external XML entities. [Adapters](../../crates/aether-api/src/literature/discovery.rs)
2. `Citation::normalized` checks identifier shape and field limits. `Citation::aliases` provides the small list of exact keys for the work. `CitationCatalog::import` writes metadata and aliases in one immediate transaction. If an alias is new, a lookup by that alias can find the earlier record. [Catalog](../../crates/aether-api/src/literature.rs)
3. The HTTP handler checks the server's provider flag before any external request. The local catalog remains available when every provider flag is absent. [Handler](../../crates/aether-api/src/http.rs)

Run `cargo test --locked -p aether-api literature::` to exercise fixtures for all three formats, catalog compatibility, identity conflicts, topic search, retry delays and malformed Atom documents. These fixtures use invented example identifiers inside the tests and create no live provider traffic.

### Rebuild exercise

Start with two rows that refer to the same paper. Give the first row a DOI and the second row both that DOI and a provider ID. Write a test showing that the first title remains after both rows are imported and that searching by the provider ID still finds it. Next add a second existing DOI and an incoming row whose identifiers refer to two different saved works. Explain why an atomic rollback is safer than choosing one snapshot.
