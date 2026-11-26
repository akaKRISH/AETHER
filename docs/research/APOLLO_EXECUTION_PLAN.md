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

1. Review the starter corpus and expand it with authorized full-text copies. Label conversion methods and record version-specific rights before importing a new format.
2. Evaluate section extraction on a reviewed corpus. Record missing sections, disagreement and unsupported formats before considering model assistance.
3. Qualify optional extraction against adversarial documents and exact span tests. Any future model output is advisory and requires a separate review.

Discovery queries send only explicitly supplied bibliographic terms to the enabled provider. Provider 429s and outages leave the requested page resumable. Shallow offset pagination can shift when an upstream index changes. Extraction coverage and scientific verification remain separate outcomes.
