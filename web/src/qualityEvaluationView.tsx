import { useEffect, useMemo, useState } from 'react'
import {
  ApiError,
  createEvalSnapshot,
  getEvalArtifactBytes,
  getEvalBenchmark,
  getEvalBenchmarks,
  getEvalReport,
  getEvalSnapshotDocument,
  listEvalReports,
  rows,
  runEvalQuality,
  searchKnowledge,
} from './api'
import { Fields, Panel, RecordTable, readableRecord, text } from './components'
import {
  EmptyOrLoading,
  EpistemicBoundaryStrip,
  ExecutionOutcomeBadge,
  IntegrationStatusNotice,
  IntegrityOutcomeBadge,
  LimitationsCallout,
  QualityAreaTabs,
  QualityErrorBanner,
  StructuralOutcomeBadge,
} from './qualityComponents'
import {
  DEFAULT_CHECK_PAGE_SIZE,
  DEFAULT_REPORT_PAGE_SIZE,
  QUALITY_AREAS,
  QUALITY_DIMENSIONS,
  QUALITY_LIMITS,
  buildRunRequest,
  buildSnapshotRequest,
  defaultDimensionsUnavailable,
  describeStructuralStatus,
  estimateRequestBytes,
  expectedArtifactKindForSubject,
  formatDigest,
  formatQualityError,
  isHexDigest,
  mapBenchmarkCatalogEntryToFallbackView,
  mapBenchmarkDetailToView,
  mapReportEnvelopeToDetailView,
  mapReportListToRunViews,
  mapSnapshotReceiptToPreview,
  paginateItems,
  partitionEvidence,
  sortChecksNonPassFirst,
  sortDimensionsNonPassFirst,
  verifyRawArtifactDigest,
  type BenchmarkView,
  type CanonicalRunRequest,
  type CanonicalSnapshotDocument,
  type CanonicalSnapshotRequest,
  type CanonicalSubjectRef,
  type DigestVerificationResult,
  type IntegrationStatus,
  type KnowledgeRootOption,
  type QualityActionError,
  type QualityAreaId,
  type QualityDimensionId,
  type QualityReportDetailView,
  type QualityRunSummaryView,
  type QualityWorkspaceActions,
  type QualityWorkspaceData,
  type SnapshotPreviewView,
  type StructuralOutcome,
  type SubjectKind,
} from './qualityModels'

export interface QualityEvaluationProps {
  readonly token?: string
  readonly data?: QualityWorkspaceData
  readonly actions?: QualityWorkspaceActions
  readonly integration?: IntegrationStatus
  readonly initialArea?: QualityAreaId
}

const DEFAULT_INTEGRATION: IntegrationStatus = {
  status: 'awaiting_canonical_schema',
  expectedSchemaPath: 'contracts/apo11/quality-evaluation.schema.json',
  summary:
    'Live token or workspace provider is not connected. Navigation, epistemic boundaries, sorting, pagination, and raw-byte verification controls operate safely without fabricated server payloads.',
}

function toActionError(reason: unknown): QualityActionError {
  if (reason instanceof ApiError) {
    return {
      statusCode: reason.status,
      code: reason.code,
      message: reason.message,
    }
  }
  if (typeof reason === 'object' && reason !== null && 'message' in reason) {
    const candidate = reason as {
      message: unknown
      code?: unknown
      statusCode?: unknown
      status?: unknown
      detail?: unknown
    }
    return {
      message: typeof candidate.message === 'string' ? candidate.message : 'Unexpected evaluation error.',
      code: typeof candidate.code === 'string' ? candidate.code : undefined,
      statusCode:
        typeof candidate.statusCode === 'number'
          ? candidate.statusCode
          : typeof candidate.status === 'number'
            ? candidate.status
            : undefined,
      detail: typeof candidate.detail === 'string' ? candidate.detail : undefined,
    }
  }
  return { message: reason instanceof Error ? reason.message : 'Unexpected evaluation error.' }
}

