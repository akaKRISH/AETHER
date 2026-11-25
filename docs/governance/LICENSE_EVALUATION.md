# License evaluation

Apache-2.0 is the v0.2 candidate for first-party software. Its explicit contributor patent grant and termination provisions are relevant to commercial adoption and open collaboration. Its redistribution requirements include preserving applicable notices, giving a copy of the license and marking changed files, plus retaining provided NOTICE attribution when applicable. [Apache-2.0 text](https://www.apache.org/licenses/LICENSE-2.0)

| Alternative | Trade-off | Source |
| --- | --- | --- |
| MIT | Short permissive terms requiring preservation of the copyright and permission notice. It has no express patent grant in the text. | [OSI MIT text](https://opensource.org/license/mit) |
| BSD-3-Clause | Short permissive terms with attribution and a non-endorsement clause. It has no express patent grant in the text. | [OSI BSD-3-Clause text](https://opensource.org/license/bsd-3-clause) |
| Defer publication | Provides time to inventory rights and decide how to license software, docs, schemas and datasets. External contributors need terms resolved before their work is accepted. | [Project license decision](../../LICENSE_DECISION.md) |

Before licensing or distributing, verify copyright and provenance by asset class. Catalogue each component's version, origin, rights holder if known, SPDX expression or exact terms, whether redistribution is allowed and required notices. Review original design archives, third-party examples, fixtures, benchmark data, model weights, generated reports and evaluation bundles individually. They may require separate terms or exclusion from a release. Pin references and record checksums for externally obtained assets. Require the owner to approve the terms and release scope before adding a final license. [ADR-008](../decisions/ADR-008.md)
