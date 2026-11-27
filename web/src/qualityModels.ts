export type QualityAreaId = 'benchmarks' | 'overview' | 'diagnostics' | 'integrity'

export type QualityDimensionId =
  | 'literature_scope_inventory_integrity'
  | 'claim_source_locator_integrity'
  | 'evidence_provenance_field_integrity'
  | 'counterevidence_edge_integrity'
  | 'hypothesis_structure_completeness'
  | 'experiment_design_arithmetic_integrity'
  | 'uncertainty_field_wellformedness'
  | 'artifact_snapshot_replay_integrity'

export type StructuralOutcome = 'pass' | 'fail' | 'unavailable'
export type ExecutionOutcome = 'completed' | 'partial' | 'abstained' | 'failed'
export type EvidenceIntegrityOutcome = 'valid' | 'invalid' | 'unavailable'
export type EvidencePolarity = 'supporting' | 'contradictory' | 'contextual'
export type SourceReplayStatus = 'available' | 'unavailable'
export type EmptyCollectionPolicy = 'fail_required' | 'unavailable_optional' | 'pass_only_if_explicitly_permitted'
export type SubjectKind = 'hypothesis' | 'experiment_design'
export type ArtifactKind = 'hypothesis_report' | 'experiment_design_report'
export type EpistemicStatus =
  | 'proposed'
  | 'reported'
  | 'extracted'
  | 'inferred'
  | 'observed'
  | 'independently_verified'
  | 'unknown'
export type SourceIdentityStatus = 'verified' | 'unverified' | 'not_applicable'
export type GraphKind =
  | 'paper'
  | 'claim'
  | 'method'
  | 'dataset'
  | 'metric'
  | 'experiment'
  | 'model'
  | 'evidence'
  | 'limitation'
  | 'hypothesis'
  | 'research_question'
  | 'campaign'
export type EvidenceRelation = 'supports' | 'contradicts' | 'other'

export const QUALITY_LIMITS = {
  requestBytes: 8192,
  graphDepthMax: 3,
  graphNodesMax: 64,
  graphEdgesMax: 192,
  subjectsMax: 8,
  artifactBytesMax: 2_097_152,
  snapshotBytesMax: 4_194_304,
  reportBytesMax: 1_048_576,
  reportListBytesMax: 262_144,
  pageSize: 32,
} as const

export const DEFAULT_REPORT_PAGE_SIZE = QUALITY_LIMITS.pageSize
export const DEFAULT_CHECK_PAGE_SIZE = 50

export interface QualityDimensionDescriptor {
  readonly id: QualityDimensionId
  readonly title: string
  readonly structuralScope: string
  readonly cannotEstablish: string
  readonly defaultPolicy: EmptyCollectionPolicy
}

export const QUALITY_AREAS: readonly {
  readonly id: QualityAreaId
  readonly label: string
  readonly summary: string
}[] = [
  {
    id: 'benchmarks',
    label: 'Benchmarks & Run Capture',
    summary: 'Inspect pinned structural rules, admission bounds, graph/subject inputs, and capture immutable snapshots.',
  },
  {
    id: 'overview',
    label: 'Runs & Report Overview',
    summary: 'Browse newest-first evaluation history, execution status, all eight structural dimensions, and limitations.',
  },
  {
    id: 'diagnostics',
    label: 'Dimension & Evidence Diagnostics',
    summary: 'Inspect checks across all eight dimensions, evidence origin/identity fields, and contradiction edges.',
  },
  {
    id: 'integrity',
    label: 'Integrity, Snapshot & Audit',
    summary: 'Inspect digests, evaluator identity limits, snapshot pins, raw-byte Web Crypto checks, and safe export.',
  },
]

export const QUALITY_DIMENSIONS: readonly QualityDimensionDescriptor[] = [
  {
    id: 'literature_scope_inventory_integrity',
    title: 'Literature scope & inventory integrity',
    structuralScope: 'Checks for a nonempty, untruncated paper inventory.',
    cannotEstablish: 'Literature coverage, relevance, source quality, or truth.',
    defaultPolicy: 'fail_required',
  },
  {
    id: 'claim_source_locator_integrity',
    title: 'Claim source locator integrity',
    structuralScope: 'Checks typed evidence links and source locator fields.',
    cannotEstablish: 'Semantic entailment or whether the passage supports a claim.',
    defaultPolicy: 'unavailable_optional',
  },
  {
    id: 'evidence_provenance_field_integrity',
    title: 'Evidence provenance field integrity',
    structuralScope: 'Checks typed origin, source identity state, and epistemic state fields.',
    cannotEstablish: 'Real-world source identity or epistemic correctness.',
    defaultPolicy: 'unavailable_optional',
  },
  {
    id: 'counterevidence_edge_integrity',
    title: 'Counterevidence edge integrity',
    structuralScope: 'Checks explicit contradiction edges and endpoint references.',
    cannotEstablish: 'Whether the contradiction is substantively valid.',
    defaultPolicy: 'unavailable_optional',
  },
  {
    id: 'hypothesis_structure_completeness',
    title: 'Hypothesis structure completeness',
    structuralScope: 'Checks nonempty structural fields and required list presence.',
    cannotEstablish: 'Novelty, plausibility, or truth.',
    defaultPolicy: 'fail_required',
  },
  {
    id: 'experiment_design_arithmetic_integrity',
    title: 'Experiment design arithmetic integrity',
    structuralScope: 'Checks declared controls, analysis fields, and checked budget sums.',
    cannotEstablish: 'Power, feasibility, fairness, or scientific validity.',
    defaultPolicy: 'fail_required',
  },
  {
    id: 'uncertainty_field_wellformedness',
    title: 'Uncertainty field well-formedness',
    structuralScope: 'Checks declared uncertainty grades and explicit epistemic status.',
    cannotEstablish: 'Calibration or whether uncertainty is justified.',
    defaultPolicy: 'fail_required',
  },
  {
    id: 'artifact_snapshot_replay_integrity',
    title: 'Artifact snapshot & replay integrity',
    structuralScope: 'Checks digest-bound snapshot identity and reports unavailable source replay.',
    cannotEstablish: 'Reproducibility of unavailable source documents or scientific findings.',
    defaultPolicy: 'fail_required',
  },
]

export interface CanonicalRulePin {
  readonly dimension: QualityDimensionId
  readonly rule_id: string
  readonly version: string
  readonly title: string
  readonly scope: string
  readonly cannot_establish: string
  readonly empty_collection_policy: EmptyCollectionPolicy
}