function useLiveQualityWorkspace(token: string | undefined): {
  readonly data: QualityWorkspaceData | undefined
  readonly actions: QualityWorkspaceActions | undefined
} {
  const [benchmarks, setBenchmarks] = useState<BenchmarkView[]>([])
  const [benchmarksLoading, setBenchmarksLoading] = useState(false)
  const [benchmarksError, setBenchmarksError] = useState<QualityActionError | null>(null)
  const [runs, setRuns] = useState<QualityRunSummaryView[]>([])
  const [runsLoading, setRunsLoading] = useState(false)
  const [runsError, setRunsError] = useState<QualityActionError | null>(null)
  const [runsHasMore, setRunsHasMore] = useState(false)
  const [runsTruncated, setRunsTruncated] = useState(false)
  const [nextBeforeSequence, setNextBeforeSequence] = useState<number | null>(null)
  const [activeReport, setActiveReport] = useState<QualityReportDetailView | null>(null)
  const [activeReportLoading, setActiveReportLoading] = useState(false)
  const [activeReportError, setActiveReportError] = useState<QualityActionError | null>(null)
  const [knowledgeRoots, setKnowledgeRoots] = useState<KnowledgeRootOption[]>([])
  const [knowledgeSearchLoading, setKnowledgeSearchLoading] = useState(false)
  const [knowledgeSearchError, setKnowledgeSearchError] = useState<QualityActionError | null>(null)
  const [snapshotPreview, setSnapshotPreview] = useState<SnapshotPreviewView | null>(null)

  async function loadReportDetail(
    bearer: string,
    reportDigest: string,
    benchmarkList: readonly BenchmarkView[],
    signal?: AbortSignal,
  ) {
    setActiveReportLoading(true)
    setActiveReportError(null)
    try {
      const envelope = await getEvalReport(bearer, reportDigest, signal)
      if (signal?.aborted) return
      let snapshotDoc: CanonicalSnapshotDocument | undefined
      try {
        snapshotDoc = await getEvalSnapshotDocument(bearer, envelope.report.snapshot_sha256, signal)
      } catch {
        // Report rendering remains valid even if optional snapshot projection details fail to load.
      }
      if (signal?.aborted) return
      const benchmark = benchmarkList.find((item) => item.digest === envelope.report.benchmark_sha256)
      setActiveReport(mapReportEnvelopeToDetailView(envelope, { benchmark, snapshotDoc }))
    } catch (reason) {
      if (!signal?.aborted) setActiveReportError(toActionError(reason))
    } finally {
      if (!signal?.aborted) setActiveReportLoading(false)
    }
  }

  async function fetchKnowledge(bearer: string, query: string, kind: string, signal?: AbortSignal) {
    setKnowledgeSearchLoading(true)
    setKnowledgeSearchError(null)
    try {
      const response = await searchKnowledge(bearer, query, kind, signal ?? new AbortController().signal)
      if (signal?.aborted) return
      const items: KnowledgeRootOption[] = rows(response.nodes).map((node) => {
        const current = readableRecord(node.current)
        return {
          id: text(node.id),
          kind: text(node.kind),
          label: text(current.label ?? node.id),
          status: current.status === undefined ? undefined : text(current.status),
        }
      })
      setKnowledgeRoots(items)
    } catch (reason) {
      if (!signal?.aborted) setKnowledgeSearchError(toActionError(reason))
    } finally {
      if (!signal?.aborted) setKnowledgeSearchLoading(false)
    }
  }

  // biome-ignore lint/correctness/useExhaustiveDependencies: Initial workspace bootstrap is keyed strictly on token change.
  useEffect(() => {
    if (!token) {
      setBenchmarks([])
      setRuns([])
      setActiveReport(null)
      setKnowledgeRoots([])
      setSnapshotPreview(null)
      return
    }
    const controller = new AbortController()
    const { signal } = controller

    async function bootstrap(bearer: string) {
      setBenchmarksLoading(true)
      setBenchmarksError(null)
      setRunsLoading(true)
      setRunsError(null)

      let loadedBenchmarks: BenchmarkView[] = []
      try {
        const catalog = await getEvalBenchmarks(bearer, signal)
        if (signal.aborted) return
        const details = await Promise.all(
          catalog.benchmarks.map(async (entry) => {
            try {
              return mapBenchmarkDetailToView(await getEvalBenchmark(bearer, entry.benchmark_sha256, signal))
            } catch {
              return mapBenchmarkCatalogEntryToFallbackView(entry)
            }
          }),
        )
        if (signal.aborted) return
        loadedBenchmarks = details
        setBenchmarks(details)
      } catch (reason) {
        if (!signal.aborted) setBenchmarksError(toActionError(reason))
      } finally {
        if (!signal.aborted) setBenchmarksLoading(false)
      }

      try {
        const list = await listEvalReports(bearer, DEFAULT_REPORT_PAGE_SIZE, undefined, signal)
        if (signal.aborted) return
        const mappedRuns = mapReportListToRunViews(list, loadedBenchmarks[0])
        setRuns(mappedRuns)
        setRunsTruncated(list.truncated)
        setRunsHasMore(list.truncated && list.next_before_sequence !== null)
        setNextBeforeSequence(list.next_before_sequence)
        if (mappedRuns[0]) {
          void loadReportDetail(bearer, mappedRuns[0].reportDigest, loadedBenchmarks, signal)
        }
      } catch (reason) {
        if (!signal.aborted) setRunsError(toActionError(reason))
      } finally {
        if (!signal.aborted) setRunsLoading(false)
      }
    }

    void bootstrap(token)
    void fetchKnowledge(token, '', '', signal)

    return () => controller.abort()
  }, [token])

  if (!token) return { data: undefined, actions: undefined }

  const actions: QualityWorkspaceActions = {
    onSearchKnowledgeRoots: async (query, kind) => fetchKnowledge(token, query, kind),
    onCaptureSnapshot: async (request: CanonicalSnapshotRequest) => {
      const receipt = await createEvalSnapshot(token, request)
      let doc: CanonicalSnapshotDocument | undefined
      try {
        doc = await getEvalSnapshotDocument(token, receipt.snapshot_sha256)
      } catch {
        // Receipt alone still pins the immutable snapshot digest for evaluation submission.
      }
      const preview = mapSnapshotReceiptToPreview(receipt, doc)
      setSnapshotPreview(preview)
      return preview
    },
    onRunEvaluation: async (request: CanonicalRunRequest) => {
      const envelope = await runEvalQuality(token, request)
      let snapshotDoc: CanonicalSnapshotDocument | undefined
      try {
        snapshotDoc = await getEvalSnapshotDocument(token, envelope.report.snapshot_sha256)
      } catch {
        // Report detail renders safely even if optional snapshot document fetch fails.
      }
      const benchmark = benchmarks.find((item) => item.digest === envelope.report.benchmark_sha256)
      setActiveReport(mapReportEnvelopeToDetailView(envelope, { benchmark, snapshotDoc }))
      try {
        const list = await listEvalReports(token, DEFAULT_REPORT_PAGE_SIZE)
        setRuns(mapReportListToRunViews(list, benchmarks[0]))
        setRunsTruncated(list.truncated)
        setRunsHasMore(list.truncated && list.next_before_sequence !== null)
        setNextBeforeSequence(list.next_before_sequence)
      } catch {
        // Keep active report visible even if background list refresh fails.
      }
    },
    onSelectRun: async (reportDigest) => {
      await loadReportDetail(token, reportDigest, benchmarks)
    },
    onLoadMoreRuns: async () => {
      if (nextBeforeSequence === null) return
      const page = await listEvalReports(token, DEFAULT_REPORT_PAGE_SIZE, nextBeforeSequence)
      const nextViews = mapReportListToRunViews(page, benchmarks[0])
      setRuns((current) => {
        const seen = new Set(current.map((item) => item.reportDigest))
        return [...current, ...nextViews.filter((item) => !seen.has(item.reportDigest))]
      })
      setRunsTruncated(page.truncated)
      setRunsHasMore(page.truncated && page.next_before_sequence !== null)
      setNextBeforeSequence(page.next_before_sequence)
    },
    onFetchRawReportBytes: async (reportDigest) => getEvalArtifactBytes(token, reportDigest),
  }

  return {
    data: {
      integration: { status: 'connected' },
      benchmarksLoading,
      benchmarksError,
      benchmarks,
      runsLoading,
      runsError,
      runs,
      runsHasMore,
      runsTruncated,
      activeReportLoading,
      activeReportError,
      activeReport,
      knowledgeRoots,
      knowledgeSearchLoading,
      knowledgeSearchError,
      snapshotPreview,
    },
    actions,
  }
}

interface CaptureDraft {
  readonly selectedBenchmarkDigest: string
  readonly rootId: string
  readonly rootTouched: boolean
  readonly depth: number
  readonly generateHypotheses: boolean
  readonly generateDesigns: boolean
  readonly subjects: readonly CanonicalSubjectRef[]
}

