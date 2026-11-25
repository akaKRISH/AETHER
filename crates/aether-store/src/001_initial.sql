CREATE TABLE campaign (
    id TEXT PRIMARY KEY,
    title TEXT NOT NULL,
    objective TEXT NOT NULL,
    protocol_hash TEXT NOT NULL,
    policy_hash TEXT NOT NULL,
    evaluator_hash TEXT NOT NULL,
    requires_replay INTEGER NOT NULL CHECK (requires_replay IN (0, 1)),
    state TEXT NOT NULL CHECK (state IN ('draft', 'approved', 'running', 'paused', 'completed', 'failed', 'cancelled'))
) STRICT;
CREATE TABLE budget (
    campaign_id TEXT PRIMARY KEY REFERENCES campaign(id),
    caps TEXT NOT NULL CHECK (json_valid(caps)),
    used TEXT NOT NULL CHECK (json_valid(used)),
    reserved TEXT NOT NULL CHECK (json_valid(reserved)),
    halted INTEGER NOT NULL DEFAULT 0 CHECK (halted IN (0, 1))
) STRICT;
CREATE TABLE hypothesis (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaign(id),
    statement TEXT NOT NULL
) STRICT;
CREATE TABLE experiment (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaign(id),
    hypothesis_id TEXT REFERENCES hypothesis(id),
    task_key TEXT NOT NULL,
    definition_hash TEXT NOT NULL
) STRICT;
CREATE TABLE run (
    id TEXT PRIMARY KEY,
    experiment_id TEXT NOT NULL REFERENCES experiment(id),
    parent_run_id TEXT REFERENCES run(id),
    action TEXT NOT NULL CHECK (action IN ('explore', 'replay', 'challenge')),
    seed INTEGER NOT NULL CHECK (seed >= 0),
    input_hash TEXT NOT NULL
) STRICT;
CREATE TABLE blob (
    sha256 TEXT PRIMARY KEY,
    content BLOB NOT NULL
) STRICT;
CREATE TABLE attempt (
    id TEXT PRIMARY KEY,
    run_id TEXT NOT NULL REFERENCES run(id),
    lease_token TEXT NOT NULL UNIQUE,
    lease_expires_at_ms INTEGER NOT NULL CHECK (lease_expires_at_ms > 0),
    estimate TEXT NOT NULL CHECK (json_valid(estimate)),
    outcome TEXT CHECK (outcome IN ('succeeded', 'failed', 'cancelled', 'expired')),
    usage TEXT CHECK (json_valid(usage)),
    manifest_sha256 TEXT REFERENCES blob(sha256),
    reason TEXT,
    completion_sha256 TEXT,
    CHECK ((outcome IS NULL) = (usage IS NULL) AND (outcome IS NULL) = (completion_sha256 IS NULL))
) STRICT;
CREATE UNIQUE INDEX attempt_active_run ON attempt(run_id) WHERE outcome IS NULL;
CREATE INDEX attempt_run_id ON attempt(run_id, id);
CREATE INDEX attempt_expiries ON attempt(lease_expires_at_ms, id) WHERE outcome IS NULL;
CREATE TABLE attempt_artifact (
    attempt_id TEXT NOT NULL REFERENCES attempt(id),
    name TEXT NOT NULL,
    media_type TEXT NOT NULL,
    sha256 TEXT NOT NULL REFERENCES blob(sha256),
    PRIMARY KEY (attempt_id, name)
) STRICT;
CREATE INDEX attempt_artifact_sha256 ON attempt_artifact(attempt_id, sha256);
CREATE TABLE metric (
    id TEXT PRIMARY KEY,
    attempt_id TEXT NOT NULL REFERENCES attempt(id),
    name TEXT NOT NULL,
    unit TEXT NOT NULL,
    evaluator_hash TEXT NOT NULL,
    split_hash TEXT NOT NULL,
    value REAL NOT NULL,
    UNIQUE (attempt_id, name, split_hash)
) STRICT;
CREATE TABLE claim (
    id TEXT PRIMARY KEY,
    campaign_id TEXT NOT NULL REFERENCES campaign(id),
    statement TEXT NOT NULL,
    state TEXT NOT NULL CHECK (state IN ('proposed', 'supported', 'verified', 'rejected', 'inconclusive', 'revoked'))
) STRICT;
CREATE TABLE claim_dependency (
    claim_id TEXT NOT NULL REFERENCES claim(id),
    prerequisite_id TEXT NOT NULL REFERENCES claim(id),
    CHECK (claim_id != prerequisite_id),
    PRIMARY KEY (claim_id, prerequisite_id)
) STRICT;
CREATE INDEX prerequisite_dependents ON claim_dependency(prerequisite_id, claim_id);
CREATE TABLE evidence (
    id TEXT PRIMARY KEY,
    claim_id TEXT NOT NULL REFERENCES claim(id),
    metric_id TEXT REFERENCES metric(id),
    artifact_attempt_id TEXT REFERENCES attempt(id),
    artifact_sha256 TEXT REFERENCES blob(sha256),
    relation TEXT NOT NULL CHECK (relation IN ('supports', 'contradicts', 'derived_from', 'reproduces', 'invalidates')),
    invalidated INTEGER NOT NULL DEFAULT 0 CHECK (invalidated IN (0, 1)),
    CHECK ((metric_id IS NOT NULL AND artifact_attempt_id IS NULL AND artifact_sha256 IS NULL)
        OR (metric_id IS NULL AND artifact_attempt_id IS NOT NULL AND artifact_sha256 IS NOT NULL))
) STRICT;
CREATE INDEX evidence_claim_id ON evidence(claim_id, id);
CREATE UNIQUE INDEX evidence_metric_identity
    ON evidence(claim_id, metric_id, relation) WHERE metric_id IS NOT NULL;
