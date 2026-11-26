CREATE TABLE IF NOT EXISTS knowledge_node (
    id TEXT PRIMARY KEY,
    kind TEXT NOT NULL,
    identity TEXT NOT NULL,
    label TEXT NOT NULL COLLATE NOCASE,
    UNIQUE(kind, identity)
) STRICT;
CREATE INDEX IF NOT EXISTS knowledge_label ON knowledge_node(label, id);
CREATE INDEX IF NOT EXISTS knowledge_kind_label ON knowledge_node(kind, label, id);
CREATE TABLE IF NOT EXISTS knowledge_revision (
    sequence INTEGER PRIMARY KEY,
    node TEXT NOT NULL REFERENCES knowledge_node(id),
    digest TEXT NOT NULL,
    record TEXT NOT NULL,
    source_group TEXT NOT NULL,
    content_sha256 TEXT,
    UNIQUE(node, digest)
) STRICT;
CREATE INDEX IF NOT EXISTS knowledge_history ON knowledge_revision(node, sequence);
CREATE INDEX IF NOT EXISTS knowledge_source ON knowledge_revision(source_group, node);
CREATE INDEX IF NOT EXISTS knowledge_duplicate ON knowledge_revision(content_sha256, node);
CREATE TABLE IF NOT EXISTS knowledge_edge (
    id TEXT PRIMARY KEY,
    origin TEXT NOT NULL REFERENCES knowledge_node(id),
    relation TEXT NOT NULL,
    target TEXT NOT NULL REFERENCES knowledge_node(id),
    origin_revision TEXT NOT NULL,
    target_revision TEXT NOT NULL,
    record TEXT NOT NULL,
    FOREIGN KEY(origin, origin_revision) REFERENCES knowledge_revision(node, digest),
    FOREIGN KEY(target, target_revision) REFERENCES knowledge_revision(node, digest),
    CHECK(origin <> target)
) STRICT;
CREATE INDEX IF NOT EXISTS knowledge_outbound ON knowledge_edge(origin, relation, target, id);
CREATE INDEX IF NOT EXISTS knowledge_inbound ON knowledge_edge(target, relation, origin, id);
CREATE TABLE IF NOT EXISTS knowledge_citation (
    id TEXT PRIMARY KEY,
    origin TEXT NOT NULL REFERENCES knowledge_node(id),
    alias TEXT NOT NULL,
    relation TEXT NOT NULL,
    record TEXT NOT NULL,
    resolved TEXT REFERENCES knowledge_node(id)
) STRICT;
CREATE INDEX IF NOT EXISTS knowledge_pending ON knowledge_citation(alias) WHERE resolved IS NULL;
CREATE INDEX IF NOT EXISTS knowledge_paper_citations ON knowledge_citation(origin, id);
