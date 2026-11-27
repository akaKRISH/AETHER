import { strict as assert } from 'node:assert'
import { existsSync, readFileSync } from 'node:fs'
import { dirname, resolve } from 'node:path'
import { test } from 'node:test'
import { fileURLToPath } from 'node:url'
import {
  DEFAULT_CHECK_PAGE_SIZE,
  DEFAULT_REPORT_PAGE_SIZE,
  QUALITY_AREAS,
  QUALITY_DIMENSIONS,
  QUALITY_LIMITS,
  buildRunRequest,
  buildSnapshotRequest,
  defaultDimensionsUnavailable,
  describeExecutionStatus,
  describeStructuralStatus,
  formatQualityError,
  mapBenchmarkDetailToView,
  mapReportEnvelopeToDetailView,
  mapReportListToRunViews,
  mapSnapshotFactsToEvidence,
  mapSnapshotReceiptToPreview,
  paginateItems,
  partitionEvidence,
  safeHttpUrl,
  sortChecksNonPassFirst,
  sortDimensionsNonPassFirst,
  verifyRawArtifactDigest,
} from '../src/qualityModels.ts'

const here = dirname(fileURLToPath(import.meta.url))
const codexContractDir = process.env.AETHER_APO11_CONTRACT_DIR
  || resolve(here, '..', '..', '..', 'AETHER-apo11-codex', 'contracts', 'apo11')

test('defines four principal workspace areas and preserves all eight evaluation dimensions', () => {
  assert.deepEqual(
    QUALITY_AREAS.map((area) => area.id),
    ['benchmarks', 'overview', 'diagnostics', 'integrity'],
  )
  assert.equal(QUALITY_AREAS.some((area) => String(area.id).includes('human')), false)
  assert.equal(QUALITY_DIMENSIONS.length, 8)
  assert.deepEqual(
    QUALITY_DIMENSIONS.map((entry) => entry.id),
    [
      'literature_scope_inventory_integrity',
      'claim_source_locator_integrity',
      'evidence_provenance_field_integrity',
      'counterevidence_edge_integrity',
      'hypothesis_structure_completeness',
      'experiment_design_arithmetic_integrity',
      'uncertainty_field_wellformedness',
      'artifact_snapshot_replay_integrity',
    ],
  )
  const defaults = defaultDimensionsUnavailable()
  assert.equal(defaults.length, 8)
  assert.ok(defaults.every((item) => item.status === 'unavailable'))
})

test('structural and execution descriptions never imply scientific verification', () => {
  assert.equal(describeStructuralStatus('pass'), 'Structural pass')
  assert.equal(describeStructuralStatus('fail'), 'Structural fail')
  assert.equal(describeStructuralStatus('unavailable'), 'Unavailable')
  assert.notEqual(describeStructuralStatus('pass'), 'Verified')
  assert.equal(describeExecutionStatus('completed'), 'Execution completed')
  assert.equal(describeExecutionStatus('abstained'), 'Execution abstained')
})

test('sorts non-pass dimensions and checks before passing items', () => {
  const orderedDims = sortDimensionsNonPassFirst([
    {
      dimension: 'literature_scope_inventory_integrity',
      title: 'Lit',
      status: 'pass',
      semanticLabel: 'Human review: not_evaluated',
      checksCount: 1,
      failCount: 0,
      unavailableCount: 0,
      passCount: 1,
      limitations: [],
    },
    {
      dimension: 'evidence_provenance_field_integrity',
      title: 'Prov',
      status: 'unavailable',
      semanticLabel: 'Human review: not_evaluated',
      checksCount: 1,
      failCount: 0,
      unavailableCount: 1,
      passCount: 0,
      limitations: [],
    },
    {
      dimension: 'claim_source_locator_integrity',
      title: 'Trace',
      status: 'fail',
      semanticLabel: 'Human review: not_evaluated',
      checksCount: 1,
      failCount: 1,
      unavailableCount: 0,
      passCount: 0,
      limitations: [],
    },
  ])
  assert.deepEqual(
    orderedDims.map((item) => item.status),
    ['fail', 'unavailable', 'pass'],
  )

  const orderedChecks = sortChecksNonPassFirst([
    {
      id: 'b_pass',
      dimension: 'artifact_snapshot_replay_integrity',
      ruleVersion: '1',
      status: 'pass',
      reasonLabel: 'None',
      reasonDetail: 'ok',
      evidenceIds: [],
    },
    {
      id: 'c_unavail',
      dimension: 'artifact_snapshot_replay_integrity',
      ruleVersion: '1',
      status: 'unavailable',
      reasonLabel: 'source_replay_unavailable_rights_unknown',
      reasonDetail: 'missing',
      evidenceIds: [],
    },
    {
      id: 'a_fail',
      dimension: 'artifact_snapshot_replay_integrity',
      ruleVersion: '1',
      status: 'fail',
      reasonLabel: 'digest_mismatch',
      reasonDetail: 'bad',
      evidenceIds: [],
    },
  ])
  assert.deepEqual(
    orderedChecks.map((item) => item.id),
    ['a_fail', 'c_unavail', 'b_pass'],
  )
})