export interface CanonicalBenchmarkLimits {
  readonly request_bytes: number
  readonly graph_depth: number
  readonly graph_nodes: number
  readonly graph_edges: number
  readonly subjects: number
  readonly artifact_bytes: number
  readonly snapshot_bytes: number
  readonly report_bytes: number
  readonly report_list_bytes: number
  readonly page_size: number
}

export interface CanonicalBenchmark {
  readonly schema: 'aether.quality-benchmark/1'
  readonly id: 'aether-research-quality-structural'
  readonly version: string
  readonly scope: 'structural_checks_only'
  readonly evaluator_version: string
  readonly dimension_rules: readonly CanonicalRulePin[]
  readonly limits: CanonicalBenchmarkLimits
  readonly excluded_metrics: readonly string[]
}

export interface CanonicalBenchmarkCatalogEntry {
  readonly benchmark_sha256: string
  readonly id: string
  readonly version: string
  readonly scope: 'structural_checks_only'
  readonly dimensions: readonly QualityDimensionId[]
}

export interface CanonicalBenchmarkCatalog {
  readonly schema: 'aether.quality-benchmarks/1'
  readonly benchmarks: readonly CanonicalBenchmarkCatalogEntry[]
}

export interface CanonicalBenchmarkDetail {
  readonly schema: 'aether.quality-benchmark-detail/1'
  readonly benchmark_sha256: string
  readonly benchmark: CanonicalBenchmark
}

export interface CanonicalArtifactRef {
  readonly kind: ArtifactKind
  readonly sha256: string
}

export interface CanonicalSubjectRef {
  readonly kind: SubjectKind
  readonly artifact: CanonicalArtifactRef
  readonly pointer: string
}

export interface CanonicalSnapshotRequest {
  readonly schema: 'aether.quality-snapshot-request/1'
  readonly graph_root_id: string
  readonly graph_depth: number
  readonly subjects: readonly CanonicalSubjectRef[]
  readonly generate_hypotheses: boolean
  readonly generate_experiment_designs: boolean
}

export interface CanonicalSnapshotReceipt {
  readonly schema: 'aether.quality-snapshot-receipt/1'
  readonly snapshot_sha256: string
  readonly graph_root_id: string
  readonly graph_sha256: string
  readonly source_text_included: false
  readonly source_replay: SourceReplayStatus
}

export type CanonicalEvidenceOrigin =
  | {
      readonly kind: 'passage'
      readonly paper: string | null
      readonly paper_version: string | null
      readonly document: string | null
      readonly source_sha256: string | null
      readonly source_url: string | null
      readonly license: string | null
      readonly start_byte: number | null
      readonly end_byte: number | null
      readonly section: string | null
    }
  | {
      readonly kind: 'metadata'
      readonly paper: string | null
      readonly snapshot_sha256: string | null
    }
  | {
      readonly kind: 'archive'
      readonly artifact: string | null
      readonly artifact_sha256: string | null
      readonly pointer: string | null
    }
  | {
      readonly kind: 'campaign'
      readonly campaign: string | null
      readonly protocol_sha256: string | null
      readonly record: string | null
    }
  | { readonly kind: 'operator' }

export interface CanonicalGraphNodeFact {
  readonly id: string
  readonly kind: GraphKind
  readonly revision_sha256: string
  readonly statement_sha256: string | null
  readonly statement_bytes: number
  readonly source: CanonicalEvidenceOrigin | null
  readonly source_identity_status: SourceIdentityStatus | null
  readonly epistemic_status: EpistemicStatus | null
}

export interface CanonicalGraphEdgeFact {
  readonly id: string
  readonly origin: string
  readonly target: string
  readonly relation: EvidenceRelation
}

export type CanonicalSubjectFacts =
  | {
      readonly kind: 'hypothesis'
      readonly reference: CanonicalSubjectRef
      readonly proposition_present: boolean
      readonly support_count: number
      readonly counterevidence_count: number
      readonly alternative_count: number
      readonly assumption_count: number
      readonly limitation_count: number
      readonly falsification_present: boolean
      readonly metric_count: number
      readonly resources_present: boolean
      readonly uncertainty_grade: number | null
      readonly epistemic_status: EpistemicStatus | null
    }
  | {
      readonly kind: 'experiment_design'
      readonly reference: CanonicalSubjectRef
      readonly independent_variable_count: number
      readonly dependent_variable_present: boolean
      readonly control_count: number
      readonly baseline_present: boolean
      readonly confounder_analysis_present: boolean
      readonly development_and_test_separated: boolean
      readonly sample_size_basis_present: boolean
      readonly statistical_plan_present: boolean
      readonly metric_count: number
      readonly falsification_present: boolean
      readonly termination_present: boolean
      readonly resource_cap_attempts: number
      readonly search_attempts: number
      readonly verification_attempts: number
      readonly oracle_attempts: number
      readonly reserve_attempts: number
      readonly resource_cap_wall_ms: number
      readonly declared_cpu_ms_ceiling: number
    }

export interface CanonicalStructuralInput {
  readonly graph_root_id: string
  readonly graph_sha256: string
  readonly graph_depth: number
  readonly graph_truncated: boolean
  readonly nodes: readonly CanonicalGraphNodeFact[]
  readonly edges: readonly CanonicalGraphEdgeFact[]
  readonly subjects: readonly CanonicalSubjectFacts[]
  readonly snapshot_digest_verified: boolean
  readonly graph_digest_verified: boolean
  readonly source_replay_available: boolean
}

export interface CanonicalSnapshotDocument {
  readonly schema: 'aether.quality-snapshot/1'
  readonly input: CanonicalStructuralInput
  readonly source_text_included: false
  readonly source_replay: 'unavailable'
}

export interface CanonicalRunRequest {
  readonly schema: 'aether.quality-run-request/1'
  readonly benchmark_sha256: string
  readonly snapshot_sha256: string
}

export interface CanonicalStructuralCheck {
  readonly rule_id: string
  readonly outcome: StructuralOutcome
  readonly reason_codes: readonly string[]
}

export interface CanonicalDimensionResult {
  readonly dimension: QualityDimensionId
  readonly outcome: OutcomeStatus
  readonly checks: readonly CanonicalStructuralCheck[]
}

type OutcomeStatus = StructuralOutcome

export interface CanonicalDimensionRulePin {
  readonly dimension: QualityDimensionId
  readonly rule_id: string
  readonly version: string
}

