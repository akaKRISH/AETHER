# Operation APOLLO execution plan

## APO-01 literature archive

Keep provider discovery and archive ingestion in Rust. The existing SQLite catalog retains the first citation snapshot for campaign planning. The archive adds immutable metadata revisions, source-reported outgoing identifiers, and local text passages. The authority boundary and limits are recorded in [ADR-014](../decisions/ADR-014.md).

| Gate | Evidence |
|---|---|
| Provider parsing | Parse bounded Crossref JSON, OpenAlex JSON and arXiv Atom fixtures. Preserve missing references, incomplete identifier coverage and related publication DOI. |
| Continuation | Reopen SQLite after page zero, reuse its response digest, honor a persisted cooldown and resume the next page. |
| Provenance | Resolve aliases, preserve explicit revisions, round-trip source bytes and check exact UTF-8 offsets. |
| Authority | Deny missing rights confirmation and anonymous API inspection. Retain instruction-like content as quoted text and create no campaigns or verdicts. |
| Operator workflow | Import the starter corpus and inspect an archive record through the CLI and Research OS. |

## Next slices

### APO-02 knowledge graph

New literature imports populate the graph transactionally. Run `aether-lab graph-paper <paper-id>` to backfill saved literature, or `aether-lab graph-campaign <id>` to capture a local campaign snapshot. To record a paper's reported claim, use `graph-report <paper-id> <document-id> <start-byte> <end-byte>` with the half-open byte range printed by `literature-ingest`. The source must already be archived and the range must be valid UTF-8. Run `graph-export <node-id> 2` for a reproducible view. The authenticated Knowledge Graph view supports title prefix or exact ID search and relationship, citation, history and source-group navigation.

To enter a local proposal, save this example as a JSON file and run `aether-lab graph-apply <batch.json>`. Use a local key consistently when you intend to revise the same entity. The returned stable IDs can also be used as endpoints with `{"by":"id","id":"…"}`.

```json
{
  "nodes": [
    { "kind": "claim", "key": "claim-a", "label": "Hypothetical reported difference" },
    { "kind": "evidence", "key": "evidence-b", "label": "Hypothetical counter-observation" }
  ],
  "edges": [{
    "origin": { "by": "local", "kind": "evidence", "key": "evidence-b" },
    "target": { "by": "local", "kind": "claim", "key": "claim-a" },
    "relation": "contradicts",
    "confidence_bps": 5000
  }]
}
```

The [graph decision](../decisions/ADR-015.md) describes projection, versioning and limits. The [two reviewed excerpts](../../research/literature/README.md#apo-02-reviewed-excerpts) test source grounding separately from synthetic contradictory and adversarial cases.

### APO-03 hypothesis discovery

Run `aether-lab hypothesis-index` to index all 20 matched cases from the pinned DAE-04 archive at eight candidate evaluations per arm. Use the returned graph root with `aether-lab hypothesis-discover <node-id>`. Alternatively, search for **DAE-04 matched** in the Research OS Knowledge Graph and select **Discover hypotheses**. The same read path works from a literature, claim, method, or experiment node. Inspect the exact source references and scope before evaluating a proposal.

The annotated comparison path accepts a local `metric` whose statement is a JSON object with numeric `treatment_gap` and `control_gap`, where smaller is better, and a `scope` object. The scope includes `task`, `case_sha256`, `profile`, `size`, `metric`, `treatment`, `control` and `budget`. Link the metric to a grounded claim or evidence revision with `derived_from`. The declared scope is an annotation and must be checked against the original. For transfer candidates, a dataset can declare a domain with a statement like `{"domain":"scheduling"}`. A shared grounded method and explicit dataset links are required on both sides.

Synthetic fixtures cover false conflicts, copied evidence, unsupported claims, misleading associations, proposal deduplication and incomplete local coverage. DAE-04 provides the separate measured development cases. Any proposed test needs a reviewed protocol, resource decision and the existing approval workflow. [Ranking and limits](../decisions/ADR-016.md)

### APO-04 experimental design

Run `aether-lab hypothesis-index` and pass its `root` to `aether-lab design-experiments <node-id>`. To check a saved draft, extract one element of `designs` into a UTF-8 JSON file and run `aether-lab design-validate <draft.json>`. The check requires the same local archive-backed graph slice. The Research OS shows **Review experiment designs** after the archived hypotheses are loaded in the Knowledge Graph. It displays the same three drafts over an authenticated read-only route.

The negative-result design has 20 matched cases and two arms. The boundary design crosses 12 seed blocks with two difficulty profiles and compares the same arms. The method-composition design has 12 cases and four factorial cells, including one missing cell. Each arm has eight candidate evaluations per case. The plans call for 320, 384 and 384 candidate evaluations respectively, with separately reserved scoring and oracle work. Sample sizes are exploratory. The drafts contain no power estimate or private inputs. See the [design and approval decision](../decisions/ADR-017.md).

### Pending literature work

1. Review the starter corpus and expand it with authorized full-text copies. Label conversion methods and record version-specific rights before importing a new format.
2. Evaluate section extraction on a reviewed corpus. Record missing sections, disagreement and unsupported formats before considering model assistance.
3. Qualify optional extraction against adversarial documents and exact span tests. Any future model output is advisory and requires a separate review.
4. Qualify live Crossref, OpenAlex and arXiv calls independently of deterministic fixtures. PDF ingestion requires a separate parser and source-span qualification.

Discovery queries send only explicitly supplied bibliographic terms to the enabled provider. Provider 429s and outages leave the requested page resumable. Shallow offset pagination can shift when an upstream index changes. Extraction coverage and scientific verification remain separate outcomes.