function BenchmarksArea({
  data,
  actions,
  draft,
  onDraftChange,
  onEvaluationSubmitted,
}: {
  readonly data: QualityWorkspaceData
  readonly actions?: QualityWorkspaceActions
  readonly draft: CaptureDraft
  readonly onDraftChange: (updater: (current: CaptureDraft) => CaptureDraft) => void
  readonly onEvaluationSubmitted: () => void
}) {
  const [rootQuery, setRootQuery] = useState('')
  const [draftSubjectKind, setDraftSubjectKind] = useState<SubjectKind>('hypothesis')
  const [draftArtifactSha, setDraftArtifactSha] = useState('')
  const [draftPointer, setDraftPointer] = useState('/proposals/0')
  const [snapshot, setSnapshot] = useState<SnapshotPreviewView | null | undefined>(data.snapshotPreview)
  const [busy, setBusy] = useState<'snapshot' | 'run' | ''>('')
  const [workflowError, setWorkflowError] = useState<QualityActionError | null>(null)
  const [workflowNotice, setWorkflowNotice] = useState('')

  const activeBenchmark =
    data.benchmarks.find((item) => item.digest === draft.selectedBenchmarkDigest) ?? data.benchmarks[0]
  const activeSnapshot = snapshot === undefined ? data.snapshotPreview : snapshot
  const resolvedRootId = draft.rootTouched
    ? draft.rootId
    : draft.rootId || data.snapshotPreview?.rootId || data.knowledgeRoots[0]?.id || ''

  const plannedRequestPreview = useMemo<CanonicalSnapshotRequest | null>(() => {
    const trimmed = resolvedRootId.trim()
    if (!isHexDigest(trimmed)) return null
    return {
      schema: 'aether.quality-snapshot-request/1',
      graph_root_id: trimmed,
      graph_depth: draft.depth,
      subjects: draft.subjects,
      generate_hypotheses: draft.generateHypotheses,
      generate_experiment_designs: draft.generateDesigns,
    }
  }, [resolvedRootId, draft.depth, draft.subjects, draft.generateHypotheses, draft.generateDesigns])

  const plannedBytes = useMemo(
    () => (plannedRequestPreview ? estimateRequestBytes(plannedRequestPreview) : 0),
    [plannedRequestPreview],
  )

  function addSubjectRef() {
    setWorkflowError(null)
    const sha = draftArtifactSha.trim()
    const pointer = draftPointer.trim()
    if (!isHexDigest(sha)) {
      setWorkflowError({
        code: 'invalid_request',
        statusCode: 400,
        message: 'Typed subject artifact SHA-256 must be 64 lowercase hexadecimal characters.',
      })
      return
    }
    if (!pointer.startsWith('/') || pointer.length > 512) {
      setWorkflowError({
        code: 'invalid_request',
        statusCode: 400,
        message: 'Subject JSON pointer must begin with "/" and stay within 512 characters.',
      })
      return
    }
    const candidate: CanonicalSubjectRef = {
      kind: draftSubjectKind,
      artifact: { kind: expectedArtifactKindForSubject(draftSubjectKind), sha256: sha },
      pointer,
    }
    if (draft.subjects.length >= QUALITY_LIMITS.subjectsMax) {
      setWorkflowError({
        code: 'invalid_request',
        statusCode: 400,
        message: `At most ${QUALITY_LIMITS.subjectsMax} typed subject references are allowed per snapshot.`,
      })
      return
    }
    const duplicate = draft.subjects.some(
      (item) => item.kind === candidate.kind && item.artifact.sha256 === candidate.artifact.sha256 && item.pointer === candidate.pointer,
    )
    if (duplicate) {
      setWorkflowError({
        code: 'invalid_request',
        statusCode: 400,
        message: 'Duplicate typed subject reference ignored.',
      })
      return
    }
    onDraftChange((current) => ({ ...current, subjects: [...current.subjects, candidate] }))
    setDraftArtifactSha('')
    setSnapshot(null)
  }

  function removeSubject(index: number) {
    onDraftChange((current) => ({
      ...current,
      subjects: current.subjects.filter((_, itemIndex) => itemIndex !== index),
    }))
    setSnapshot(null)
  }

  async function captureSnapshot() {
    if (!actions?.onCaptureSnapshot) return
    let requestBody: CanonicalSnapshotRequest
    try {
      requestBody = buildSnapshotRequest({
        graphRootId: resolvedRootId,
        graphDepth: draft.depth,
        subjects: draft.subjects,
        generateHypotheses: draft.generateHypotheses,
        generateExperimentDesigns: draft.generateDesigns,
      })
    } catch (reason) {
      setWorkflowError({
        code: 'invalid_request',
        statusCode: 400,
        message: reason instanceof Error ? reason.message : 'Invalid snapshot request.',
      })
      return
    }

    setBusy('snapshot')
    setWorkflowError(null)
    setWorkflowNotice('')
    try {
      const captured = await actions.onCaptureSnapshot(requestBody)
      onDraftChange((current) => ({ ...current, rootId: captured.rootId, rootTouched: true }))
      setSnapshot(captured)
      setWorkflowNotice(
        captured.snapshotDigest
          ? `Immutable snapshot captured (${formatDigest(captured.snapshotDigest)}). Review projection bounds and unresolved references before running.`
          : 'Snapshot preview updated.',
      )
    } catch (reason) {
      setWorkflowError(toActionError(reason))
    } finally {
      setBusy('')
    }
  }

  async function runEvaluation() {
    if (!actions?.onRunEvaluation || !activeBenchmark || !activeSnapshot?.snapshotDigest) return
    let requestBody: CanonicalRunRequest
    try {
      requestBody = buildRunRequest(activeBenchmark.digest, activeSnapshot.snapshotDigest)
    } catch (reason) {
      setWorkflowError({
        code: 'invalid_request',
        statusCode: 400,
        message: reason instanceof Error ? reason.message : 'Invalid run request.',
      })
      return
    }

    setBusy('run')
    setWorkflowError(null)
    setWorkflowNotice('')
    try {
      await actions.onRunEvaluation(requestBody)
      setWorkflowNotice('Evaluation run finished. Switched focus to Runs & Report Overview.')
      onEvaluationSubmitted()
    } catch (reason) {
      setWorkflowError(toActionError(reason))
    } finally {
      setBusy('')
    }
  }

  return (
    <div className="eval-area-stack">
      <div className="dashboard-columns">
        <Panel
          title="Benchmark specifications"
          aside={
            <span className="mono muted">
              {data.benchmarks.length} pinned{data.benchmarkHasMore ? '+' : ''}
            </span>
          }
        >
          <QualityErrorBanner error={data.benchmarksError} />
          {data.benchmarks.length ? (
            <>
              <RecordTable
                label="Available benchmark specifications"
                data={data.benchmarks.map((item) => ({
                  id: item.id,
                  version: item.version,
                  scope: item.scopeLabel,
                  digest: item.digest,
                  rules: item.rules.length,
                }))}
                columns={[
                  'id',
                  'version',
                  'scope',
                  'rules',
                  {
                    key: 'digest',
                    label: 'Benchmark pin',
                    render: (value) => (
                      <button
                        type="button"
                        className="button button--quiet"
                        aria-pressed={activeBenchmark?.digest === value}
                        onClick={() =>
                          onDraftChange((current) => ({ ...current, selectedBenchmarkDigest: String(value) }))
                        }
                      >
                        Inspect {formatDigest(String(value), 8)}
                      </button>
                    ),
                  },
                ]}
              />
              {data.benchmarkHasMore && actions?.onLoadMoreBenchmarks && (
                <div className="toolbar">
                  <button
                    type="button"
                    className="button button--quiet"
                    onClick={() => void actions.onLoadMoreBenchmarks?.()}
                  >
                    Load additional benchmarks
                  </button>
                </div>
              )}
            </>
          ) : (
            <EmptyOrLoading
              loading={data.benchmarksLoading}
              loadingText="Loading pinned benchmark specifications…"
              emptyTitle="No benchmark specifications loaded."
              emptyBody={
                data.integration.status === 'awaiting_canonical_schema'
                  ? 'Benchmark specifications will load once connected to /v1/eval/benchmarks.'
                  : 'No registered benchmarks were returned by the local API.'
              }
            />
          )}
        </Panel>

        <Panel title="Resource limits & excluded metrics">
          {activeBenchmark ? (
            <>
              <Fields
                data={{
                  benchmark_id: activeBenchmark.id,
                  version: activeBenchmark.version,
                  evaluator_version: activeBenchmark.evaluatorVersion,
                  benchmark_digest: activeBenchmark.digest,
                  scope: activeBenchmark.scopeLabel,
                }}
              />
              <RecordTable
                label="Admission and resource ceilings"
                data={activeBenchmark.constraints.map((entry) => ({ ...entry }))}
                columns={['label', 'value', 'detail']}
                empty="No server constraints published."
              />
              <LimitationsCallout
                title="Excluded scientific claims"
                limitations={activeBenchmark.unavailableMetrics}
              />
            </>
          ) : (
            <EmptyOrLoading
              loading={data.benchmarksLoading}
              loadingText="Loading benchmark limits…"
              emptyTitle="Select a benchmark specification to inspect limits."
              emptyBody="Server-enforced body caps (8192-byte POST ceiling), artifact bounds, check budgets, and excluded scientific metrics appear here."
            />
          )}
        </Panel>
      </div>

      {activeBenchmark && (
        <Panel
          title="Benchmark rule catalog"
          aside={<span className="mono muted">{activeBenchmark.rules.length} deterministic rules</span>}
        >
          <RecordTable
            label="Benchmark structural rule catalog"
            data={activeBenchmark.rules.map((rule) => ({
              rule_id: rule.id,
              dimension: rule.dimension,
              version: rule.version,
              title: rule.title,
              empty_policy: rule.emptyCollectionPolicy,
              cannot_establish: rule.cannotEstablish,
            }))}
            columns={['rule_id', 'dimension', 'version', 'title', 'empty_policy', 'cannot_establish']}
            empty="This benchmark specification did not list individual rule entries."
          />
        </Panel>
      )}

      <Panel title="Snapshot capture & two-step evaluation workflow">
        <p className="panel-note">
          Step 1 captures an immutable, text-redacted snapshot projection via <code>POST /v1/eval/snapshots</code>. Step 2 submits the pinned benchmark and snapshot digests to <code>POST /v1/eval/runs</code>.
        </p>
        <div className="eval-workflow-grid">
          <div>
            <h3>Step 1 · Configure snapshot request</h3>
            <div className="modal-form">
              {actions?.onSearchKnowledgeRoots && (
                <div className="eval-inline-controls">
                  <div className="toolbar">
                    <label>
                      Find knowledge graph root
                      <input
                        type="search"
                        value={rootQuery}
                        maxLength={128}
                        onChange={(event) => setRootQuery(event.target.value)}
                        placeholder="Title prefix or digest"
                      />
                    </label>
                    <button
                      type="button"
                      className="button button--quiet"
                      onClick={() => void actions.onSearchKnowledgeRoots?.(rootQuery, '')}
                    >
                      {data.knowledgeSearchLoading ? 'Searching…' : 'Search graph roots'}
                    </button>
                  </div>
                  <QualityErrorBanner error={data.knowledgeSearchError} />
                  {data.knowledgeRoots.length > 0 && (
                    <RecordTable
                      label="Available knowledge graph roots"
                      data={data.knowledgeRoots.slice(0, 8).map((node) => ({
                        id: node.id,
                        kind: node.kind,
                        label: node.label,
                        status: node.status ?? '—',
                      }))}
                      columns={[
                        {
                          key: 'id',
                          label: 'Use root',
                          render: (value, row) => (
                            <button
                              type="button"
                              className="button button--quiet"
                              aria-pressed={resolvedRootId === value}
                              onClick={() => {
                                onDraftChange((current) => ({ ...current, rootTouched: true, rootId: String(value) }))
                                setSnapshot(null)
                              }}
                            >
                              {text(row.label)}
                            </button>
                          ),
                        },
                        'kind',
                        'status',
                      ]}
                    />
                  )}
                </div>
              )}

              <label>
                Knowledge graph root ID (64-character lowercase hex)
                <input
                  value={resolvedRootId}
                  onChange={(event) => {
                    const nextRoot = event.target.value
                    onDraftChange((current) => ({ ...current, rootTouched: true, rootId: nextRoot }))
                    setSnapshot(null)
                  }}
                  placeholder="64 lowercase hex characters"
                  disabled={!actions?.onCaptureSnapshot}
                />
              </label>

              <label>
                Traversal depth (0–3; generated subjects require depth = 2)
                <select
                  value={String(draft.depth)}
                  onChange={(event) => {
                    const nextDepth = Number(event.target.value)
                    onDraftChange((current) => ({ ...current, depth: nextDepth }))
                    setSnapshot(null)
                  }}
                  disabled={!actions?.onCaptureSnapshot}
                >
                  {['0', '1', '2', '3'].map((value) => (
                    <option key={value} value={value}>
                      Depth {value}
                    </option>
                  ))}
                </select>
              </label>

              <fieldset className="eval-bundle-fieldset">
                <legend>Snapshot subject options</legend>
                <label className="eval-bundle-option">
                  <input
                    type="checkbox"
                    checked={draft.generateHypotheses}
                    onChange={(event) => {
                      const next = event.target.checked
                      onDraftChange((current) => ({
                        ...current,
                        generateHypotheses: next,
                        depth: next ? 2 : current.depth,
                      }))
                      setSnapshot(null)
                    }}
                    disabled={!actions?.onCaptureSnapshot}
                  />
                  <span>Generate APO-03 hypothesis proposals from root (requires depth = 2)</span>
                </label>
                <label className="eval-bundle-option">
                  <input
                    type="checkbox"
                    checked={draft.generateDesigns}
                    onChange={(event) => {
                      const next = event.target.checked
                      onDraftChange((current) => ({
                        ...current,
                        generateDesigns: next,
                        depth: next ? 2 : current.depth,
                      }))
                      setSnapshot(null)
                    }}
                    disabled={!actions?.onCaptureSnapshot}
                  />
                  <span>Generate APO-04 experiment designs from root (requires depth = 2)</span>
                </label>
              </fieldset>

              <fieldset className="eval-bundle-fieldset">
                <legend>
                  Explicit typed subject references ({draft.subjects.length}/{QUALITY_LIMITS.subjectsMax})
                </legend>
                <div className="eval-subject-row">
                  <label>
                    Subject kind
                    <select
                      value={draftSubjectKind}
                      onChange={(event) => {
                        const next = event.target.value as SubjectKind
                        setDraftSubjectKind(next)
                        setDraftPointer(next === 'hypothesis' ? '/proposals/0' : '/designs/0')
                      }}
                      disabled={!actions?.onCaptureSnapshot}
                    >
                      <option value="hypothesis">hypothesis (hypothesis_report)</option>
                      <option value="experiment_design">experiment_design (experiment_design_report)</option>
                    </select>
                  </label>
                  <label>
                    JSON pointer
                    <input
                      value={draftPointer}
                      onChange={(event) => setDraftPointer(event.target.value)}
                      placeholder="/proposals/0"
                      disabled={!actions?.onCaptureSnapshot}
                    />
                  </label>
                </div>
                <div className="toolbar">
                  <label>
                    Artifact SHA-256 (64 hex)
                    <input
                      value={draftArtifactSha}
                      onChange={(event) => setDraftArtifactSha(event.target.value)}
                      placeholder="64 lowercase hex digest"
                      disabled={!actions?.onCaptureSnapshot}
                    />
                  </label>
                  <button
                    type="button"
                    className="button button--quiet"
                    disabled={!actions?.onCaptureSnapshot}
                    onClick={addSubjectRef}
                  >
                    Add subject ref
                  </button>
                </div>
                {draft.subjects.length > 0 && (
                  <div className="eval-area-stack">
                    {draft.subjects.map((ref, index) => (
                      <div key={`${ref.kind}-${ref.artifact.sha256}-${ref.pointer}`} className="eval-subject-chip">
                        <span>
                          <strong>{ref.kind}</strong> · {ref.artifact.kind} ·{' '}
                          <code>{formatDigest(ref.artifact.sha256, 12)}</code> · <code>{ref.pointer}</code>
                        </span>
                        <button
                          type="button"
                          className="button button--quiet"
                          onClick={() => removeSubject(index)}
                        >
                          Remove
                        </button>
                      </div>
                    ))}
                  </div>
                )}
              </fieldset>

              <div className="eval-preview-box">
                <strong>Canonical request preview</strong>
                <p className="muted">
                  Benchmark: {activeBenchmark ? `${activeBenchmark.id} v${activeBenchmark.version}` : 'None selected'} ·
                  Root: <code>{formatDigest(resolvedRootId.trim(), 16)}</code> (depth {draft.depth}) · Request size:{' '}
                  {plannedBytes}/{QUALITY_LIMITS.requestBytes} bytes
                </p>
              </div>
              <div className="toolbar">
                <button
                  type="button"
                  className="button button--primary"
                  disabled={busy !== '' || !actions?.onCaptureSnapshot}
                  onClick={() => void captureSnapshot()}
                >
                  {busy === 'snapshot' ? 'Capturing immutable snapshot…' : 'Capture snapshot'}
                </button>
                <button
                  type="button"
                  className="button"
                  disabled={
                    busy !== '' ||
                    !actions?.onRunEvaluation ||
                    !activeBenchmark ||
                    !activeSnapshot?.snapshotDigest
                  }
                  onClick={() => void runEvaluation()}
                >
                  {busy === 'run' ? 'Running structural evaluation…' : 'Run evaluation against snapshot'}
                </button>
              </div>
            </div>
          </div>

          <div>
            <h3>Step 2 · Captured snapshot receipt &amp; projection</h3>
            {workflowNotice && (
              <p className="notice" role="status">
                {workflowNotice}
              </p>
            )}
            <QualityErrorBanner error={workflowError} />
            {activeSnapshot ? (
              <>
                {activeSnapshot.isStaleRevision && (
                  <p className="error" role="alert">
                    Source revisions changed during capture; snapshot was rejected to protect immutability. Re-check graph state before capturing again.
                  </p>
                )}
                <Fields
                  data={{
                    snapshot_digest: activeSnapshot.snapshotDigest ?? 'Not finalized',
                    graph_root: activeSnapshot.rootId,
                    graph_depth: activeSnapshot.depth ?? 'Not loaded',
                    graph_digest: activeSnapshot.graphDigest,
                    graph_truncated: activeSnapshot.truncated ?? 'Not loaded',
                    source_text_included: activeSnapshot.sourceTextIncluded,
                    projected_nodes: activeSnapshot.nodeCount ?? '—',
                    projected_edges: activeSnapshot.edgeCount ?? '—',
                    projected_subjects: activeSnapshot.subjectCount ?? activeSnapshot.resolvedArtifactsCount,
                    unresolved_notes: activeSnapshot.unresolvedItems.length,
                    replay_status: activeSnapshot.replayStatusLabel ?? 'Unavailable — not provided',
                  }}
                />
                {activeSnapshot.unresolvedItems.length > 0 && (
                  <RecordTable
                    label="Missing or unresolved snapshot references"
                    data={activeSnapshot.unresolvedItems.map((item) => ({
                      reference_digest: item.referenceDigest,
                      reason: item.reason,
                      legacy_note: item.legacyNote ?? 'Recorded as missing/legacy; affected checks remain unavailable.',
                    }))}
                    columns={['reference_digest', 'reason', 'legacy_note']}
                  />
                )}
                {activeSnapshot.notes.length > 0 && (
                  <LimitationsCallout title="Snapshot boundary notes" limitations={activeSnapshot.notes} />
                )}
              </>
            ) : (
              <EmptyOrLoading
                loading={busy === 'snapshot'}
                loadingText="Capturing immutable graph & artifact snapshot…"
                emptyTitle="No snapshot captured yet."
                emptyBody="Select or enter a knowledge graph root ID, capture an immutable snapshot first, inspect its digest and redacted projection counts, and then trigger the evaluator."
              />
            )}
          </div>
        </div>
      </Panel>
    </div>
  )
}