test('paginates check rows using bounded 50-item pages and aligns backend report page limit 32', () => {
  assert.equal(DEFAULT_REPORT_PAGE_SIZE, 32)
  assert.equal(DEFAULT_CHECK_PAGE_SIZE, 50)
  const items = Array.from({ length: 85 }, (_, index) => index + 1)
  const first = paginateItems(items, 1, DEFAULT_CHECK_PAGE_SIZE)
  assert.equal(first.slice.length, 50)
  assert.equal(first.startIndex, 1)
  assert.equal(first.endIndex, 50)
  assert.equal(first.totalPages, 2)

  const second = paginateItems(items, 2, DEFAULT_CHECK_PAGE_SIZE)
  assert.equal(second.slice.length, 35)
  assert.equal(second.startIndex, 51)
  assert.equal(second.endIndex, 85)

  const clamped = paginateItems(items, 99, DEFAULT_CHECK_PAGE_SIZE)
  assert.equal(clamped.page, 2)
})

test('partitions evidence into supporting, contradictory, and contextual without losing empty states', () => {
  const onlySupporting = partitionEvidence([
    {
      id: '1'.repeat(64),
      relationLabel: 'supports',
      polarity: 'supporting',
      observedAliases: [],
      integrityStatus: 'valid',
    },
  ])
  assert.equal(onlySupporting.supporting.length, 1)
  assert.equal(onlySupporting.contradictory.length, 0)
  assert.equal(onlySupporting.contextual.length, 0)
})

test('verifies raw artifact bytes using Web Crypto without re-serializing JSON objects', async () => {
  const nonNormalizedRaw = new TextEncoder().encode('{\n  "b":  2,\n  "a": 1\n}')
  const rawDigest = [...new Uint8Array(await crypto.subtle.digest('SHA-256', nonNormalizedRaw))]
    .map((byte) => byte.toString(16).padStart(2, '0'))
    .join('')

  const valid = await verifyRawArtifactDigest(nonNormalizedRaw, rawDigest)
  assert.equal(valid.matched, true)
  assert.equal(valid.computedSha256, rawDigest)
  assert.equal(valid.byteLength, nonNormalizedRaw.byteLength)

  const reserializedBytes = new TextEncoder().encode(
    JSON.stringify(JSON.parse(new TextDecoder().decode(nonNormalizedRaw))),
  )
  const tampered = await verifyRawArtifactDigest(reserializedBytes, rawDigest)
  assert.equal(tampered.matched, false)
  assert.notEqual(tampered.computedSha256, rawDigest)

  const invalidHex = await verifyRawArtifactDigest(nonNormalizedRaw, 'not-a-sha256')
  assert.equal(invalidHex.matched, false)
})

test('formats HTTP 413 request_too_large and API errors clearly', () => {
  assert.match(
    formatQualityError({
      statusCode: 413,
      code: 'request_too_large',
      message: 'request body exceeds the 8192-byte limit',
      detail: '8400 bytes',
    }),
    /HTTP 413 request_too_large:.*8192-byte.*\(8400 bytes\)/,
  )
  assert.equal(
    formatQualityError({ statusCode: 409, code: 'state_conflict', message: 'Source changed.' }),
    'HTTP 409 state_conflict: Source changed.',
  )
})