export interface CanonicalEvaluatorIdentity {
  readonly version: string
  readonly executable_sha256: string | null
  readonly build_attestation: 'not_provided'
}

export interface CanonicalReport {
  readonly schema: 'aether.quality-report/1'
  readonly benchmark_sha256: string
  readonly dimension_rule_pins: readonly CanonicalDimensionRulePin[]
  readonly evaluator: CanonicalEvaluatorIdentity
  readonly snapshot_sha256: string
  readonly graph_root_id: string
  readonly graph_sha256: string
  readonly execution_outcome: 'completed'
  readonly dimensions: readonly CanonicalDimensionResult[]
  readonly semantic_assessment: 'not_evaluated'
  readonly independent_scientific_validation: 'not_performed'
  readonly source_replay: SourceReplayStatus
  readonly limitations: readonly string[]
}

export interface CanonicalReportEnvelope {
  readonly schema: 'aether.quality-report-envelope/1'
  readonly report_sha256: string
  readonly sequence: number
  readonly report: CanonicalReport
}

export interface CanonicalDimensionSummary {
  readonly dimension: QualityDimensionId
  readonly outcome: StructuralOutcome
}

export interface CanonicalReportSummary {
  readonly report_sha256: string
  readonly sequence: number
  readonly benchmark_sha256: string
  readonly snapshot_sha256: string
  readonly execution_outcome: 'completed'
  readonly dimensions: readonly CanonicalDimensionSummary[]
}

export interface CanonicalReportList {
  readonly schema: 'aether.quality-reports/1'
  readonly reports: readonly CanonicalReportSummary[]
  readonly truncated: boolean
  readonly next_before_sequence: number | null
}

export type IntegrationStatus =
  | {
      readonly status: 'awaiting_canonical_schema'
      readonly expectedSchemaPath: string
      readonly summary: string
    }
  | { readonly status: 'connected' }

export interface BenchmarkRuleView {
  readonly id: string
  readonly version: string
  readonly dimension: QualityDimensionId
  readonly title: string
  readonly description: string
  readonly cannotEstablish: string
  readonly emptyCollectionPolicy: EmptyCollectionPolicy
}

export interface ResourceConstraintView {
  readonly label: string
  readonly value: string
  readonly detail: string
}

export interface BenchmarkView {
  readonly digest: string
  readonly id: string
  readonly version: string
  readonly evaluatorVersion: string
  readonly scopeLabel: string
  readonly rules: readonly BenchmarkRuleView[]
  readonly constraints: readonly ResourceConstraintView[]
  readonly unavailableMetrics: readonly string[]
}

export interface KnowledgeRootOption {
  readonly id: string
  readonly kind: string
  readonly label: string
  readonly status?: string
}

export interface SnapshotUnresolvedView {
  readonly referenceDigest: string
  readonly reason: string
  readonly legacyNote?: string
}

export interface SnapshotPreviewView {
  readonly rootId: string
  readonly depth?: number
  readonly graphDigest: string
  readonly exportDigest?: string
  readonly truncated?: boolean
  readonly snapshotDigest?: string
  readonly sourceTextIncluded: boolean
  readonly nodeCount?: number
  readonly edgeCount?: number
  readonly subjectCount?: number
  readonly resolvedArtifactsCount: number
  readonly unresolvedItems: readonly SnapshotUnresolvedView[]
  readonly replayStatusLabel?: string
  readonly isStaleRevision?: boolean
  readonly notes: readonly string[]
}

export interface QualityDimensionSummaryView {
  readonly dimension: QualityDimensionId
  readonly title: string
  readonly status: StructuralOutcome
  readonly semanticLabel: string
  readonly checksCount: number
  readonly failCount: number
  readonly unavailableCount: number
  readonly passCount: number
  readonly limitations: readonly string[]
}

export interface QualityRunSummaryView {
  readonly runKey: string
  readonly sequence?: number
  readonly reportDigest: string
  readonly benchmarkTitle: string
  readonly benchmarkVersion: string
  readonly benchmarkDigest: string
  readonly snapshotDigest: string
  readonly executionStatus: ExecutionOutcome
  readonly dimensions: readonly QualityDimensionSummaryView[]
}

export interface QualityCheckView {
  readonly id: string
  readonly dimension: QualityDimensionId
  readonly ruleVersion: string
  readonly status: StructuralOutcome
  readonly reasonLabel: string
  readonly reasonDetail: string
  readonly subjectKind?: string
  readonly subjectDigest?: string
  readonly subjectPointer?: string
  readonly evidenceIds: readonly string[]
}

export interface QualityEvidenceItemView {
  readonly id: string
  readonly relationLabel: string
  readonly polarity: EvidencePolarity
  readonly claimRef?: string
  readonly canonicalSource?: string
  readonly observedAliases: readonly string[]
  readonly version?: string
  readonly sourceDigest?: string
  readonly sourceUrl?: string
  readonly sourceGroup?: string
  readonly passageLocatorLabel?: string
  readonly representationDigest?: string
  readonly passageDigest?: string
  readonly provenanceDigest?: string
  readonly provenanceSchema?: string
  readonly sourceIdentityStatus?: SourceIdentityStatus | null
  readonly epistemicOriginLabel?: string | null
  readonly integrityStatus: EvidenceIntegrityOutcome
  readonly integrityReason?: string
  readonly campaignVerification?: string
}

export interface QualityReportDetailView {
  readonly reportDigest: string
  readonly sequence?: number
  readonly snapshotDigest: string
  readonly graphRootId: string
  readonly graphSha256: string
  readonly benchmarkLabel: string
  readonly benchmarkDigest: string
  readonly configurationDigest: string
  readonly serializationLabel: string
  readonly evaluatorVersion: string
  readonly evaluatorIdentityDigest?: string
  readonly evaluatorBinaryDigest?: string
  readonly evaluatorBuildAttestation: string
  readonly evaluatorLimitations: readonly string[]
  readonly replaySummary: string
  readonly executionStatus: ExecutionOutcome
  readonly humanReviewLabel: string
  readonly independentValidationLabel: string
  readonly limitations: readonly string[]
  readonly dimensions: readonly QualityDimensionSummaryView[]
  readonly checks: readonly QualityCheckView[]
  readonly evidence: readonly QualityEvidenceItemView[]
  readonly snapshotMetadata?: SnapshotPreviewView
  readonly rawArtifactBytes?: Uint8Array
}

