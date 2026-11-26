# Literature records

The [review and query log](../../docs/research/LITERATURE_REVIEW.md) covers Phase Zero. Store later screening decisions with query, date, canonical work ID, version, source type, inclusion reason and reviewed method evidence. Deduplicate a preprint and published version while retaining changes that affect the comparison.

## APOLLO starter corpus

The [importable metadata](apollo-corpus.json) records three papers relevant to reproducibility and experiment verification. The titles and versions were checked against the source pages. This corpus contains bibliographic metadata only.

- [Improving Reproducibility in Machine Learning Research, arXiv v4](https://arxiv.org/abs/2003.12206v4)
- [REFORMS, arXiv v2](https://arxiv.org/abs/2308.07832v2)
- [PaperBench, arXiv v3](https://arxiv.org/abs/2504.01848v3)

With the local `aether-lab` executable, use `import-citations research/literature/apollo-corpus.json`, then `bibliography reproducibility` or `literature-work arxiv:2003.12206v4`.

For discovery, start `literature-discover crossref "machine learning reproducibility" 0` with the existing Crossref opt-in set. OpenAlex and arXiv use their own opt-ins. Under the authenticated API, send an explicit `page` to `POST /v1/literature/lookup` for the response envelope with `next_page`. Repeat the same provider, query and page to reuse a recent result. Set `save: true` to import results. The cache retains up to 256 pages and serves observations up to one hour old. Each query stops after 50 pages of 12 records. A full last page sets `capped`. A short page ends traversal. Results can shift after cache expiry as providers update their indexes.

To import locally acquired, rights-cleared UTF-8 full text, run `literature-ingest manifest.json source.txt`. The manifest takes `paper_id`, `source_url`, `license` and `rights_confirmed`. Enter the exact source version, ensure the license permits copying the exact input, and use an allowed SPDX value (`CC-BY-4.0` or `CC0-1.0`). PDF, archived content and arbitrary remote links have no import route. Inspect archived bytes via `literature-text <paper-id> <document-id>`.

The [synthetic fixture](../../crates/aether-api/tests/fixtures/apollo-extraction.txt) tests extraction without attributing invented results to these papers. The extractor recognizes explicit headings and saves literal question, method, claim, result and limitation paragraphs with half-open UTF-8 byte ranges. Missing sections and truncated output are explicit. The document hash covers the supplied UTF-8 file. For converted source text the offsets refer to the conversion, not the original PDF or layout.

## APO-02 reviewed excerpts

The [two-paper fixture](apo02-reviewed-excerpts.json) contains one verbatim sentence per article. The test prepends an editor-supplied heading to exercise extraction. It records offsets into that UTF-8 fixture. This is a small passage-grounding check using separately attributed real sources. Synthetic papers supply the cycle, conflict and duplicate cases.

- Brian M. Schilder, Alan E. Murphy and Nathan G. Skene (2024), © Crown 2024. One excerpt from the introduction was copied unchanged and grouped under a fixture heading. [Article and rights](https://doi.org/10.1038/s41467-023-44484-5)
- Markus Konkol, Daniel Nüst and Laura Goulier (2020), © The Author(s) 2020. One excerpt from the discussion was copied unchanged and grouped under a fixture heading. [Article and rights](https://doi.org/10.1186/s41073-020-00095-y)

Both excerpts are used under [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/). They are excerpts from two articles, with no automated full-text or PDF retrieval.