CREATE UNIQUE INDEX evidence_artifact_identity
    ON evidence(claim_id, artifact_attempt_id, artifact_sha256, relation) WHERE artifact_sha256 IS NOT NULL;
CREATE TABLE replication (
    id TEXT PRIMARY KEY,
    original_attempt_id TEXT NOT NULL REFERENCES attempt(id),
    replay_attempt_id TEXT NOT NULL UNIQUE REFERENCES attempt(id),
    passed INTEGER NOT NULL CHECK (passed IN (0, 1)),
    reason TEXT NOT NULL,
    CHECK (original_attempt_id != replay_attempt_id)
) STRICT;
CREATE TABLE verification (
    id TEXT PRIMARY KEY,
    claim_id TEXT NOT NULL REFERENCES claim(id),
    verdict TEXT NOT NULL CHECK (verdict IN ('verified', 'rejected', 'inconclusive')),
    rule_hash TEXT NOT NULL,
    evaluator_hash TEXT NOT NULL,
    replay_attempt_id TEXT REFERENCES attempt(id),
    reason TEXT NOT NULL
) STRICT;
CREATE INDEX verification_claim_id ON verification(claim_id, id);
CREATE TABLE contradiction (
    id TEXT PRIMARY KEY,
    claim_id TEXT NOT NULL REFERENCES claim(id),
    evidence_id TEXT NOT NULL REFERENCES evidence(id),
    reason TEXT NOT NULL,
    invalidates INTEGER NOT NULL CHECK (invalidates IN (0, 1))
) STRICT;
CREATE TABLE event (
    campaign_id TEXT NOT NULL REFERENCES campaign(id),
    sequence INTEGER NOT NULL CHECK (sequence > 0),
    kind TEXT NOT NULL,
    occurred_at_ms INTEGER NOT NULL CHECK (occurred_at_ms >= 0),
    detail TEXT NOT NULL CHECK (json_valid(detail)),
    PRIMARY KEY (campaign_id, sequence)
) STRICT;
CREATE INDEX hypothesis_campaign_id ON hypothesis(campaign_id, id);
CREATE INDEX experiment_campaign_id ON experiment(campaign_id, id);
CREATE INDEX run_experiment_id ON run(experiment_id, id);
CREATE INDEX claim_campaign_id ON claim(campaign_id, id);

CREATE TRIGGER event_immutable_update BEFORE UPDATE ON event
BEGIN SELECT RAISE(ABORT, 'audit event is immutable'); END;
CREATE TRIGGER event_immutable_delete BEFORE DELETE ON event
BEGIN SELECT RAISE(ABORT, 'audit event is immutable'); END;
CREATE TRIGGER verification_immutable_update BEFORE UPDATE ON verification
BEGIN SELECT RAISE(ABORT, 'verification record is immutable'); END;
CREATE TRIGGER verification_immutable_delete BEFORE DELETE ON verification
BEGIN SELECT RAISE(ABORT, 'verification record is immutable'); END;
CREATE TRIGGER blob_immutable_update BEFORE UPDATE ON blob
BEGIN SELECT RAISE(ABORT, 'evidence bytes are immutable'); END;
CREATE TRIGGER attempt_immutable_update BEFORE UPDATE ON attempt WHEN OLD.outcome IS NOT NULL
BEGIN SELECT RAISE(ABORT, 'completed attempt is immutable'); END;