export interface QualityActionError {
  readonly message: string
  readonly code?: string
  readonly statusCode?: number
  readonly detail?: string
}

export interface QualityWorkspaceData {
  readonly integration: IntegrationStatus
  readonly benchmarksLoading?: boolean
  readonly benchmarksError?: QualityActionError | null
  readonly benchmarks: readonly BenchmarkView[]
  readonly benchmarkHasMore?: boolean
  readonly runsLoading?: boolean
  readonly runsError?: QualityActionError | null
  readonly runs: readonly QualityRunSummaryView[]
  readonly runsHasMore?: boolean
  readonly runsTruncated?: boolean
  readonly activeReportLoading?: boolean
  readonly activeReportError?: QualityActionError | null
  readonly activeReport?: QualityReportDetailView | null
  readonly knowledgeRoots: readonly KnowledgeRootOption[]
  readonly knowledgeSearchLoading?: boolean
  readonly knowledgeSearchError?: QualityActionError | null
  readonly snapshotPreview?: SnapshotPreviewView | null
}

export interface QualityWorkspaceActions {
  readonly onLoadMoreBenchmarks?: () => Promise<void>
  readonly onSearchKnowledgeRoots?: (query: string, kind: string) => Promise<void>
  readonly onCaptureSnapshot?: (request: CanonicalSnapshotRequest) => Promise<SnapshotPreviewView>
  readonly onRunEvaluation?: (request: CanonicalRunRequest) => Promise<void>
  readonly onSelectRun?: (reportDigest: string) => Promise<void>
  readonly onLoadMoreRuns?: () => Promise<void>
  readonly onFetchRawReportBytes?: (reportDigest: string) => Promise<Uint8Array>
}

export interface DigestVerificationResult {
  readonly matched: boolean
  readonly computedSha256: string
  readonly byteLength: number
  readonly error?: string
}

const OUTCOME_PRIORITY: Readonly<Record<StructuralOutcome, number>> = {
  fail: 0,
  unavailable: 1,
  pass: 2,
}

const REASON_CODE_DESCRIPTIONS: Readonly<Record<string, string>> = {
  graph_truncated: 'Graph projection reached backend traversal caps before full inventory/replay could be observed.',
  required_collection_empty: 'Required graph or paper collection is empty under the pinned structural rule.',
  legacy_locator_unavailable: 'One or more linked evidence origins use legacy metadata/operator records lacking a passage byte range or archive pointer.',
  invalid_source_locator: 'An explicit source locator has invalid digest formatting, pointer path, or byte offsets (start_byte >= end_byte).',
  evidence_collection_empty: 'No evidence nodes exist in the snapshot graph slice (unavailable under optional policy).',
  provenance_fields_invalid: 'An evidence node is missing its provenance object or contains malformed typed provenance fields.',
  epistemic_or_identity_status_missing: 'One or more evidence nodes omit source_identity_status or epistemic_status (missing vs explicit unknown are distinct).',
  counterevidence_collection_empty: 'No explicit contradicts edges appear in the snapshot graph slice; absence never vacuously passes.',
  invalid_counterevidence_reference: 'A contradicts edge points to a missing node or forms a self-referential cycle.',
  required_hypothesis_missing: 'No typed hypothesis subjects were resolved in this snapshot.',
  hypothesis_required_field_missing_or_empty: 'One or more hypotheses omit required proposition, support, alternative, assumption, limitation, falsification, metric, or resource fields.',
  required_experiment_design_missing: 'No typed experiment design subjects were resolved in this snapshot.',
  design_fields_or_budget_arithmetic_invalid: 'An experiment design omits required methodology fields, held-out custody separation, or fails checked attempt/CPU budget arithmetic.',
  uncertainty_out_of_range: 'Declared uncertainty_grade exceeds the bounded 0–4 scale.',
  uncertainty_or_epistemic_status_missing: 'One or more hypotheses omit uncertainty_grade or explicit epistemic_status.',
  digest_mismatch: 'Snapshot or graph export digest verification reported false.',
  source_replay_unavailable_rights_unknown: 'Source text is redacted (source_text_included=false) and source replay is unavailable without retained rights.',
}

export function isHexDigest(value: string): boolean {
  return /^[0-9a-f]{64}$/.test(value)
}

export function safeHttpUrl(value: unknown): string | null {
  if (typeof value !== 'string') return null
  try {
    const url = new URL(value)
    return url.protocol === 'https:' || url.protocol === 'http:' ? url.href : null
  } catch {
    return null
  }
}

export function formatDigest(digest: string | undefined | null, head = 12): string {
  if (!digest) return 'Not recorded'
  return digest.length > head ? `${digest.slice(0, head)}…` : digest
}

export function estimateRequestBytes(payload: unknown): number {
  return new TextEncoder().encode(JSON.stringify(payload)).byteLength
}

export function expectedArtifactKindForSubject(kind: SubjectKind): ArtifactKind {
  return kind === 'hypothesis' ? 'hypothesis_report' : 'experiment_design_report'
}

export function buildSnapshotRequest(params: {
  readonly graphRootId: string
  readonly graphDepth: number
  readonly subjects: readonly CanonicalSubjectRef[]
  readonly generateHypotheses: boolean
  readonly generateExperimentDesigns: boolean
}): CanonicalSnapshotRequest {
  const root = params.graphRootId.trim()
  if (!isHexDigest(root)) {
    throw new Error('Graph root ID must be a 64-character lowercase hexadecimal digest.')
  }
  if (!Number.isInteger(params.graphDepth) || params.graphDepth < 0 || params.graphDepth > QUALITY_LIMITS.graphDepthMax) {
    throw new Error(`Graph traversal depth must be an integer from 0 to ${QUALITY_LIMITS.graphDepthMax}.`)
  }
  if ((params.generateHypotheses || params.generateExperimentDesigns) && params.graphDepth !== 2) {
    throw new Error('Generating APO-03 hypotheses or APO-04 experiment designs requires graph_depth = 2.')
  }
  if (params.subjects.length > QUALITY_LIMITS.subjectsMax) {
    throw new Error(`Snapshot requests accept at most ${QUALITY_LIMITS.subjectsMax} typed subject references.`)
  }
  const seen = new Set<string>()
  for (const ref of params.subjects) {
    if (!isHexDigest(ref.artifact.sha256)) {
      throw new Error('Each subject artifact digest must be 64 lowercase hexadecimal characters.')
    }
    if (ref.artifact.kind !== expectedArtifactKindForSubject(ref.kind)) {
      throw new Error(`Subject kind "${ref.kind}" must reference artifact kind "${expectedArtifactKindForSubject(ref.kind)}".`)
    }
    if (!ref.pointer.startsWith('/') || ref.pointer.length > 512) {
      throw new Error('Each subject JSON pointer must start with "/" and be at most 512 characters.')
    }
    const key = `${ref.kind}:${ref.artifact.sha256}:${ref.pointer}`
    if (seen.has(key)) {
      throw new Error(`Duplicate typed subject reference (${key}).`)
    }
    seen.add(key)
  }
  return {
    schema: 'aether.quality-snapshot-request/1',
    graph_root_id: root,
    graph_depth: params.graphDepth,
    subjects: [...params.subjects],
    generate_hypotheses: params.generateHypotheses,
    generate_experiment_designs: params.generateExperimentDesigns,
  }
}