function OverviewArea({
  data,
  actions,
  onInspectDimension,
}: {
  readonly data: QualityWorkspaceData
  readonly actions?: QualityWorkspaceActions
  readonly onInspectDimension: (dimension: QualityDimensionId) => void
}) {
  const report = data.activeReport
  const dimensions = sortDimensionsNonPassFirst(report?.dimensions ?? defaultDimensionsUnavailable())

  return (
    <div className="eval-area-stack">
      <Panel
        title="Evaluation run history (newest first)"
        aside={
          <span className="mono muted">
            {data.runs.length} recorded{data.runsHasMore ? '+' : ''}
          </span>
        }
      >
        <QualityErrorBanner error={data.runsError} />
        {data.runsTruncated && (
          <p className="panel-note" role="status">
            Showing the newest bounded registry page. Use pagination below to load earlier runs without reordering existing entries.
          </p>
        )}
        {data.runs.length ? (
          <>
            <RecordTable
              label="Evaluation run history"
              data={data.runs.map((run) => ({
                sequence: run.sequence ?? '—',
                benchmark: `${run.benchmarkTitle} ${run.benchmarkVersion}`,
                execution: run.executionStatus,
                fail_dims: run.dimensions.filter((dim) => dim.status === 'fail').length,
                unavailable_dims: run.dimensions.filter((dim) => dim.status === 'unavailable').length,
                snapshot: run.snapshotDigest,
                report_digest: run.reportDigest,
              }))}
              columns={[
                'sequence',
                'benchmark',
                {
                  key: 'execution',
                  label: 'Execution status',
                  render: (value) => <ExecutionOutcomeBadge status={value as QualityReportDetailView['executionStatus']} />,
                },
                {
                  key: 'fail_dims',
                  label: 'Fail dims',
                  render: (value) => <span className="mono">{text(value)}</span>,
                },
                {
                  key: 'unavailable_dims',
                  label: 'Unavailable dims',
                  render: (value) => <span className="mono">{text(value)}</span>,
                },
                'snapshot',
                {
                  key: 'report_digest',
                  label: 'Inspect report',
                  render: (value) => {
                    const digest = String(value)
                    return (
                      <button
                        type="button"
                        className="button button--quiet"
                        aria-pressed={report?.reportDigest === digest}
                        disabled={!actions?.onSelectRun}
                        onClick={() => void actions?.onSelectRun?.(digest)}
                      >
                        {formatDigest(digest, 10)}
                      </button>
                    )
                  },
                },
              ]}
            />
            {data.runsHasMore && actions?.onLoadMoreRuns && (
              <div className="toolbar">
                <button
                  type="button"
                  className="button button--quiet"
                  onClick={() => void actions.onLoadMoreRuns?.()}
                >
                  Load earlier evaluation runs
                </button>
              </div>
            )}
          </>
        ) : (
          <EmptyOrLoading
            loading={data.runsLoading}
            loadingText="Loading newest-first evaluation run history…"
            emptyTitle="No structural evaluation reports recorded yet."
            emptyBody="Use Benchmarks & Run Capture to capture an immutable research snapshot and start a structural evaluation run."
          />
        )}
      </Panel>

      <QualityErrorBanner error={data.activeReportError} />
      {data.activeReportLoading ? (
        <Panel title="Active report overview">
          <p className="loading" role="status">
            Loading evaluation report details…
          </p>
        </Panel>
      ) : (
        <div className="dashboard-columns">
          <Panel
            title="Active run summary"
            aside={report ? <ExecutionOutcomeBadge status={report.executionStatus} /> : undefined}
          >
            {report ? (
              <>
                <p className="panel-note">
                  Execution status describes whether the evaluator completed running its deterministic rules; it is never an aggregate research-quality score.
                </p>
                <Fields
                  data={{
                    report_digest: report.reportDigest,
                    sequence: report.sequence ?? '—',
                    benchmark: `${report.benchmarkLabel} (${formatDigest(report.benchmarkDigest)})`,
                    input_snapshot: report.snapshotDigest,
                    graph_root: report.graphRootId,
                    human_review: report.humanReviewLabel,
                    independent_scientific_validation: report.independentValidationLabel,
                  }}
                />
              </>
            ) : (
              <EmptyOrLoading
                loading={false}
                loadingText=""
                emptyTitle="No active evaluation report selected."
                emptyBody="Each structural dimension remains unavailable until an evaluation report is selected or generated."
              />
            )}
          </Panel>

          <Panel title="Scientific scope & explicit limitations">
            <LimitationsCallout
              limitations={
                report?.limitations ?? [
                  'No evaluation report loaded; structural completeness checks have not run.',
                  'Automated structural checks do not measure scientific truth, novelty, statistical power, or source credibility.',
                  'Human semantic review writes are excluded in Slice 1; independent scientific validation remains not performed.',
                ]
              }
            />
          </Panel>
        </div>
      )}

      <Panel
        title="Eight structural dimensions (non-pass sorted first · no composite score)"
        aside={<span className="muted">Categorical status per dimension only</span>}
      >
        <div className="eval-dimension-grid">
          {dimensions.map((dimension) => {
            const descriptor = QUALITY_DIMENSIONS.find((item) => item.id === dimension.dimension)
            return (
              <article key={dimension.dimension} className="eval-dimension-card">
                <div className="eval-dimension-head">
                  <h3>{dimension.title}</h3>
                  <StructuralOutcomeBadge status={dimension.status} />
                </div>
                <p className="eval-dimension-counts mono">
                  Fail: {dimension.failCount} · Unavail: {dimension.unavailableCount} · Structural pass:{' '}
                  {dimension.passCount}
                </p>
                <p className="muted">{descriptor?.structuralScope}</p>
                <p className="eval-dimension-limit">{dimension.semanticLabel}</p>
                {dimension.limitations.length > 0 && (
                  <p className="eval-dimension-limit">Limit: {dimension.limitations.join('; ')}</p>
                )}
                <button
                  type="button"
                  className="button button--quiet"
                  onClick={() => onInspectDimension(dimension.dimension)}
                >
                  Inspect dimension checks
                </button>
              </article>
            )
          })}
        </div>
      </Panel>
    </div>
  )
}

