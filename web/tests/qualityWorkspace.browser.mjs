import { strict as assert } from 'node:assert'
import { createRequire } from 'node:module'
import { dirname, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'
import { createServer } from 'vite'

const require = createRequire(import.meta.url)
const testsDir = dirname(fileURLToPath(import.meta.url))
const webDir = resolve(testsDir, '..')
const playwrightPath = process.env.AETHER_PLAYWRIGHT_PATH || resolve(webDir, 'node_modules', 'playwright')
const { chromium } = require(playwrightPath)

const harnessSource = `
import { StrictMode, createElement as h, useState } from 'react'
import { createRoot } from 'react-dom/client'
import '/src/styles.css'
import { externalLink } from '/src/components.tsx'
import { QualityEvaluationWorkspace } from '/src/qualityEvaluationView.tsx'

window.__externalLinkCheck = {
  js: externalLink('javascript:alert(1)'),
  data: externalLink('data:text/html,<script>'),
  https: externalLink('https://example.invalid/paper'),
}

const ROOT_DIGEST = 'a'.repeat(64)
const EXPORT_DIGEST = 'b'.repeat(64)
const GRAPH_DIGEST = 'c'.repeat(64)
const SNAP_DIGEST = 'd'.repeat(64)
const BENCH_DIGEST = 'e'.repeat(64)
const CONFIG_DIGEST = 'f'.repeat(64)
const EVAL_DIGEST = '1'.repeat(64)
const RUN1_DIGEST = '2'.repeat(64)
const RUN2_DIGEST = '3'.repeat(64)

const rawReportText = '{"report":"canonical-raw-bytes-test","version":1}'
const rawBytes = new TextEncoder().encode(rawReportText)
const hashBytes = new Uint8Array(await crypto.subtle.digest('SHA-256', rawBytes))
const realReportDigest = [...hashBytes].map((b) => b.toString(16).padStart(2, '0')).join('')

const checks = Array.from({ length: 65 }, (_, index) => {
  const num = index + 1
  if (num === 1) {
    return {
      id: 'claim_source_locators_resolve',
      dimension: 'claim_source_locator_integrity',
      ruleVersion: '1',
      status: 'fail',
      reasonLabel: 'invalid_source_locator',
      reasonDetail: 'Explicit source locator failed UTF-8 byte range check <script>window.__apo11Xss=1</script>.',
      subjectKind: 'hypothesis',
      subjectDigest: ROOT_DIGEST,
      subjectPointer: '/proposals/0',
      evidenceIds: ['9'.repeat(64)],
    }
  }
  if (num === 2) {
    return {
      id: 'evidence_provenance_fields_present',
      dimension: 'evidence_provenance_field_integrity',
      ruleVersion: '1',
      status: 'unavailable',
      reasonLabel: 'epistemic_or_identity_status_missing',
      reasonDetail: 'Legacy Metadata node lacks source_identity_status and epistemic_status.',
      subjectKind: 'graph_snapshot',
      subjectDigest: BENCH_DIGEST,
      evidenceIds: ['8'.repeat(64)],
    }
  }
  return {
    id: 'structural_check_' + String(num).padStart(3, '0'),
    dimension: num % 2 === 0 ? 'literature_scope_inventory_integrity' : 'artifact_snapshot_replay_integrity',
    ruleVersion: '1',
    status: 'pass',
    reasonLabel: 'None',
    reasonDetail: 'All pinned schema predicates satisfied.',
    subjectKind: 'graph_snapshot',
    subjectDigest: EXPORT_DIGEST,
    evidenceIds: ['9'.repeat(64)],
  }
})

const dimensions = [
  { dimension: 'literature_scope_inventory_integrity', title: 'Literature scope & inventory integrity', status: 'pass', semanticLabel: 'Human review: not_evaluated', checksCount: 31, failCount: 0, unavailableCount: 0, passCount: 31, limitations: ['Does not establish literature recall.'] },
  { dimension: 'claim_source_locator_integrity', title: 'Claim source locator integrity', status: 'fail', semanticLabel: 'Human review: not_evaluated', checksCount: 1, failCount: 1, unavailableCount: 0, passCount: 0, limitations: ['Semantic entailment not evaluated.'] },
  { dimension: 'evidence_provenance_field_integrity', title: 'Evidence provenance field integrity', status: 'unavailable', semanticLabel: 'Human review: not_evaluated', checksCount: 1, failCount: 0, unavailableCount: 1, passCount: 0, limitations: ['Legacy passage absent.'] },
  { dimension: 'counterevidence_edge_integrity', title: 'Counterevidence edge integrity', status: 'unavailable', semanticLabel: 'Human review: not_evaluated', checksCount: 0, failCount: 0, unavailableCount: 0, passCount: 0, limitations: ['No contradicts edges in slice.'] },
  { dimension: 'hypothesis_structure_completeness', title: 'Hypothesis structure completeness', status: 'pass', semanticLabel: 'Human review: not_evaluated', checksCount: 0, failCount: 0, unavailableCount: 0, passCount: 0, limitations: ['Plausibility not evaluated.'] },
  { dimension: 'experiment_design_arithmetic_integrity', title: 'Experiment design arithmetic integrity', status: 'pass', semanticLabel: 'Human review: not_evaluated', checksCount: 0, failCount: 0, unavailableCount: 0, passCount: 0, limitations: ['Statistical power not evaluated.'] },
  { dimension: 'uncertainty_field_wellformedness', title: 'Uncertainty field well-formedness', status: 'pass', semanticLabel: 'Human review: not_evaluated', checksCount: 0, failCount: 0, unavailableCount: 0, passCount: 0, limitations: ['Calibration not evaluated.'] },
  { dimension: 'artifact_snapshot_replay_integrity', title: 'Artifact snapshot & replay integrity', status: 'pass', semanticLabel: 'Human review: not_evaluated', checksCount: 32, failCount: 0, unavailableCount: 0, passCount: 32, limitations: ['Source text replay unavailable.'] },
]

function Harness() {
  const [scenario, setScenario] = useState('normal')
  const [runs, setRuns] = useState([
    {
      runKey: realReportDigest,
      sequence: 12,
      reportDigest: realReportDigest,
      benchmarkTitle: 'aether-research-quality-structural',
      benchmarkVersion: '1.0.0',
      benchmarkDigest: BENCH_DIGEST,
      snapshotDigest: SNAP_DIGEST,
      executionStatus: 'completed',
      dimensions,
    },
    {
      runKey: RUN2_DIGEST,
      sequence: 11,
      reportDigest: RUN2_DIGEST,
      benchmarkTitle: 'aether-research-quality-structural',
      benchmarkVersion: '1.0.0',
      benchmarkDigest: BENCH_DIGEST,
      snapshotDigest: RUN1_DIGEST,
      executionStatus: 'abstained',
      dimensions,
    },
  ])
  const [selectedDigest, setSelectedDigest] = useState(realReportDigest)

  const activeReport = {
    reportDigest: selectedDigest,
    sequence: 12,
    snapshotDigest: SNAP_DIGEST,
    graphRootId: ROOT_DIGEST,
    graphSha256: GRAPH_DIGEST,
    benchmarkLabel: 'aether-research-quality-structural v1.0.0',
    benchmarkDigest: BENCH_DIGEST,
    configurationDigest: GRAPH_DIGEST,
    serializationLabel: 'serde_json::to_vec (aether.quality-report/1)',
    evaluatorVersion: 'aether-structural-evaluator/1.0.0',
    evaluatorIdentityDigest: undefined,
    evaluatorBinaryDigest: EVAL_DIGEST,
    evaluatorBuildAttestation: 'not_provided',
    evaluatorLimitations: ['executable_sha256 is self-reported and build_attestation is fixed to not_provided.'],
    replaySummary: 'Unavailable — source_text_included: false',
    executionStatus: selectedDigest === RUN2_DIGEST ? 'abstained' : 'completed',
    humanReviewLabel: 'Human review: Not started (semantic_assessment = not_evaluated)',
    independentValidationLabel: 'Independent scientific validation: Not performed (not_performed)',
    limitations: [
      'Structural checks confirm schema and digest integrity only.',
      'Untrusted limitation text: <img src=x onerror="window.__apo11Xss=2">',
    ],
    dimensions,
    checks,
    evidence: [
      {
        id: '9'.repeat(64),
        relationLabel: 'supports',
        polarity: 'supporting',
        claimRef: ROOT_DIGEST,
        canonicalSource: '10.1000/test-doi <script>window.__apo11Xss=3</script>',
        observedAliases: ['doi:10.1000/test-doi'],
        version: 'v1',
        sourceDigest: GRAPH_DIGEST,
        sourceUrl: 'javascript:window.__apo11Xss=4',
        sourceGroup: 'evidence',
        passageLocatorLabel: 'utf8_byte_range [120, 260)',
        representationDigest: CONFIG_DIGEST,
        passageDigest: EXPORT_DIGEST,
        provenanceDigest: SNAP_DIGEST,
        provenanceSchema: 'origin:passage',
        sourceIdentityStatus: 'unverified',
        epistemicOriginLabel: 'unknown',
        integrityStatus: 'invalid',
        integrityReason: 'invalid_source_locator',
        campaignVerification: undefined,
      },
      {
        id: '8'.repeat(64),
        relationLabel: 'other',
        polarity: 'contextual',
        canonicalSource: '10.1000/legacy-node',
        observedAliases: ['arxiv:2501.00001'],
        sourceGroup: 'evidence',
        provenanceSchema: 'origin:metadata',
        sourceIdentityStatus: null,
        epistemicOriginLabel: null,
        integrityStatus: 'unavailable',
        integrityReason: 'legacy_metadata_lacks_passage_locator',
      },
    ],
    snapshotMetadata: {
      rootId: ROOT_DIGEST,
      depth: 2,
      graphDigest: GRAPH_DIGEST,
      exportDigest: EXPORT_DIGEST,
      truncated: true,
      snapshotDigest: SNAP_DIGEST,
      sourceTextIncluded: false,
      nodeCount: 3,
      edgeCount: 2,
      subjectCount: 1,
      resolvedArtifactsCount: 1,
      unresolvedItems: [{ referenceDigest: '8'.repeat(64), reason: 'legacy_locator_unavailable', legacyNote: 'Legacy Metadata node without text span' }],
      replayStatusLabel: 'Unavailable — not provided',
      notes: ['Graph export reached traversal truncation limit.'],
    },
    rawArtifactBytes: selectedDigest === RUN2_DIGEST ? new TextEncoder().encode('tampered') : rawBytes,
  }

  return h(
    'div',
    { style: { padding: 16 } },
    h(
      'div',
      { className: 'toolbar' },
      h('button', { type: 'button', onClick: () => setScenario('too-large') }, 'Simulate 413 next capture'),
      h('button', { type: 'button', onClick: () => setScenario('stale') }, 'Simulate stale revision capture'),
      h('button', { type: 'button', onClick: () => setScenario('normal') }, 'Reset capture mode'),
    ),
    h(QualityEvaluationWorkspace, {
      data: {
        integration: { status: 'connected' },
        benchmarks: [{
          digest: BENCH_DIGEST,
          id: 'aether-research-quality-structural',
          version: '1.0.0',
          evaluatorVersion: 'aether-structural-evaluator/1.0.0',
          scopeLabel: 'Structural checks only',
          rules: [{
            id: 'claim_source_locators_resolve',
            version: '1',
            dimension: 'claim_source_locator_integrity',
            title: 'Claim locators resolve structurally',
            description: 'Checks typed evidence links and source locator fields.',
            cannotEstablish: 'Semantic entailment.',
            emptyCollectionPolicy: 'unavailable_optional',
          }],
          constraints: [{ label: 'POST request body limit', value: '8192 bytes', detail: 'Enforced before JSON parsing' }],
          unavailableMetrics: ['semantic_entailment', 'scientific_truth', 'independent_validation'],
        }],
        runs,
        runsHasMore: true,
        runsTruncated: true,
        activeReport,
        knowledgeRoots: [{ id: ROOT_DIGEST, kind: 'experiment', label: 'Indexed experiment root', status: 'observed' }],
        snapshotPreview: {
          rootId: ROOT_DIGEST,
          depth: 2,
          graphDigest: GRAPH_DIGEST,
          exportDigest: EXPORT_DIGEST,
          truncated: false,
          sourceTextIncluded: false,
          nodeCount: 3,
          edgeCount: 2,
          subjectCount: 1,
          resolvedArtifactsCount: 1,
          unresolvedItems: [],
          notes: [],
        },
      },
      actions: {
        onCaptureSnapshot: async (req) => {
          if (req.schema !== 'aether.quality-snapshot-request/1') {
            throw { statusCode: 400, code: 'invalid_request', message: 'Missing canonical snapshot request schema.' }
          }
          if (scenario === 'too-large') {
            throw { statusCode: 413, code: 'request_too_large', message: 'request body exceeds the 8192-byte limit' }
          }
          if (scenario === 'stale') {
            return {
              rootId: ROOT_DIGEST,
              depth: req.graph_depth,
              graphDigest: GRAPH_DIGEST,
              exportDigest: EXPORT_DIGEST,
              truncated: true,
              sourceTextIncluded: false,
              resolvedArtifactsCount: 0,
              unresolvedItems: [{ referenceDigest: '8'.repeat(64), reason: 'legacy_locator_unavailable' }],
              isStaleRevision: true,
              replayStatusLabel: 'Unavailable — stale revision',
              notes: ['Concurrent catalog change detected.'],
            }
          }
          return {
            rootId: req.graph_root_id,
            depth: req.graph_depth,
            graphDigest: GRAPH_DIGEST,
            exportDigest: EXPORT_DIGEST,
            truncated: false,
            snapshotDigest: SNAP_DIGEST,
            sourceTextIncluded: false,
            nodeCount: 3,
            edgeCount: 2,
            subjectCount: req.subjects.length,
            resolvedArtifactsCount: req.subjects.length,
            unresolvedItems: [{ referenceDigest: '8'.repeat(64), reason: 'legacy_locator_unavailable', legacyNote: 'Legacy node preserved as unresolved' }],
            replayStatusLabel: 'Unavailable — dev mode',
            notes: ['Snapshot verified before evaluation.'],
          }
        },
        onRunEvaluation: async (req) => {
          if (req.schema !== 'aether.quality-run-request/1') {
            throw { statusCode: 400, code: 'invalid_request', message: 'Missing canonical run request schema.' }
          }
          setRuns((current) => [...current])
        },
        onSelectRun: async (digest) => setSelectedDigest(digest),
        onLoadMoreRuns: async () => {
          setRuns((current) => current.length > 2 ? current : [...current, {
            runKey: RUN1_DIGEST, sequence: 10, reportDigest: RUN1_DIGEST,
            benchmarkTitle: 'aether-research-quality-structural', benchmarkVersion: '1.0.0',
            benchmarkDigest: BENCH_DIGEST, snapshotDigest: SNAP_DIGEST,
            executionStatus: 'partial', dimensions,
          }])
        },
      },
    }),
  )
}

createRoot(document.getElementById('root')).render(h(StrictMode, null, h(Harness)))
`

function harnessPlugin() {
  return {
    name: 'apo11-harness',
    configureServer(server) {
      server.middlewares.use((req, res, next) => {
        if (req.url === '/__apo11_harness.html') {
          res.setHeader('Content-Type', 'text/html; charset=utf-8')
          res.end('<!doctype html><html><head><meta charset="utf-8"><meta name="viewport" content="width=device-width,initial-scale=1"></head><body><div id="root"></div><script type="module" src="/@apo11-harness.js"></script></body></html>')
          return
        }
        next()
      })
    },
    resolveId(id) {
      if (id === '/@apo11-harness.js') return '\0apo11-harness.js'
    },
    load(id) {
      if (id === '\0apo11-harness.js') return harnessSource
    },
  }
}

async function main() {
  const server = await createServer({
    root: webDir,
    configFile: false,
    plugins: [harnessPlugin()],
    server: { host: '127.0.0.1', port: 0 },
  })
  await server.listen()
  const address = server.httpServer.address()
  const origin = `http://127.0.0.1:${address.port}`
  const browser = await chromium.launch({
    executablePath: process.env.AETHER_CHROMIUM_PATH || undefined,
  })
  const errors = []
  try {
    const page = await browser.newPage({ viewport: { width: 1366, height: 900 } })
    page.on('pageerror', (err) => errors.push(err.message))
    await page.goto(`${origin}/__apo11_harness.html`)

    // 1. Verify 4 areas, epistemic boundaries, zero Human Review tab
    const tabs = page.getByRole('tab')
    assert.equal(await tabs.count(), 4)
    assert.equal(await page.getByRole('tab', { name: /Human Review/i }).count(), 0)
    await page.getByText('Human review: Not started').first().waitFor()
    await page.getByText('Independent scientific validation: Not performed').first().waitFor()

    // 2. Keyboard navigation across 4 tabs
    await page.getByRole('tab', { name: 'Benchmarks & Run Capture' }).focus()
    await page.keyboard.press('ArrowRight')
    assert.equal(await page.getByRole('tab', { name: 'Runs & Report Overview' }).getAttribute('aria-selected'), 'true')
    await page.keyboard.press('End')
    assert.equal(await page.getByRole('tab', { name: 'Integrity, Snapshot & Audit' }).getAttribute('aria-selected'), 'true')
    await page.keyboard.press('Home')
    assert.equal(await page.getByRole('tab', { name: 'Benchmarks & Run Capture' }).getAttribute('aria-selected'), 'true')

    // 3. Area 1: Canonical Snapshot -> Run workflow + 413 + stale revision handling
    await page.getByRole('button', { name: 'Capture snapshot' }).click()
    await page.getByText('Immutable snapshot captured').waitFor()
    await page.getByRole('button', { name: 'Simulate 413 next capture' }).click()
    await page.getByRole('button', { name: 'Capture snapshot' }).click()
    await page.getByText(/HTTP 413 request_too_large/).waitFor()
    await page.getByRole('button', { name: 'Simulate stale revision capture' }).click()
    await page.getByRole('button', { name: 'Capture snapshot' }).click()
    await page.getByText(/Source revisions changed during capture/).waitFor()
    await page.getByRole('button', { name: 'Reset capture mode' }).click()
    await page.getByRole('button', { name: 'Capture snapshot' }).click()
    await page.getByRole('button', { name: 'Run evaluation against snapshot' }).click()

    // 4. Area 2: Canonical 8 dimensions, newest-first list, pagination, non-pass-first sorting, distinct badges, XSS inert
    await page.getByRole('heading', { name: /Eight structural dimensions/ }).waitFor()
    const cardTitles = await page.locator('.eval-dimension-card h3').allTextContents()
    assert.equal(cardTitles.length, 8)
    assert.equal(cardTitles[0], 'Claim source locator integrity')
    assert.ok(['Evidence provenance field integrity', 'Counterevidence edge integrity'].includes(cardTitles[1]))
    await page.getByText('Structural pass').first().waitFor()
    await page.getByText('Structural fail').first().waitFor()
    await page.locator('.eval-dimension-card').getByText('Unavailable', { exact: true }).first().waitFor()
    assert.equal(await page.locator('.badge--good', { hasText: 'Structural pass' }).count(), 0)
    await page.getByText(/Untrusted limitation text: <img src=x/).waitFor()
    await page.getByRole('button', { name: 'Load earlier evaluation runs' }).click()
    await page.getByText('Execution partial').waitFor()

    // 5. Area 3: Diagnostics, 50-row bounded pagination, legacy provenance, empty counterevidence, untrusted XSS inertness
    await page.getByRole('tab', { name: 'Dimension & Evidence Diagnostics' }).click()
    await page.getByText('Showing 1–50 of 65 checks').waitFor()
    await page.getByText(/<script>window\.__apo11Xss=1<\/script>/).first().waitFor()
    await page.getByText('javascript:window.__apo11Xss=4').waitFor()
    assert.equal(await page.locator('a[href^="javascript:"]').count(), 0)
    assert.equal(await page.evaluate(() => window.__apo11Xss), undefined)
    assert.deepEqual(
      await page.evaluate(() => window.__externalLinkCheck),
      { js: null, data: null, https: 'https://example.invalid/paper' },
    )
    await page.getByRole('button', { name: 'Next page' }).click()
    await page.getByText('Showing 51–65 of 65 checks').waitFor()
    await page.getByRole('button', { name: 'Previous page' }).click()
    await page.getByText(/No contradictory evidence was recorded/).waitFor()
    await page.getByRole('button', { name: 'evidence_provenance_fields_present' }).click()
    await page.getByText(/Unavailable — no legacy passage span recorded/).waitFor()

    // 6. Area 4: Web Crypto raw-byte digest verification pass + fail-closed mismatch
    await page.getByRole('tab', { name: 'Integrity, Snapshot & Audit' }).click()
    await page.getByRole('button', { name: 'Verify raw artifact bytes (Web Crypto)' }).click()
    await page.getByText(/Exact raw-byte SHA-256 verified/).waitFor()
    assert.equal(await page.getByRole('button', { name: 'Download verified report JSON' }).isEnabled(), true)

    await page.getByRole('tab', { name: 'Runs & Report Overview' }).click()
    await page.getByRole('button', { name: '3333333333…' }).click()
    await page.getByRole('tab', { name: 'Integrity, Snapshot & Audit' }).click()
    await page.getByRole('button', { name: 'Verify raw artifact bytes (Web Crypto)' }).click()
    await page.getByText(/Computed SHA-256 of raw artifact bytes differs/).waitFor()
    assert.equal(await page.getByRole('button', { name: 'Download verified report JSON' }).isDisabled(), true)

    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1), true)

    // 7. Mobile viewport overflow & responsiveness check
    const mobile = await browser.newPage({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true })
    mobile.on('pageerror', (err) => errors.push(err.message))
    await mobile.goto(`${origin}/__apo11_harness.html`)
    for (const label of [
      'Benchmarks & Run Capture',
      'Runs & Report Overview',
      'Dimension & Evidence Diagnostics',
      'Integrity, Snapshot & Audit',
    ]) {
      await mobile.getByRole('tab', { name: label }).click()
      assert.equal(
        await mobile.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1),
        true,
        `Mobile horizontal overflow detected on area ${label}`,
      )
    }

    assert.deepEqual(errors, [])
    console.log('Quality workspace browser harness passed all four areas, canonical dimensions, XSS safety, Web Crypto verification, 50-row pagination, keyboard navigation, and 390x844 mobile layout checks.')
  } finally {
    await browser.close()
    await server.close()
  }
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})