export function buildRunRequest(benchmarkSha256: string, snapshotSha256: string): CanonicalRunRequest {
  const benchmark = benchmarkSha256.trim()
  const snapshot = snapshotSha256.trim()
  if (!isHexDigest(benchmark)) {
    throw new Error('Benchmark SHA-256 must be a 64-character lowercase hexadecimal digest.')
  }
  if (!isHexDigest(snapshot)) {
    throw new Error('Snapshot SHA-256 must be a 64-character lowercase hexadecimal digest.')
  }
  return {
    schema: 'aether.quality-run-request/1',
    benchmark_sha256: benchmark,
    snapshot_sha256: snapshot,
  }
}

export function describeStructuralStatus(status: StructuralOutcome): string {
  switch (status) {
    case 'pass':
      return 'Structural pass'
    case 'fail':
      return 'Structural fail'
    case 'unavailable':
      return 'Unavailable'
  }
}

export function describeExecutionStatus(status: ExecutionOutcome): string {
  switch (status) {
    case 'completed':
      return 'Execution completed'
    case 'partial':
      return 'Execution partial'
    case 'abstained':
      return 'Execution abstained'
    case 'failed':
      return 'Execution failed'
  }
}

export function describeIntegrityStatus(status: EvidenceIntegrityOutcome): string {
  switch (status) {
    case 'valid':
      return 'Integrity valid'
    case 'invalid':
      return 'Integrity invalid'
    case 'unavailable':
      return 'Integrity unavailable'
  }
}

export function describeReasonCodes(codes: readonly string[]): string {
  if (!codes.length) return 'All pinned structural predicates in this rule were satisfied.'
  return codes.map((code) => REASON_CODE_DESCRIPTIONS[code] ?? `Reported code: ${code}`).join(' ')
}

export function sortDimensionsNonPassFirst(
  dimensions: readonly QualityDimensionSummaryView[],
): QualityDimensionSummaryView[] {
  const order = new Map(QUALITY_DIMENSIONS.map((entry, index) => [entry.id, index]))
  return [...dimensions].sort((left, right) => {
    const rank = OUTCOME_PRIORITY[left.status] - OUTCOME_PRIORITY[right.status]
    if (rank !== 0) return rank
    return (order.get(left.dimension) ?? 99) - (order.get(right.dimension) ?? 99)
  })
}

export function sortChecksNonPassFirst(checks: readonly QualityCheckView[]): QualityCheckView[] {
  return [...checks].sort((left, right) => {
    const rank = OUTCOME_PRIORITY[left.status] - OUTCOME_PRIORITY[right.status]
    if (rank !== 0) return rank
    return left.id.localeCompare(right.id)
  })
}

export interface PaginatedResult<T> {
  readonly slice: readonly T[]
  readonly page: number
  readonly totalPages: number
  readonly totalItems: number
  readonly startIndex: number
  readonly endIndex: number
}

export function paginateItems<T>(
  items: readonly T[],
  requestedPage: number,
  pageSize = DEFAULT_CHECK_PAGE_SIZE,
): PaginatedResult<T> {
  const safeSize = Math.max(1, Math.floor(pageSize))
  const totalItems = items.length
  const totalPages = Math.max(1, Math.ceil(totalItems / safeSize))
  const page = Math.min(totalPages, Math.max(1, Math.floor(requestedPage) || 1))
  const offset = (page - 1) * safeSize
  const slice = items.slice(offset, offset + safeSize)
  return {
    slice,
    page,
    totalPages,
    totalItems,
    startIndex: totalItems === 0 ? 0 : offset + 1,
    endIndex: offset + slice.length,
  }
}

export function partitionEvidence(evidence: readonly QualityEvidenceItemView[]) {
  const supporting: QualityEvidenceItemView[] = []
  const contradictory: QualityEvidenceItemView[] = []
  const contextual: QualityEvidenceItemView[] = []
  for (const item of evidence) {
    if (item.polarity === 'supporting') supporting.push(item)
    else if (item.polarity === 'contradictory') contradictory.push(item)
    else contextual.push(item)
  }
  return { supporting, contradictory, contextual }
}

export function formatQualityError(error: QualityActionError | null | undefined): string {
  if (!error) return ''
  if (error.statusCode === 413 || error.code === 'request_too_large') {
    const suffix = error.detail ? ` (${error.detail})` : ''
    return `HTTP 413 request_too_large: ${error.message}${suffix}`
  }
  if (error.statusCode || error.code) {
    const tags = [error.statusCode ? `HTTP ${error.statusCode}` : '', error.code ?? ''].filter(Boolean).join(' ')
    return `${tags}: ${error.message}`
  }
  return error.message
}

export async function verifyRawArtifactDigest(
  bytes: Uint8Array,
  expectedSha256: string,
): Promise<DigestVerificationResult> {
  if (!isHexDigest(expectedSha256)) {
    return {
      matched: false,
      computedSha256: '',
      byteLength: bytes.byteLength,
      error: 'Expected SHA-256 digest is not a 64-character lowercase hexadecimal string.',
    }
  }
  if (!globalThis.crypto?.subtle) {
    return {
      matched: false,
      computedSha256: '',
      byteLength: bytes.byteLength,
      error: 'Web Crypto API (crypto.subtle.digest) is unavailable in this environment.',
    }
  }
  const copy = new Uint8Array(bytes.byteLength)
  copy.set(bytes)
  const digest = await globalThis.crypto.subtle.digest('SHA-256', copy.buffer)
  const computedSha256 = [...new Uint8Array(digest)]
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('')
  return {
    matched: computedSha256 === expectedSha256,
    computedSha256,
    byteLength: bytes.byteLength,
    error:
      computedSha256 === expectedSha256
        ? undefined
        : 'Computed SHA-256 of raw artifact bytes differs from expected report digest.',
  }
}