function DiagnosticsArea({
  report,
  selectedDimension,
  onDimensionChange,
}: {
  readonly report: QualityReportDetailView | null | undefined
  readonly selectedDimension: QualityDimensionId | 'all'
  readonly onDimensionChange: (dimension: QualityDimensionId | 'all') => void
}) {
  const [statusFilter, setStatusFilter] = useState<'all' | StructuralOutcome>('all')
  const [query, setQuery] = useState('')
  const [page, setPage] = useState(1)
  const [selectedCheckId, setSelectedCheckId] = useState('')

  const filteredChecks = useMemo(() => {
    const all = report?.checks ?? []
    const needle = query.trim().toLowerCase()
    const filtered = all.filter((check) => {
      if (selectedDimension !== 'all' && check.dimension !== selectedDimension) return false
      if (statusFilter !== 'all' && check.status !== statusFilter) return false
      if (
        needle &&
        !`${check.id} ${check.reasonLabel} ${check.reasonDetail}`.toLowerCase().includes(needle)
      ) {
        return false
      }
      return true
    })
    return sortChecksNonPassFirst(filtered)
  }, [report?.checks, selectedDimension, statusFilter, query])

  const paginated = useMemo(
    () => paginateItems(filteredChecks, page, DEFAULT_CHECK_PAGE_SIZE),
    [filteredChecks, page],
  )

  const selectedCheck =
    paginated.slice.find((check) => check.id === selectedCheckId) ?? paginated.slice[0]
  const evidenceById = useMemo(
    () => new Map((report?.evidence ?? []).map((item) => [item.id, item])),
    [report?.evidence],
  )
  const relatedEvidence = useMemo(() => {
    if (!selectedCheck) return []
    return selectedCheck.evidenceIds
      .map((id) => evidenceById.get(id))
      .filter((item): item is NonNullable<typeof item> => item !== undefined)
  }, [selectedCheck, evidenceById])

  const scopedEvidence = useMemo(() => {
    if (!report) return []
    if (selectedDimension === 'all') return report.evidence
    const ids = new Set(
      report.checks
        .filter((check) => check.dimension === selectedDimension)
        .flatMap((check) => check.evidenceIds),
    )
    return ids.size > 0
      ? report.evidence.filter((item) => ids.has(item.id))
      : selectedDimension === 'counterevidence_edge_integrity'
        ? report.evidence.filter((item) => item.polarity === 'contradictory')
        : report.evidence
  }, [report, selectedDimension])

  const polarity = useMemo(() => partitionEvidence(scopedEvidence), [scopedEvidence])
  const activeDescriptor =
    selectedDimension === 'all'
      ? undefined
      : QUALITY_DIMENSIONS.find((entry) => entry.id === selectedDimension)

  return (
    <div className="eval-area-stack">
      <Panel title="Dimension & check filters">
        <form
          className="toolbar"
          onSubmit={(event) => {
            event.preventDefault()
            setPage(1)
          }}
        >
          <label>
            Evaluation dimension
            <select
              value={selectedDimension}
              onChange={(event) => {
                onDimensionChange(event.target.value as QualityDimensionId | 'all')
                setPage(1)
              }}
            >
              <option value="all">All 8 structural dimensions</option>
              {QUALITY_DIMENSIONS.map((entry) => (
                <option key={entry.id} value={entry.id}>
                  {entry.title}
                </option>
              ))}
            </select>
          </label>
          <label>
            Structural outcome
            <select
              value={statusFilter}
              onChange={(event) => {
                setStatusFilter(event.target.value as 'all' | StructuralOutcome)
                setPage(1)
              }}
            >
              <option value="all">All outcomes (non-pass sorted first)</option>
              <option value="fail">Structural fail only</option>
              <option value="unavailable">Unavailable only</option>
              <option value="pass">Structural pass only</option>
            </select>
          </label>
          <label>
            Filter by rule ID or reason code
            <input
              type="search"
              value={query}
              maxLength={80}
              onChange={(event) => {
                setQuery(event.target.value)
                setPage(1)
              }}
              placeholder="e.g. legacy_locator_unavailable, required_collection_empty"
            />
          </label>
        </form>
        {activeDescriptor && (
          <p className="panel-note">
            <strong>{activeDescriptor.title}:</strong> {activeDescriptor.structuralScope} <em>Cannot establish:</em>{' '}
            {activeDescriptor.cannotEstablish}
          </p>
        )}
      </Panel>

      <div className="graph-grid">
        <Panel
          title="Structural checks"
          aside={
            <span className="mono muted">
              Showing {paginated.startIndex}–{paginated.endIndex} of {paginated.totalItems} checks (50/page cap)
            </span>
          }
        >
          {paginated.slice.length ? (
            <>
              <RecordTable
                label="Dimension structural checks"
                data={paginated.slice.map((check) => ({
                  id: check.id,
                  dimension: QUALITY_DIMENSIONS.find((item) => item.id === check.dimension)?.title ?? check.dimension,
                  status: check.status,
                  reason: check.reasonLabel,
                  subject: check.subjectKind
                    ? `${check.subjectKind} · ${formatDigest(check.subjectDigest, 8)}`
                    : formatDigest(check.subjectDigest, 8),
                }))}
                columns={[
                  {
                    key: 'id',
                    label: 'Check rule',
                    render: (value) => (
                      <button
                        type="button"
                        className="button button--quiet"
                        aria-pressed={selectedCheck?.id === value}
                        onClick={() => setSelectedCheckId(String(value))}
                      >
                        {String(value)}
                      </button>
                    ),
                  },
                  'dimension',
                  {
                    key: 'status',
                    label: 'Structural status',
                    render: (value) => <StructuralOutcomeBadge status={value as StructuralOutcome} />,
                  },
                  'reason',
                  'subject',
                ]}
              />
              {paginated.totalPages > 1 && (
                <nav className="eval-pagination" aria-label="Check pagination">
                  <button
                    type="button"
                    className="button button--quiet"
                    disabled={paginated.page <= 1}
                    onClick={() => setPage((current) => Math.max(1, current - 1))}
                  >
                    Previous page
                  </button>
                  <span className="mono muted">
                    Page {paginated.page} of {paginated.totalPages}
                  </span>
                  <button
                    type="button"
                    className="button button--quiet"
                    disabled={paginated.page >= paginated.totalPages}
                    onClick={() => setPage((current) => Math.min(paginated.totalPages, current + 1))}
                  >
                    Next page
                  </button>
                </nav>
              )}
            </>
          ) : (
            <EmptyOrLoading
              loading={false}
              loadingText=""
              emptyTitle={
                report ? 'No structural checks matched these filters.' : 'No evaluation report selected for diagnostics.'
              }
              emptyBody={
                report
                  ? 'Clear or widen the dimension, status, or keyword filters above.'
                  : 'Select an evaluation report in Runs & Report Overview or capture a snapshot to inspect structural checks across all eight dimensions.'
              }
            />
          )}
        </Panel>

        <Panel title="Check, failure & provenance inspector">
          {selectedCheck ? (
            <>
              <div className="eval-inspector-header">
                <h3>{selectedCheck.id}</h3>
                <StructuralOutcomeBadge status={selectedCheck.status} />
              </div>
              <Fields
                data={{
                  rule_version: selectedCheck.ruleVersion,
                  status: describeStructuralStatus(selectedCheck.status),
                  reason_codes: selectedCheck.reasonLabel || 'None (structural predicate satisfied)',
                  explanation: selectedCheck.reasonDetail,
                  subject_kind: selectedCheck.subjectKind ?? 'Not recorded',
                  subject_digest: selectedCheck.subjectDigest ?? 'Not recorded',
                  subject_pointer: selectedCheck.subjectPointer ?? '—',
                }}
              />
              <h4>Linked evidence provenance ({relatedEvidence.length})</h4>
              {relatedEvidence.length ? (
                relatedEvidence.map((item) => (
                  <details key={item.id} open>
                    <summary>
                      {item.relationLabel} · {item.canonicalSource ?? 'Legacy / unattributed source'} ·{' '}
                      {item.integrityStatus}
                    </summary>
                    <div className="eval-evidence-badge-row">
                      <IntegrityOutcomeBadge status={item.integrityStatus} />
                      <span className="mono muted">Ref {formatDigest(item.id, 10)}</span>
                    </div>
                    <Fields
                      data={{
                        relation: item.relationLabel,
                        claim_ref: item.claimRef ?? 'Not recorded',
                        canonical_source: item.canonicalSource ?? 'Unavailable',
                        observed_aliases: item.observedAliases.length ? item.observedAliases.join(', ') : 'None recorded',
                        node_kind: item.sourceGroup ?? 'Not recorded',
                        source_url: item.sourceUrl ?? 'Not recorded',
                        source_bytes_digest: item.sourceDigest ?? 'Unavailable — raw source bytes not archived',
                        passage_locator: item.passageLocatorLabel ?? 'Unavailable — no legacy passage span recorded',
                        revision_sha256: item.representationDigest ?? 'Unavailable',
                        statement_sha256: item.passageDigest ?? 'Unavailable (redacted or empty)',
                        provenance_origin: item.provenanceSchema ?? 'Unrecorded (null)',
                        source_identity_status: item.sourceIdentityStatus ?? 'Unrecorded (null)',
                        epistemic_status: item.epistemicOriginLabel ?? 'Unrecorded (null)',
                        campaign_verification:
                          item.campaignVerification ?? 'Unverified — no resolved campaign protocol receipt',
                        integrity_reason: item.integrityReason ?? 'None',
                      }}
                    />
                  </details>
                ))
              ) : (
                <p className="panel-note">
                  No snapshot evidence references were attached to this check. All snapshot source prose remains excluded.
                </p>
              )}
            </>
          ) : (
            <EmptyOrLoading
              loading={false}
              loadingText=""
              emptyTitle="Select a check row to inspect details."
              emptyBody="Failure reason codes, unavailable explanations, source metadata, passage byte ranges, and legacy missing-provenance notices render safely as plain text here."
            />
          )}
        </Panel>
      </div>

      <div className="dashboard-columns">
        <Panel
          title="Supporting evidence relationships"
          aside={<span className="mono muted">{polarity.supporting.length} linked</span>}
        >
          <RecordTable
            label="Supporting evidence relationships"
            data={polarity.supporting.map((item) => ({
              reference: formatDigest(item.id, 10),
              relation: item.relationLabel,
              source: item.canonicalSource ?? 'Legacy source (unattributed)',
              passage: item.passageLocatorLabel ?? 'Not recorded',
              integrity: item.integrityStatus,
              verification: item.campaignVerification ?? 'Unverified',
            }))}
            columns={['reference', 'relation', 'source', 'passage', 'integrity', 'verification']}
            empty="No supporting evidence edges linked to the active selection."
          />
        </Panel>

        <Panel
          title="Contradictory & opposing evidence"
          aside={<span className="mono muted">{polarity.contradictory.length} linked</span>}
        >
          {polarity.contradictory.length === 0 && (
            <p className="notice" role="status">
              No contradictory evidence was recorded in the selected evidence set. Absence of recorded contradiction links is not proof that conflicting literature or counterevidence does not exist.
            </p>
          )}
          <RecordTable
            label="Contradictory evidence relationships"
            data={polarity.contradictory.map((item) => ({
              reference: formatDigest(item.id, 10),
              relation: item.relationLabel,
              source: item.canonicalSource ?? 'Legacy source (unattributed)',
              passage: item.passageLocatorLabel ?? 'Not recorded',
              integrity: item.integrityStatus,
              reason: item.integrityReason ?? '—',
            }))}
            columns={['reference', 'relation', 'source', 'passage', 'integrity', 'reason']}
            empty="No contradicting evidence entries present."
          />
        </Panel>
      </div>

      <Panel
        title="Contextual relationships and unlinked snapshot evidence"
        aside={<span className="mono muted">{polarity.contextual.length} recorded</span>}
      >
        <RecordTable
          label="Contextual relationships and unlinked snapshot evidence"
          data={polarity.contextual.map((item) => ({
            reference: formatDigest(item.id, 10),
            relation: item.relationLabel,
            source: item.canonicalSource ?? 'Legacy / unattributed source',
            origin: item.provenanceSchema ?? 'Unrecorded (null)',
            integrity: item.integrityStatus,
            reason: item.integrityReason ?? '—',
          }))}
          columns={['reference', 'relation', 'source', 'origin', 'integrity', 'reason']}
          empty="No contextual graph edges or unlinked evidence nodes were present in this snapshot."
        />
      </Panel>
    </div>
  )
}

