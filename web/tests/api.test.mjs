import { strict as assert } from 'node:assert'
import { test } from 'node:test'
import {
  ApiError,
  createEvalSnapshot,
  getEvalArtifactBytes,
  getEvalBenchmark,
  getEvalBenchmarks,
  getEvalReport,
  getEvalSnapshotDocument,
  getResearchArchive,
  getResearchArtifact,
  getSnapshot,
  listEvalReports,
  post,
  runEvalQuality,
  runStudy,
} from '../src/api.ts'

test('benchmark response unwraps the measured simulation envelope', async (t) => {
  const payload = { scope: 'synthetic', measurement: { simulation_wall_us: 20 }, report: { campaigns: [{ mode: 'A' }] } }
  t.mock.method(globalThis, 'fetch', async (url, options) => {
    assert.equal(url, '/v1/benchmarks')
    assert.equal(options.method, 'POST')
    assert.equal(options.headers.get('Authorization'), 'Bearer test-token')
    return Response.json(payload)
  })
  assert.deepEqual((await runStudy('test-token')).campaigns, [{ mode: 'A' }])
})

test('a rejected frozen approval reports the structured API message', async (t) => {
  t.mock.method(globalThis, 'fetch', async () => Response.json(
    { code: 'state_conflict', error: 'approval hash does not match frozen protocol' },
    { status: 409 },
  ))
  await assert.rejects(
    post('test-token', '/campaigns/frozen/approve', { protocol_hash: 'bad' }),
    (error) => error instanceof ApiError && error.status === 409 && error.code === 'state_conflict'
      && error.message === 'approval hash does not match frozen protocol',
  )
})

test('non-campaign data cannot become an authenticated snapshot', async (t) => {
  t.mock.method(globalThis, 'fetch', async () => Response.json({ claims: [] }))
  await assert.rejects(getSnapshot('test-token', 'invalid'), /Invalid campaign snapshot/)
})

test('research archive and raw artifacts use authenticated read-only requests', async (t) => {
  const archive = { schema: 'aether.research-archive/1', recorded_on: '2026-11-26', performance: {}, discovery: { baseline: {}, hybrid: {} } }
  const calls = []
  t.mock.method(globalThis, 'fetch', async (url, options) => {
    calls.push([url, options])
    if (url === '/v1/research/archive') return Response.json(archive)
    const bytes = new TextEncoder().encode('{"schema":"aether.native-performance/1"}')
    const digest = [...new Uint8Array(await crypto.subtle.digest('SHA-256', bytes))].map((byte) => byte.toString(16).padStart(2, '0')).join('')
    return new Response(bytes, { headers: { 'x-content-sha256': digest } })
  })
  assert.equal((await getResearchArchive('test-token')).schema, archive.schema)
  assert.equal(new TextDecoder().decode(await getResearchArtifact('test-token', 'dae-native-performance')), '{"schema":"aether.native-performance/1"}')
  assert.deepEqual(calls.map(([url]) => url), ['/v1/research/archive', '/v1/research/archive/dae-native-performance'])
  assert.ok(calls.every(([, options]) => options.headers.get('Authorization') === 'Bearer test-token' && options.method === undefined))
  await assert.rejects(getResearchArtifact('test-token', '../secrets'), /Invalid research artifact ID/)
})

test('research artifacts fail closed when their content digest differs', async (t) => {
  t.mock.method(globalThis, 'fetch', async () => new Response('{"schema":"aether.native-performance/1"}', {
    headers: { 'x-content-sha256': '0'.repeat(64) },
  }))
  await assert.rejects(getResearchArtifact('test-token', 'dae-native-performance'), /integrity check failed/)
})