export function defaultDimensionsUnavailable(): QualityDimensionSummaryView[] {
  return QUALITY_DIMENSIONS.map((entry) => ({
    dimension: entry.id,
    title: entry.title,
    status: 'unavailable',
    semanticLabel: 'Human review: Not started (not_evaluated)',
    checksCount: 0,
    failCount: 0,
    unavailableCount: 0,
    passCount: 0,
    limitations: [entry.cannotEstablish],
  }))
}

export function mapBenchmarkDetailToView(detail: CanonicalBenchmarkDetail): BenchmarkView {
  const { benchmark } = detail
  return {
    digest: detail.benchmark_sha256,
    id: benchmark.id,
    version: benchmark.version,
    evaluatorVersion: benchmark.evaluator_version,
    scopeLabel: 'Structural checks only (no scientific truth or validation)',
    rules: benchmark.dimension_rules.map((rule) => ({
      id: rule.rule_id,
      version: rule.version,
      dimension: rule.dimension,
      title: rule.title,
      description: rule.scope,
      cannotEstablish: rule.cannot_establish,
      emptyCollectionPolicy: rule.empty_collection_policy,
    })),
    constraints: [
      { label: 'POST request body limit', value: `${benchmark.limits.request_bytes} bytes`, detail: 'Enforced on /v1 router via DefaultBodyLimit (HTTP 413 request_too_large)' },
      { label: 'Max graph depth', value: String(benchmark.limits.graph_depth), detail: 'Generated hypotheses or experiment designs require depth = 2' },
      { label: 'Max graph nodes', value: String(benchmark.limits.graph_nodes), detail: 'Bounded structural node projection ceiling' },
      { label: 'Max graph edges', value: String(benchmark.limits.graph_edges), detail: 'Bounded directed relation ceiling' },
      { label: 'Max typed subjects', value: String(benchmark.limits.subjects), detail: 'Combined explicit + generated hypotheses and designs' },
      { label: 'Max subject artifact', value: `${benchmark.limits.artifact_bytes} bytes`, detail: '2 MiB content-addressed artifact limit' },
      { label: 'Max snapshot document', value: `${benchmark.limits.snapshot_bytes} bytes`, detail: '4 MiB redacted snapshot projection ceiling' },
      { label: 'Max evaluation report', value: `${benchmark.limits.report_bytes} bytes`, detail: '1 MiB report payload limit' },
      { label: 'Max report list response', value: `${benchmark.limits.report_list_bytes} bytes`, detail: '256 KiB listing envelope ceiling' },
      { label: 'Report listing page size', value: `${benchmark.limits.page_size} items`, detail: 'Exclusive before_sequence descending cursor pagination' },
    ],
    unavailableMetrics: [...benchmark.excluded_metrics],
  }
}

export function mapBenchmarkCatalogEntryToFallbackView(entry: CanonicalBenchmarkCatalogEntry): BenchmarkView {
  return {
    digest: entry.benchmark_sha256,
    id: entry.id,
    version: entry.version,
    evaluatorVersion: 'Loading benchmark details…',
    scopeLabel: 'Structural checks only',
    rules: QUALITY_DIMENSIONS.filter((descriptor) => entry.dimensions.includes(descriptor.id)).map((descriptor) => ({
      id: descriptor.id,
      version: '1',
      dimension: descriptor.id,
      title: descriptor.title,
      description: descriptor.structuralScope,
      cannotEstablish: descriptor.cannotEstablish,
      emptyCollectionPolicy: descriptor.defaultPolicy,
    })),
    constraints: [],
    unavailableMetrics: [],
  }
}

function summarizeOrigin(origin: CanonicalEvidenceOrigin | null): {
  readonly canonicalSource?: string
  readonly observedAliases: readonly string[]
  readonly version?: string
  readonly sourceDigest?: string
  readonly sourceUrl?: string
  readonly sourceGroup?: string
  readonly passageLocatorLabel?: string
  readonly provenanceSchema?: string
  readonly integrityReason?: string
} {
  if (!origin) {
    return {
      observedAliases: [],
      integrityReason: 'evidence_origin_missing',
    }
  }
  switch (origin.kind) {
    case 'passage': {
      const span =
        origin.start_byte !== null && origin.end_byte !== null
          ? `utf8_byte_range [${origin.start_byte}, ${origin.end_byte})${origin.section ? ` · ${origin.section}` : ''}`
          : undefined
      return {
        canonicalSource: origin.paper ?? origin.document ?? undefined,
        observedAliases: [origin.document, origin.section, origin.license].filter(
          (item): item is string => Boolean(item),
        ),
        version: origin.paper_version ?? undefined,
        sourceDigest: origin.source_sha256 ?? undefined,
        sourceUrl: origin.source_url ?? undefined,
        passageLocatorLabel: span,
        provenanceSchema: 'origin:passage',
        integrityReason: span ? undefined : 'passage_not_recorded',
      }
    }
    case 'metadata':
      return {
        canonicalSource: origin.paper ?? undefined,
        observedAliases: origin.snapshot_sha256 ? [`snapshot:${formatDigest(origin.snapshot_sha256, 10)}`] : [],
        sourceDigest: origin.snapshot_sha256 ?? undefined,
        provenanceSchema: 'origin:metadata',
        integrityReason: 'legacy_metadata_lacks_passage_locator',
      }
    case 'archive':
      return {
        canonicalSource: origin.artifact ?? undefined,
        observedAliases: origin.pointer ? [origin.pointer] : [],
        sourceDigest: origin.artifact_sha256 ?? undefined,
        passageLocatorLabel: origin.pointer ?? undefined,
        provenanceSchema: 'origin:archive',
      }
    case 'campaign':
      return {
        canonicalSource: origin.campaign ?? undefined,
        observedAliases: origin.record ? [origin.record] : [],
        sourceDigest: origin.protocol_sha256 ?? undefined,
        passageLocatorLabel: origin.record ?? undefined,
        provenanceSchema: 'origin:campaign',
      }
    case 'operator':
      return {
        canonicalSource: 'operator',
        observedAliases: [],
        provenanceSchema: 'origin:operator',
        integrityReason: 'operator_entry_without_source_passage',
      }
  }
}