function IntegrityArea({
  report,
  actions,
}: {
  readonly report: QualityReportDetailView | null | undefined
  readonly actions?: QualityWorkspaceActions
}) {
  const [verification, setVerification] = useState<DigestVerificationResult>()
  const [busy, setBusy] = useState(false)
  const [rawBytes, setRawBytes] = useState<Uint8Array | undefined>(report?.rawArtifactBytes)
  const [integrityError, setIntegrityError] = useState('')

  const effectiveBytes = rawBytes ?? report?.rawArtifactBytes

  async function verifyDigest() {
    if (!report) return
    setBusy(true)
    setIntegrityError('')
    try {
      let bytes = effectiveBytes
      if (!bytes && actions?.onFetchRawReportBytes) {
        bytes = await actions.onFetchRawReportBytes(report.reportDigest)
        setRawBytes(bytes)
      }
      if (!bytes) {
        setVerification({
          matched: false,
          computedSha256: '',
          byteLength: 0,
          error:
            'Original raw artifact bytes are unavailable; the UI refuses to reconstruct a digest by serializing a parsed JSON object.',
        })
        return
      }
      const outcome = await verifyRawArtifactDigest(bytes, report.reportDigest)
      setVerification(outcome)
    } catch (reason) {
      setIntegrityError(formatQualityError(toActionError(reason)))
    } finally {
      setBusy(false)
    }
  }

  function exportVerifiedRawBytes() {
    if (!report || !effectiveBytes || !verification?.matched) return
    const payload = new Uint8Array(effectiveBytes.byteLength)
    payload.set(effectiveBytes)
    const blob = new Blob([payload], { type: 'application/json' })
    const url = URL.createObjectURL(blob)
    const link = document.createElement('a')
    link.href = url
    link.download = `quality-report-${report.reportDigest.slice(0, 12)}.json`
    link.click()
    window.setTimeout(() => URL.revokeObjectURL(url), 500)
  }

  return (
    <div className="eval-area-stack">
      <div className="dashboard-columns">
        <Panel title="Cryptographic digests & pinned identities">
          {report ? (
            <Fields
              data={{
                report_digest: report.reportDigest,
                sequence: report.sequence ?? '—',
                snapshot_identity: report.snapshotDigest,
                benchmark_identity: `${report.benchmarkLabel} (${report.benchmarkDigest})`,
                graph_root_id: report.graphRootId,
                graph_sha256: report.graphSha256,
                serialization_profile: report.serializationLabel,
              }}
            />
          ) : (
            <EmptyOrLoading
              loading={false}
              loadingText=""
              emptyTitle="No report digests loaded."
              emptyBody="Select an evaluation run or capture a snapshot to inspect pinned report, snapshot, benchmark, and graph digests."
            />
          )}
        </Panel>

        <Panel title="Evaluator identity & reproducibility boundaries">
          {report ? (
            <>
              <Fields
                data={{
                  evaluator_version: report.evaluatorVersion,
                  evaluator_binary_digest:
                    report.evaluatorBinaryDigest ?? 'Unavailable — binary digest omitted',
                  build_attestation: report.evaluatorBuildAttestation,
                  replay_status: report.replaySummary,
                }}
              />
              <LimitationsCallout
                title="Evaluator & reproducibility limits"
                limitations={report.evaluatorLimitations}
              />
            </>
          ) : (
            <EmptyOrLoading
              loading={false}
              loadingText=""
              emptyTitle="Evaluator build details unavailable."
              emptyBody="Evaluator version pins, build attestation status, and fresh-process replay status appear here without overclaiming release trust."
            />
          )}
        </Panel>
      </div>

      {report?.snapshotMetadata && (
        <Panel title="Snapshot audit metadata">
          <Fields
            data={{
              snapshot_digest: report.snapshotMetadata.snapshotDigest ?? report.snapshotDigest,
              graph_root_id: report.snapshotMetadata.rootId,
              graph_depth: report.snapshotMetadata.depth ?? 'Not recorded',
              graph_truncated: report.snapshotMetadata.truncated ?? 'Not recorded',
              source_text_included: report.snapshotMetadata.sourceTextIncluded,
              projected_nodes: report.snapshotMetadata.nodeCount ?? '—',
              projected_edges: report.snapshotMetadata.edgeCount ?? '—',
              projected_subjects: report.snapshotMetadata.subjectCount ?? report.snapshotMetadata.resolvedArtifactsCount,
              unresolved_notes: report.snapshotMetadata.unresolvedItems.length,
            }}
          />
        </Panel>
      )}

      <Panel title="Raw-byte Web Crypto digest verification & safe export">
        <p className="panel-note">
          Verification hashes only exact raw <code>aether.quality-report/1</code> bytes fetched from content-addressed storage (<code>GET /v1/artifacts/&#123;sha256&#125;</code>) via <code>crypto.subtle.digest</code>. Parsed JSON is never re-serialized to approximate a hash.
        </p>
        {report ? (
          <>
            <div className="toolbar">
              <button
                type="button"
                className="button button--primary"
                disabled={busy}
                onClick={() => void verifyDigest()}
              >
                {busy ? 'Computing SHA-256 over raw bytes…' : 'Verify raw artifact bytes (Web Crypto)'}
              </button>
              <button
                type="button"
                className="button"
                disabled={!verification?.matched || !effectiveBytes}
                onClick={exportVerifiedRawBytes}
              >
                Download verified report JSON
              </button>
            </div>
            {integrityError && <p className="error" role="alert">{integrityError}</p>}
            {verification && (
              <div className={verification.matched ? 'notice' : 'error'} role={verification.matched ? 'status' : 'alert'}>
                {verification.matched ? (
                  <p>
                    Exact raw-byte SHA-256 verified ({verification.byteLength} bytes):{' '}
                    <code className="hash-block">{verification.computedSha256}</code>
                  </p>
                ) : (
                  <p>
                    {verification.error ?? 'Digest verification failed.'}{' '}
                    {verification.computedSha256 && <code>Computed: {verification.computedSha256}</code>}
                  </p>
                )}
              </div>
            )}
          </>
        ) : (
          <EmptyOrLoading
            loading={false}
            loadingText=""
            emptyTitle="Select a persisted evaluation report to verify raw bytes."
            emptyBody="Once a report is selected, Web Crypto verification confirms byte-level equality prior to JSON export."
          />
        )}
      </Panel>
    </div>
  )
}

export function QualityEvaluationWorkspace({
  token,
  data,
  actions,
  integration,
  initialArea = 'benchmarks',
}: QualityEvaluationProps) {
  const [activeArea, setActiveArea] = useState<QualityAreaId>(initialArea)
  const [selectedDimension, setSelectedDimension] = useState<QualityDimensionId | 'all'>('all')

  const live = useLiveQualityWorkspace(data ? undefined : token)
  const workspaceData: QualityWorkspaceData = data ??
    live.data ?? {
      integration: integration ?? DEFAULT_INTEGRATION,
      benchmarks: [],
      runs: [],
      knowledgeRoots: [],
      activeReport: null,
    }
  const [captureDraft, setCaptureDraft] = useState<CaptureDraft>(() => ({
    selectedBenchmarkDigest: workspaceData.benchmarks[0]?.digest ?? '',
    rootId: workspaceData.snapshotPreview?.rootId ?? '',
    rootTouched: Boolean(workspaceData.snapshotPreview?.rootId),
    depth: workspaceData.snapshotPreview?.depth ?? 2,
    generateHypotheses: false,
    generateDesigns: false,
    subjects: [],
  }))
  const resolvedActions = actions ?? live.actions
  const resolvedIntegration = integration ?? workspaceData.integration
  const currentAreaMeta = QUALITY_AREAS.find((entry) => entry.id === activeArea)

  function inspectDimension(dimension: QualityDimensionId) {
    setSelectedDimension(dimension)
    setActiveArea('diagnostics')
  }

  return (
    <div className="eval-workspace">
      <section className="mission-lead compact-lead">
        <div>
          <p className="eyebrow">Research quality evaluation / Structural slice 1</p>
          <h2>Structural completeness, not scientific truth.</h2>
          <p>
            Deterministic checks verify schema structure, immutable digests, typed locators, and budget arithmetic across eight research dimensions without computing an aggregate quality score.
          </p>
        </div>
        <div className="mission-current">
          <span>Active workspace area</span>
          <strong>{currentAreaMeta?.label}</strong>
          <small>{currentAreaMeta?.summary}</small>
        </div>
      </section>

      <EpistemicBoundaryStrip
        humanReviewLabel={workspaceData.activeReport?.humanReviewLabel}
        independentValidationLabel={workspaceData.activeReport?.independentValidationLabel}
      />

      <IntegrationStatusNotice integration={resolvedIntegration} />

      <QualityAreaTabs activeArea={activeArea} onSelect={setActiveArea} />

      <div
        id={`eval-panel-${activeArea}`}
        role="tabpanel"
        aria-labelledby={`eval-tab-${activeArea}`}
        className="eval-tabpanel"
      >
        {activeArea === 'benchmarks' && (
          <BenchmarksArea
            data={workspaceData}
            actions={resolvedActions}
            draft={captureDraft}
            onDraftChange={setCaptureDraft}
            onEvaluationSubmitted={() => setActiveArea('overview')}
          />
        )}
        {activeArea === 'overview' && (
          <OverviewArea
            data={workspaceData}
            actions={resolvedActions}
            onInspectDimension={inspectDimension}
          />
        )}
        {activeArea === 'diagnostics' && (
          <DiagnosticsArea
            key={workspaceData.activeReport?.reportDigest ?? 'none'}
            report={workspaceData.activeReport}
            selectedDimension={selectedDimension}
            onDimensionChange={setSelectedDimension}
          />
        )}
        {activeArea === 'integrity' && (
          <IntegrityArea
            key={workspaceData.activeReport?.reportDigest ?? 'none'}
            report={workspaceData.activeReport}
            actions={resolvedActions}
          />
        )}
      </div>
    </div>
  )
}