test('validates canonical snapshot -> run request builders and rejects unsafe external links', () => {
  const root = '2'.repeat(64)
  const bench = '1'.repeat(64)
  const snap = '3'.repeat(64)
  assert.throws(
    () =>
      buildSnapshotRequest({
        graphRootId: root,
        graphDepth: 1,
        subjects: [],
        generateHypotheses: true,
        generateExperimentDesigns: false,
      }),
    /requires graph_depth = 2/,
  )
  assert.throws(
    () =>
      buildSnapshotRequest({
        graphRootId: root,
        graphDepth: 2,
        subjects: [
          {
            kind: 'hypothesis',
            artifact: { kind: 'experiment_design_report', sha256: snap },
            pointer: '/proposals/0',
          },
        ],
        generateHypotheses: false,
        generateExperimentDesigns: false,
      }),
    /must reference artifact kind "hypothesis_report"/,
  )

  const req = buildSnapshotRequest({
    graphRootId: root,
    graphDepth: 2,
    subjects: [],
    generateHypotheses: false,
    generateExperimentDesigns: false,
  })
  assert.equal(req.schema, 'aether.quality-snapshot-request/1')
  assert.deepEqual(buildRunRequest(bench, snap), {
    schema: 'aether.quality-run-request/1',
    benchmark_sha256: bench,
    snapshot_sha256: snap,
  })

  assert.equal(safeHttpUrl('javascript:alert(1)'), null)
  assert.equal(safeHttpUrl('data:text/html,<script>alert(1)</script>'), null)
  assert.equal(safeHttpUrl('https://example.invalid/paper'), 'https://example.invalid/paper')
})

test('matches Codex canonical JSON Schema and all golden fixtures', async () => {
  assert.equal(existsSync(resolve(codexContractDir, 'quality-evaluation.schema.json')), true)
  const readJson = (relPath) => JSON.parse(readFileSync(resolve(codexContractDir, relPath), 'utf8'))

  const schema = readJson('quality-evaluation.schema.json')
  const benchmarkSpec = readJson('benchmark.json')
  const catalog = readJson('fixtures/benchmark-catalog.json')
  const benchmarkDetail = readJson('fixtures/benchmark-detail.json')
  const snapshotReq = readJson('fixtures/snapshot-request.json')
  const snapshotReceipt = readJson('fixtures/snapshot-receipt.json')
  const snapshotDoc = readJson('fixtures/snapshot-document.json')
  const runReq = readJson('fixtures/run-request.json')
  const reportEnvelope = readJson('fixtures/report-envelope.json')
  const reportList = readJson('fixtures/report-list.json')

  assert.deepEqual(
    QUALITY_DIMENSIONS.map((entry) => entry.id),
    schema.$defs.dimension.enum,
  )
  assert.equal(QUALITY_LIMITS.requestBytes, schema.$defs.benchmark.properties.limits.properties.request_bytes.const)
  assert.equal(QUALITY_LIMITS.pageSize, schema.$defs.benchmark.properties.limits.properties.page_size.const)
  assert.equal(benchmarkSpec.dimension_rules.length, 8)
  assert.deepEqual(
    buildSnapshotRequest({
      graphRootId: snapshotReq.graph_root_id,
      graphDepth: snapshotReq.graph_depth,
      subjects: snapshotReq.subjects,
      generateHypotheses: snapshotReq.generate_hypotheses,
      generateExperimentDesigns: snapshotReq.generate_experiment_designs,
    }),
    snapshotReq,
  )
  assert.deepEqual(buildRunRequest(runReq.benchmark_sha256, runReq.snapshot_sha256), runReq)

  const benchmarkView = mapBenchmarkDetailToView(benchmarkDetail)
  assert.equal(benchmarkView.rules.length, 8)
  assert.equal(benchmarkView.constraints.length, 10)
  assert.deepEqual(benchmarkView.unavailableMetrics, benchmarkSpec.excluded_metrics)
  assert.equal(catalog.benchmarks[0].dimensions.length, 8)

  const preview = mapSnapshotReceiptToPreview(snapshotReceipt, snapshotDoc)
  assert.equal(preview.sourceTextIncluded, false)
  assert.equal(preview.snapshotDigest, snapshotReceipt.snapshot_sha256)

  const runs = mapReportListToRunViews(reportList, benchmarkView)
  assert.equal(runs.length, 0)

  const rawReportBytes = new TextEncoder().encode(JSON.stringify(reportEnvelope.report))
  const verified = await verifyRawArtifactDigest(rawReportBytes, reportEnvelope.report_sha256)
  assert.equal(verified.matched, true)

  const detailView = mapReportEnvelopeToDetailView(reportEnvelope, {
    benchmark: benchmarkView,
    snapshotDoc,
    rawArtifactBytes: rawReportBytes,
  })
  assert.equal(detailView.dimensions.length, 8)
  assert.equal(detailView.checks.length, 8)
  assert.equal(detailView.executionStatus, 'completed')
  assert.match(detailView.humanReviewLabel, /not_evaluated/)
  assert.match(detailView.independentValidationLabel, /not_performed/)
  assert.deepEqual(mapSnapshotFactsToEvidence(snapshotDoc), [])
})
