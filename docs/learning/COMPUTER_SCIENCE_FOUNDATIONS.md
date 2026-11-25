# Computer science foundations

| Concept | First-principles explanation | Exercise |
|---|---|---|
| Complexity | Count input-dependent work | Count comparisons for n labels and compare two-pass vs one-pass |
| State machine | Valid states and explicit authorized transitions | Draw three run states and reject a second terminal transition |
| Hash | Map bytes to a stable fixed-length digest | Explain why modifying a stored blob changes the recomputed digest |
| Transaction | Commit related operations as one unit | Sketch a crash between budget reservation and queue creation |
| Graph | Vertices and directed relations | Trace the effects of revoking a prerequisite claim |
| Process isolation | Boundary restricting execution, resources and visibility | Name three things a reviewed worker still must never read |

Apply ideas to current scope. Defer asynchronous Rust until reliable synchronous behavior exists.
