use serde_json::{Value, json};

use super::*;

fn apply(catalog: &mut CitationCatalog, batch: Value) -> Result<Value, LiteratureError> {
    catalog.graph_apply(serde_json::from_value(batch)?)
}

fn claim(key: &str) -> Value {
    json!({ "kind": "claim", "key": key, "label": key, "statement": format!("{key} reports an outcome") })
}

fn connect(origin: &str, target: &str, relation: &str) -> Value {
    json!({
        "origin": {"by": "local", "kind": "claim", "key": origin},
        "target": {"by": "local", "kind": "claim", "key": target},
        "relation": relation, "confidence_bps": 5100
    })
}

#[test]
fn dependency_cycle_and_invalid_endpoint_roll_back_the_batch() {
    let mut catalog = CitationCatalog::open(":memory:").unwrap();
    let batch = apply(
        &mut catalog,
        json!({
            "nodes": [claim("A"), claim("B"), claim("C")],
            "edges": [connect("A", "B", "depends_on"), connect("B", "C", "depends_on")]
        }),
    )
    .unwrap();
    let id = batch["node_ids"][0].as_str().unwrap();
    let before = catalog.graph_export(id, 3).unwrap().graph_sha256;
    for (link, expected) in [
        (connect("C", "A", "depends_on"), "claim dependency cycle"),
        (connect("A", "B", "cites"), "relationship cannot connect"),
        (connect("A", "missing", "supports"), "node not found"),
    ] {
        let failed = apply(
            &mut catalog,
            json!({
                "nodes": [claim("Transient")],
                "edges": [connect("Transient", "B", "supports"), link]
            }),
        )
        .unwrap_err();
        assert!(failed.to_string().contains(expected));
        assert_eq!(
            catalog.graph_search("Transient", None).unwrap()["nodes"],
            json!([])
        );
        assert_eq!(catalog.graph_export(id, 3).unwrap().graph_sha256, before);
    }
}

#[test]
fn revisions_and_conflicting_relationships_are_preserved_without_duplicate_rows() {
    let mut catalog = CitationCatalog::open(":memory:").unwrap();
    let initial = json!({
        "nodes": [claim("finding"), claim("observation")],
        "edges": [connect("observation", "finding", "supports"),
            connect("observation", "finding", "contradicts")]
    });
    let result = apply(&mut catalog, initial.clone()).unwrap();
    assert_eq!(apply(&mut catalog, initial).unwrap(), result);
    let id = result["node_ids"][0].as_str().unwrap();
    let expanded = catalog.graph_export(id, 3).unwrap();
    assert_eq!(expanded.graph.nodes.len(), 2);
    assert_eq!(expanded.graph.edges.len(), 2);
    assert_eq!(expanded.graph.root_history.len(), 1);
    for edge in expanded.graph.edges {
        assert_eq!(edge.status, Status::Inferred);
        assert_eq!(edge.confidence_bps, Some(5100));
    }
    let mut changed = claim("finding");
    changed["label"] = json!("Revised finding");
    changed["statement"] = json!("A later negative finding");
    assert_eq!(
        apply(&mut catalog, json!({"nodes": [changed]})).unwrap()["node_ids"][0],
        id
    );
    let updated = catalog.graph_export(id, 3).unwrap();
    assert_eq!(updated.graph.root_history.len(), 2);
    assert_eq!(
        updated.graph.root_history[1].observation.label,
        "Revised finding"
    );
    assert_eq!(updated.graph.edges.len(), 2);
    assert_eq!(
        updated.graph.edges[0].target_revision,
        updated.graph.root_history[0].sha256
    );
    let peer = result["node_ids"][1].as_str().unwrap();
    assert_eq!(
        catalog.graph_export(peer, 1).unwrap().graph.linked_history[0].node,
        id
    );
    assert_eq!(
        updated.graph_sha256,
        catalog.graph_export(id, 3).unwrap().graph_sha256
    );
    assert_ne!(updated.graph_sha256, expanded.graph_sha256);
    let mut forged = claim("forged");
    forged["status"] = json!("independently_verified");
    assert!(apply(&mut catalog, json!({"nodes": [forged]})).is_err());
}

#[test]
fn real_alias_resolution_keeps_citation_cycles_finite() {
    let mut catalog = CitationCatalog::open(":memory:").unwrap();
    let first = json!({
        "doi": "https://doi.org/10.9999/graph-a", "title": "Graph A",
        "references": ["https://openalex.org/W22"]
    });
    let second = json!({
        "doi": "10.9999/graph-b", "openalex_id": "W22", "title": "Graph B",
        "references": ["10.9999/GRAPH-A"]
    });
    let a = node_id(Kind::Paper, "catalog:10.9999/graph-a");
    catalog
        .import_manual(serde_json::from_value(json!([first.clone()])).unwrap())
        .unwrap();
    let pending = catalog.graph_export(&a, 3).unwrap();
    assert_eq!(
        pending.graph.unresolved_citations[0].target_identifier,
        "openalex:W22"
    );
    catalog
        .import_manual(serde_json::from_value(json!([second, first])).unwrap())
        .unwrap();
    let connected = catalog.graph_export(&a, 3).unwrap();
    assert!(connected.graph.unresolved_citations.is_empty());
    assert_eq!(connected.graph.nodes.len(), 2);
    assert_eq!(connected.graph.edges.len(), 2);
    assert!(
        connected
            .graph
            .edges
            .iter()
            .all(|edge| edge.relation == Relation::Cites && edge.status == Status::Reported)
    );
    assert_eq!(connected.graph.root_history.len(), 1);
}

#[test]
fn a_dense_result_reports_truncation_and_bounded_depth() {
    let mut catalog = CitationCatalog::open(":memory:").unwrap();
    let peers: Vec<_> = (0..64).map(|index| format!("peer-{index}")).collect();
    apply(
        &mut catalog,
        json!({"nodes": peers.iter().map(|key| claim(key)).collect::<Vec<_>>() }),
    )
    .unwrap();
    let result = apply(
        &mut catalog,
        json!({
            "nodes": [claim("root")],
            "edges": peers.iter().map(|key| connect(key, "root", "supports")).collect::<Vec<_>>()
        }),
    )
    .unwrap();
    let root = result["node_ids"][0].as_str().unwrap();
    let single = catalog.graph_export(root, 0).unwrap();
    assert_eq!(single.graph.nodes.len(), 1);
    assert!(single.graph.edges.is_empty());
    let bounded = catalog.graph_export(root, 1).unwrap();
    assert_eq!(bounded.graph.nodes.len(), 64);
    assert_eq!(bounded.graph.edges.len(), 63);
    assert!(bounded.graph.truncated);
    assert!(catalog.graph_export(root, 4).is_err());
}
