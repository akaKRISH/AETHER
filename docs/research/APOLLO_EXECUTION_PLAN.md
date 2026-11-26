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

### Pending literature work

1. Review the starter corpus and expand it with authorized full-text copies. Label conversion methods and record version-specific rights before importing a new format.
2. Evaluate section extraction on a reviewed corpus. Record missing sections, disagreement and unsupported formats before considering model assistance.
3. Qualify optional extraction against adversarial documents and exact span tests. Any future model output is advisory and requires a separate review.
4. Qualify live Crossref, OpenAlex and arXiv calls independently of deterministic fixtures. PDF ingestion requires a separate parser and source-span qualification.

Discovery queries send only explicitly supplied bibliographic terms to the enabled provider. Provider 429s and outages leave the requested page resumable. Shallow offset pagination can shift when an upstream index changes. Extraction coverage and scientific verification remain separate outcomes.