test('eval API preserves HTTP 413 request_too_large and routes snapshot -> run requests correctly', async (t) => {
  const benchDigest = '1'.repeat(64)
  const rootDigest = '2'.repeat(64)
  const snapDigest = '3'.repeat(64)
  const graphDigest = '4'.repeat(64)
  const reportDigest = '5'.repeat(64)
  const calls = []

  t.mock.method(globalThis, 'fetch', async (url, options) => {
    calls.push({ url, method: options?.method ?? 'GET', body: options?.body, auth: options?.headers?.get('Authorization') })
    if (url === '/v1/eval/benchmarks') {
      return Response.json({
        schema: 'aether.quality-benchmarks/1',
        benchmarks: [{
          benchmark_sha256: benchDigest,
          id: 'aether-research-quality-structural',
          version: '1.0.0',
          scope: 'structural_checks_only',
          dimensions: [
            'literature_scope_inventory_integrity',
            'claim_source_locator_integrity',
            'evidence_provenance_field_integrity',
            'counterevidence_edge_integrity',
            'hypothesis_structure_completeness',
            'experiment_design_arithmetic_integrity',
            'uncertainty_field_wellformedness',
            'artifact_snapshot_replay_integrity',
          ],
        }],
      })
    }
    if (url === `/v1/eval/benchmarks/${benchDigest}`) {
      return Response.json({
        schema: 'aether.quality-benchmark-detail/1',
        benchmark_sha256: benchDigest,
        benchmark: { schema: 'aether.quality-benchmark/1', id: 'aether-research-quality-structural', version: '1.0.0', scope: 'structural_checks_only', evaluator_version: '1', dimension_rules: Array(8).fill({}), limits: {}, excluded_metrics: [] },
      })
    }
    if (url === '/v1/eval/snapshots') {
      const parsed = JSON.parse(String(options.body))
      if (parsed.graph_depth === 0) {
        return Response.json({ code: 'request_too_large', error: 'request body exceeds the 8192-byte limit' }, { status: 413 })
      }
      return Response.json({
        schema: 'aether.quality-snapshot-receipt/1',
        snapshot_sha256: snapDigest,
        graph_root_id: rootDigest,
        graph_sha256: graphDigest,
        source_text_included: false,
        source_replay: 'unavailable',
      }, { status: 201 })
    }
    if (url === '/v1/eval/runs') {
      return Response.json({
        schema: 'aether.quality-report-envelope/1',
        report_sha256: reportDigest,
        sequence: 7,
        report: {
          schema: 'aether.quality-report/1',
          benchmark_sha256: benchDigest,
          dimension_rule_pins: [],
          evaluator: { version: 'aether-structural-evaluator/1.0.0', executable_sha256: null, build_attestation: 'not_provided' },
          snapshot_sha256: snapDigest,
          graph_root_id: rootDigest,
          graph_sha256: graphDigest,
          execution_outcome: 'completed',
          dimensions: Array(8).fill({ dimension: 'literature_scope_inventory_integrity', outcome: 'pass', checks: [] }),
          semantic_assessment: 'not_evaluated',
          independent_scientific_validation: 'not_performed',
          source_replay: 'unavailable',
          limitations: ['Structural checks do not establish truth.'],
        },
      }, { status: 201 })
    }
    if (url === '/v1/eval/reports?limit=32&before_sequence=7') {
      return Response.json({ schema: 'aether.quality-reports/1', reports: [], truncated: false, next_before_sequence: null })
    }
    if (url === `/v1/eval/reports/${reportDigest}`) {
      return Response.json({
        schema: 'aether.quality-report-envelope/1',
        report_sha256: reportDigest,
        sequence: 7,
        report: {
          schema: 'aether.quality-report/1',
          dimensions: Array(8).fill({ dimension: 'literature_scope_inventory_integrity', outcome: 'pass', checks: [] }),
          independent_scientific_validation: 'not_performed',
        },
      })
    }
    if (url === `/v1/artifacts/${snapDigest}`) {
      const snapBytes = new TextEncoder().encode(JSON.stringify({
        schema: 'aether.quality-snapshot/1',
        input: { graph_root_id: rootDigest, graph_sha256: graphDigest, graph_depth: 2, graph_truncated: false, nodes: [], edges: [], subjects: [], snapshot_digest_verified: true, graph_digest_verified: true, source_replay_available: false },
        source_text_included: false,
        source_replay: 'unavailable',
      }))
      return new Response(snapBytes)
    }
    if (url === `/v1/artifacts/${reportDigest}`) {
      return new Response(new TextEncoder().encode('{"raw":true}'))
    }
    throw new Error(`Unexpected url ${url}`)
  })

  const catalog = await getEvalBenchmarks('tok')
  assert.equal(catalog.benchmarks[0].benchmark_sha256, benchDigest)
  const detail = await getEvalBenchmark('tok', benchDigest)
  assert.equal(detail.benchmark_sha256, benchDigest)

  await assert.rejects(
    createEvalSnapshot('tok', {
      schema: 'aether.quality-snapshot-request/1',
      graph_root_id: rootDigest,
      graph_depth: 0,
      subjects: [],
      generate_hypotheses: false,
      generate_experiment_designs: false,
    }),
    (err) => err instanceof ApiError && err.status === 413 && err.code === 'request_too_large',
  )

  const receipt = await createEvalSnapshot('tok', {
    schema: 'aether.quality-snapshot-request/1',
    graph_root_id: rootDigest,
    graph_depth: 2,
    subjects: [],
    generate_hypotheses: true,
    generate_experiment_designs: false,
  })
  assert.equal(receipt.snapshot_sha256, snapDigest)

  const envelope = await runEvalQuality('tok', {
    schema: 'aether.quality-run-request/1',
    benchmark_sha256: benchDigest,
    snapshot_sha256: receipt.snapshot_sha256,
  })
  assert.equal(envelope.report_sha256, reportDigest)

  const page = await listEvalReports('tok', 32, 7)
  assert.equal(page.truncated, false)
  const fetched = await getEvalReport('tok', reportDigest)
  assert.equal(fetched.sequence, 7)
  assert.equal(new TextDecoder().decode(await getEvalArtifactBytes('tok', reportDigest)), '{"raw":true}')

  // Fail closed when snapshot raw bytes SHA-256 does not equal requested digest.
  await assert.rejects(getEvalSnapshotDocument('tok', snapDigest), /snapshot artifact integrity check failed/)
  assert.ok(calls.every((call) => call.auth === 'Bearer tok'))
})
