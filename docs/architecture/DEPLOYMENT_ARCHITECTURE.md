# Deployment architecture

| Stage | Topology | Boundary |
|---|---|---|
| Genesis | Local docs and Git | Owner design review |
| Foundation | Native Rust CLI and built-in fixture on supported local host | Local manifests and reports |
| Evidence and reviewed execution | Local SQLite, artifacts, qualified Linux or WSL2/Linux runner | No public listener |
| Research release | Controlled local lab, optional approved model adapter | Reproducibility bundle after data/license review |
| R2+ | Authenticated Rust API, optional remote qualified worker | Explicit deployment approval, HTTPS and project scoping |

Windows native hosts the trusted fixture. WSL2 presence, rootless containers and local process execution need qualification against filesystem access, process-tree termination, resource controls and egress before reviewed worker admission. Dedicated hardened isolation and threat review are prerequisites for any expanded execution exposure.

Keep provider keys, hidden labels, control storage and system credentials outside worker scope. Portable exports contain manifests and permitted data or pointers. Retest migration, restoration and restrictions when topology changes. See [deployment diagram](../diagrams/deployment.mmd) and [source operations](../archive/v0.2/AETHER/16_DEVOPS.md).