export function mapSnapshotReceiptToPreview(
  receipt: CanonicalSnapshotReceipt,
  doc?: CanonicalSnapshotDocument,
): SnapshotPreviewView {
  const unresolvedItems: SnapshotUnresolvedView[] = []
  if (doc) {
    for (const node of doc.input.nodes) {
      if (!node.source || node.source.kind === 'metadata' || node.source.kind === 'operator') {
        unresolvedItems.push({
          referenceDigest: node.id,
          reason: !node.source
            ? 'origin_missing'
            : node.source.kind === 'metadata'
              ? 'legacy_locator_unavailable'
              : 'operator_origin_no_passage',
          legacyNote: `${node.kind} node (${node.statement_bytes} redacted source bytes) preserved without source prose`,
        })
      } else if (node.epistemic_status === null || node.source_identity_status === null) {
        unresolvedItems.push({
          referenceDigest: node.id,
          reason: 'epistemic_or_identity_status_missing',
          legacyNote: 'Missing epistemic_status or source_identity_status preserved as null',
        })
      }
    }
  }
  return {
    rootId: receipt.graph_root_id,
    depth: doc?.input.graph_depth,
    graphDigest: receipt.graph_sha256,
    snapshotDigest: receipt.snapshot_sha256,
    truncated: doc?.input.graph_truncated,
    sourceTextIncluded: receipt.source_text_included,
    nodeCount: doc?.input.nodes.length,
    edgeCount: doc?.input.edges.length,
    subjectCount: doc?.input.subjects.length,
    resolvedArtifactsCount: doc ? doc.input.subjects.length : 0,
    unresolvedItems,
    replayStatusLabel:
      receipt.source_replay === 'available'
        ? 'Source replay available'
        : 'Unavailable — source text excluded (source_text_included: false)',
    notes: [
      'Snapshot excludes source prose (source_text_included = false); only digests, typed locators, edges, and subject counts are stored.',
      ...(doc?.input.graph_truncated ? ['Knowledge graph projection reached backend traversal limits (graph_truncated = true).'] : []),
    ],
  }
}

export function mapReportListToRunViews(
  list: CanonicalReportList,
  benchmark?: BenchmarkView,
): QualityRunSummaryView[] {
  const ruleTitleByDimension = new Map(QUALITY_DIMENSIONS.map((entry) => [entry.id, entry.title]))
  const cannotByDimension = new Map(
    (benchmark?.rules ?? []).map((rule) => [rule.dimension, rule.cannotEstablish]),
  )
  return list.reports.map((summary) => ({
    runKey: summary.report_sha256,
    sequence: summary.sequence,
    reportDigest: summary.report_sha256,
    benchmarkTitle: benchmark?.id ?? 'aether-research-quality-structural',
    benchmarkVersion: benchmark?.version ?? '1.0.0',
    benchmarkDigest: summary.benchmark_sha256,
    snapshotDigest: summary.snapshot_sha256,
    executionStatus: summary.execution_outcome,
    dimensions: summary.dimensions.map((item) => ({
      dimension: item.dimension,
      title: ruleTitleByDimension.get(item.dimension) ?? item.dimension,
      status: item.outcome,
      semanticLabel: 'Human review: Not started (not_evaluated)',
      checksCount: 1,
      failCount: item.outcome === 'fail' ? 1 : 0,
      unavailableCount: item.outcome === 'unavailable' ? 1 : 0,
      passCount: item.outcome === 'pass' ? 1 : 0,
      limitations: cannotByDimension.has(item.dimension)
        ? [cannotByDimension.get(item.dimension) ?? '']
        : [],
    })),
  }))
}

export function mapSnapshotFactsToEvidence(doc: CanonicalSnapshotDocument): QualityEvidenceItemView[] {
  const nodeById = new Map(doc.input.nodes.map((node) => [node.id, node]))
  const items: QualityEvidenceItemView[] = []
  const linkedOrigins = new Set<string>()

  for (const edge of doc.input.edges) {
    const originNode = nodeById.get(edge.origin)
    const targetNode = nodeById.get(edge.target)
    if (!originNode) continue
    linkedOrigins.add(originNode.id)
    const originSummary = summarizeOrigin(originNode.source)
    const integrityStatus: EvidenceIntegrityOutcome =
      !originNode.source
        ? 'invalid'
        : originNode.source.kind === 'metadata' ||
            originNode.source.kind === 'operator' ||
            originNode.epistemic_status === null ||
            originNode.source_identity_status === null
          ? 'unavailable'
          : 'valid'
    items.push({
      id: edge.id,
      relationLabel: edge.relation,
      polarity:
        edge.relation === 'supports'
          ? 'supporting'
          : edge.relation === 'contradicts'
            ? 'contradictory'
            : 'contextual',
      claimRef: targetNode ? `${targetNode.kind} · ${formatDigest(targetNode.id, 12)}` : edge.target,
      canonicalSource: originSummary.canonicalSource ?? `${originNode.kind} · ${formatDigest(originNode.id, 10)}`,
      observedAliases: originSummary.observedAliases,
      version: originSummary.version,
      sourceDigest: originSummary.sourceDigest ?? originNode.statement_sha256 ?? undefined,
      sourceUrl: originSummary.sourceUrl,
      sourceGroup: originNode.kind,
      passageLocatorLabel: originSummary.passageLocatorLabel,
      representationDigest: originNode.revision_sha256,
      passageDigest: originNode.statement_sha256 ?? undefined,
      provenanceDigest: originNode.id,
      provenanceSchema: originSummary.provenanceSchema,
      sourceIdentityStatus: originNode.source_identity_status,
      epistemicOriginLabel: originNode.epistemic_status,
      integrityStatus,
      integrityReason: originSummary.integrityReason,
      campaignVerification:
        originNode.source_identity_status === 'verified'
          ? 'Source identity asserted as verified (not independently audited by APO-11)'
          : originNode.source_identity_status === 'unverified'
            ? 'Unverified — structural check does not authenticate sources'
            : originNode.source_identity_status === 'not_applicable'
              ? 'Not applicable'
              : 'Unrecorded (null)',
    })
  }

  for (const node of doc.input.nodes) {
    if (node.kind !== 'evidence' || linkedOrigins.has(node.id)) continue
    const originSummary = summarizeOrigin(node.source)
    const integrityStatus: EvidenceIntegrityOutcome =
      !node.source
        ? 'invalid'
        : node.source.kind === 'metadata' ||
            node.source.kind === 'operator' ||
            node.epistemic_status === null ||
            node.source_identity_status === null
          ? 'unavailable'
          : 'valid'
    items.push({
      id: node.id,
      relationLabel: 'evidence_node',
      polarity: 'contextual',
      canonicalSource: originSummary.canonicalSource ?? `evidence · ${formatDigest(node.id, 10)}`,
      observedAliases: originSummary.observedAliases,
      version: originSummary.version,
      sourceDigest: originSummary.sourceDigest ?? node.statement_sha256 ?? undefined,
      sourceUrl: originSummary.sourceUrl,
      sourceGroup: node.kind,
      passageLocatorLabel: originSummary.passageLocatorLabel,
      representationDigest: node.revision_sha256,
      passageDigest: node.statement_sha256 ?? undefined,
      provenanceDigest: node.id,
      provenanceSchema: originSummary.provenanceSchema,
      sourceIdentityStatus: node.source_identity_status,
      epistemicOriginLabel: node.epistemic_status,
      integrityStatus,
      integrityReason: originSummary.integrityReason,
      campaignVerification: 'Unlinked evidence node in snapshot',
    })
  }

  return items
}

export function mapReportEnvelopeToDetailView(
  envelope: CanonicalReportEnvelope,
  options?: {
    readonly benchmark?: BenchmarkView
    readonly snapshotDoc?: CanonicalSnapshotDocument
    readonly rawArtifactBytes?: Uint8Array
  },
): QualityReportDetailView {
  const { report } = envelope
  const ruleByDimension = new Map(
    (options?.benchmark?.rules ?? []).map((rule) => [rule.dimension, rule]),
  )
  const pinByDimension = new Map(report.dimension_rule_pins.map((pin) => [pin.dimension, pin]))
  const descriptorByDimension = new Map(QUALITY_DIMENSIONS.map((entry) => [entry.id, entry]))

  const dimensions: QualityDimensionSummaryView[] = report.dimensions.map((result) => {
    const descriptor = descriptorByDimension.get(result.dimension)
    const rule = ruleByDimension.get(result.dimension)
    const failCount = result.checks.filter((check) => check.outcome === 'fail').length
    const unavailableCount = result.checks.filter((check) => check.outcome === 'unavailable').length
    const passCount = result.checks.filter((check) => check.outcome === 'pass').length
    return {
      dimension: result.dimension,
      title: descriptor?.title ?? rule?.title ?? result.dimension,
      status: result.outcome,
      semanticLabel: `Human review: ${report.semantic_assessment}`,
      checksCount: result.checks.length,
      failCount,
      unavailableCount,
      passCount,
      limitations: [rule?.cannotEstablish ?? descriptor?.cannotEstablish ?? 'Structural check only.'],
    }
  })

  const evidence = options?.snapshotDoc ? mapSnapshotFactsToEvidence(options.snapshotDoc) : []
  const snapshotSubjects = options?.snapshotDoc?.input.subjects ?? []

  const checks: QualityCheckView[] = report.dimensions.flatMap((result) =>
    result.checks.map((check) => {
      const pin = pinByDimension.get(result.dimension)
      const matchedSubject =
        result.dimension === 'hypothesis_structure_completeness' ||
        result.dimension === 'uncertainty_field_wellformedness'
          ? snapshotSubjects.find((item) => item.kind === 'hypothesis')?.reference
          : result.dimension === 'experiment_design_arithmetic_integrity'
            ? snapshotSubjects.find((item) => item.kind === 'experiment_design')?.reference
            : undefined
      const matchingEvidenceIds =
        result.dimension === 'counterevidence_edge_integrity'
          ? evidence.filter((item) => item.polarity === 'contradictory').map((item) => item.id)
          : result.dimension === 'claim_source_locator_integrity' ||
              result.dimension === 'evidence_provenance_field_integrity'
            ? evidence.map((item) => item.id)
            : []
      return {
        id: check.rule_id,
        dimension: result.dimension,
        ruleVersion: pin?.version ?? '1',
        status: check.outcome,
        reasonLabel: check.reason_codes.length ? check.reason_codes.join(', ') : 'None',
        reasonDetail: describeReasonCodes(check.reason_codes),
        subjectKind: matchedSubject?.kind ?? 'graph_snapshot',
        subjectDigest: matchedSubject?.artifact.sha256 ?? report.graph_root_id,
        subjectPointer: matchedSubject?.pointer,
        evidenceIds: matchingEvidenceIds,
      }
    }),
  )

  const snapshotMetadata = options?.snapshotDoc
    ? mapSnapshotReceiptToPreview(
        {
          schema: 'aether.quality-snapshot-receipt/1',
          snapshot_sha256: report.snapshot_sha256,
          graph_root_id: report.graph_root_id,
          graph_sha256: report.graph_sha256,
          source_text_included: false,
          source_replay: report.source_replay,
        },
        options.snapshotDoc,
      )
    : undefined

  return {
    reportDigest: envelope.report_sha256,
    sequence: envelope.sequence,
    snapshotDigest: report.snapshot_sha256,
    graphRootId: report.graph_root_id,
    graphSha256: report.graph_sha256,
    benchmarkLabel: options?.benchmark
      ? `${options.benchmark.id} v${options.benchmark.version}`
      : 'aether-research-quality-structural',
    benchmarkDigest: report.benchmark_sha256,
    configurationDigest: report.graph_sha256,
    serializationLabel: 'serde_json::to_vec (aether.quality-report/1)',
    evaluatorVersion: report.evaluator.version,
    evaluatorIdentityDigest: undefined,
    evaluatorBinaryDigest: report.evaluator.executable_sha256 ?? undefined,
    evaluatorBuildAttestation: report.evaluator.build_attestation,
    evaluatorLimitations: [
      'executable_sha256 is self-reported by the local process and build_attestation is fixed to not_provided.',
      'report_sha256 covers the exact serialized aether.quality-report/1 bytes stored at /v1/artifacts/{report_sha256}, not the outer envelope.',
    ],
    replaySummary:
      report.source_replay === 'available'
        ? 'Source replay reported available'
        : 'Unavailable — source text is excluded from snapshots under unknown storage rights',
    executionStatus: report.execution_outcome,
    humanReviewLabel: 'Human review: Not started (semantic_assessment = not_evaluated)',
    independentValidationLabel: 'Independent scientific validation: Not performed (not_performed)',
    limitations: [...report.limitations],
    dimensions,
    checks,
    evidence,
    snapshotMetadata,
    rawArtifactBytes: options?.rawArtifactBytes,
  }
}
