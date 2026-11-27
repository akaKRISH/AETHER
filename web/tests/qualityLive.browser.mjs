import { strict as assert } from 'node:assert'
import { execFileSync, spawn } from 'node:child_process'
import { randomBytes } from 'node:crypto'
import { once } from 'node:events'
import { existsSync } from 'node:fs'
import { mkdtemp, rm } from 'node:fs/promises'
import { createRequire } from 'node:module'
import { tmpdir } from 'node:os'
import { dirname, join, resolve } from 'node:path'
import { fileURLToPath } from 'node:url'

const require = createRequire(import.meta.url)
const testsDir = dirname(fileURLToPath(import.meta.url))
const webDir = resolve(testsDir, '..')
const repoRoot = resolve(webDir, '..')
const playwrightPath = process.env.AETHER_PLAYWRIGHT_PATH || resolve(webDir, 'node_modules', 'playwright')
const { chromium } = require(playwrightPath)

const binaryName = process.platform === 'win32' ? 'aether-lab.exe' : 'aether-lab'
const executable =
  process.env.AETHER_EXECUTABLE ||
  resolve(repoRoot, '..', 'AETHER-apo11-codex', 'target', 'release', binaryName)
const webDist = process.env.AETHER_WEB_DIR || resolve(webDir, 'dist')

function waitForAddress(server) {
  return new Promise((resolveAddress, reject) => {
    let output = ''
    const timer = setTimeout(() => reject(new Error(`Server startup timed out: ${output}`)), 15_000)
    server.once('error', (error) => {
      clearTimeout(timer)
      reject(error)
    })
    server.once('exit', (code) => {
      clearTimeout(timer)
      reject(new Error(`Server exited early (${code}): ${output}`))
    })
    server.stderr.on('data', (chunk) => {
      output += chunk
      const match = output.match(/AETHER listening on (http:\/\/127\.0\.0\.1:\d+)/)
      if (match) {
        clearTimeout(timer)
        resolveAddress(match[1])
      }
    })
  })
}

async function connect(page, origin, token) {
  await page.goto(origin)
  await page.getByRole('button', { name: 'Connect API', exact: true }).click()
  await page.getByLabel('API Bearer token').fill(token)
  await page.getByLabel('API Bearer token').press('Enter')
  await page.getByRole('button', { name: 'Disconnect' }).waitFor()
}

async function main() {
  assert.equal(existsSync(executable), true, `Missing APO-11 backend executable at ${executable}`)
  assert.equal(existsSync(join(webDist, 'index.html')), true, `Missing built frontend dist at ${webDist}`)

  const dataDir = await mkdtemp(join(tmpdir(), 'aether-apo11-live-'))
  const token = randomBytes(32).toString('hex')
  execFileSync(executable, ['hypothesis-index'], {
    cwd: repoRoot,
    env: { ...process.env, AETHER_DATA_DIR: dataDir },
    windowsHide: true,
  })

  const server = spawn(executable, ['serve', '0'], {
    cwd: repoRoot,
    env: {
      ...process.env,
      AETHER_DATA_DIR: dataDir,
      AETHER_TOKEN: token,
      AETHER_WEB_DIR: webDist,
      AETHER_ALLOW_CROSSREF: '0',
      AETHER_ALLOW_OPENALEX: '0',
      AETHER_ALLOW_ARXIV: '0',
    },
    stdio: ['ignore', 'ignore', 'pipe'],
    windowsHide: true,
  })

  let browser
  try {
    const origin = await waitForAddress(server)

    // Confirm real backend enforces 401 and 413 request_too_large with canonical error code.
    assert.equal((await fetch(`${origin}/v1/eval/benchmarks`)).status, 401)
    const oversized = await fetch(`${origin}/v1/eval/snapshots`, {
      method: 'POST',
      headers: { Authorization: `Bearer ${token}`, 'Content-Type': 'application/json' },
      body: JSON.stringify({ oversized: 'x'.repeat(9000) }),
    })
    assert.equal(oversized.status, 413)
    const oversizedJson = await oversized.json()
    assert.equal(oversizedJson.code, 'request_too_large')

    browser = await chromium.launch({
      executablePath: process.env.AETHER_CHROMIUM_PATH || undefined,
    })
    const errors = []
    const page = await browser.newPage({ viewport: { width: 1366, height: 900 } })
    page.on('pageerror', (error) => errors.push(error.message))

    await connect(page, origin, token)
    await page.getByRole('button', { name: 'Quality Evaluation', exact: true }).click()
    await page.getByRole('heading', { name: 'Structural completeness, not scientific truth.' }).waitFor()

    // 1. Live Area 1: Benchmarks & Limits loaded from Codex /v1/eval/benchmarks/{sha256}
    assert.equal(await page.getByRole('tab').count(), 4)
    assert.equal(await page.getByRole('tab', { name: /Human Review/i }).count(), 0)
    await page.getByRole('table', { name: 'Available benchmark specifications' })
      .getByText('aether-research-quality-structural')
      .waitFor()
    await page.getByRole('table', { name: 'Benchmark structural rule catalog' })
      .getByText('experiment_design_budget_arithmetic')
      .waitFor()
    await page.getByText('8192 bytes').first().waitFor()

    // 2. Run 1 (depth 1, no generated subjects): exercises real pass, fail, and unavailable outcomes together
    await page.getByRole('table', { name: 'Available knowledge graph roots' }).waitFor()
    await page.getByLabel('Traversal depth (0–3; generated subjects require depth = 2)').selectOption('1')
    const captureResp = page.waitForResponse((res) => res.url().endsWith('/v1/eval/snapshots'))
    await page.getByRole('button', { name: 'Capture snapshot' }).click()
    const snapHttp = await captureResp
    assert.ok([200, 201].includes(snapHttp.status()), `Unexpected /v1/eval/snapshots status ${snapHttp.status()}: ${await snapHttp.text()}`)
    await page.getByText(/Immutable snapshot captured/).waitFor()
    await page.getByRole('button', { name: 'Run evaluation against snapshot' }).click()

    await page.getByRole('heading', { name: /Eight structural dimensions/ }).waitFor()
    assert.equal(await page.locator('.eval-dimension-card').count(), 8)
    await page.getByText('Execution completed').first().waitFor()
    await page.getByText('Structural fail').first().waitFor()
    await page.locator('.eval-dimension-card').getByText('Unavailable', { exact: true }).first().waitFor()
    await page.getByText('Structural pass').first().waitFor()
    assert.equal(await page.locator('.badge--good', { hasText: 'Structural pass' }).count(), 0)
    await page.getByText(/Human review: Not started/).first().waitFor()
    await page.getByText(/Independent scientific validation: Not performed/).first().waitFor()

    // 3. Run 2 (depth 2 + generated APO-03 hypotheses & APO-04 designs): tests newest-first ordering
    await page.getByRole('tab', { name: 'Benchmarks & Run Capture' }).click()
    await page.getByLabel('Find knowledge graph root').fill('DAE-04 matched')
    await page.getByRole('button', { name: 'Search graph roots' }).click()
    await page.getByRole('button', { name: 'DAE-04 matched hybrid search (8 evaluations)' }).click()
    await page.getByLabel('Generate APO-03 hypothesis proposals from root (requires depth = 2)').check()
    await page.getByLabel('Generate APO-04 experiment designs from root (requires depth = 2)').check()
    const captureResp2 = page.waitForResponse((res) => res.url().endsWith('/v1/eval/snapshots'))
    await page.getByRole('button', { name: 'Capture snapshot' }).click()
    const snapHttp2 = await captureResp2
    assert.ok([200, 201].includes(snapHttp2.status()), `Unexpected Run 2 /v1/eval/snapshots status ${snapHttp2.status()}: ${await snapHttp2.text()}`)
    await page.getByText(/Immutable snapshot captured/).waitFor()
    await page.getByRole('button', { name: 'Run evaluation against snapshot' }).click()

    await page.getByRole('heading', { name: /Eight structural dimensions/ }).waitFor()
    const historyRows = page.getByRole('table', { name: 'Evaluation run history' }).locator('tbody tr')
    assert.equal(await historyRows.count(), 2)
    await historyRows.first().getByText('2').first().waitFor()

    // 4. Live Area 3: Diagnostics across 8 dimensions and redacted snapshot evidence/provenance
    await page.getByRole('tab', { name: 'Dimension & Evidence Diagnostics' }).click()
    await page.getByRole('table', { name: 'Dimension structural checks' }).waitFor()
    await page
      .getByRole('table', { name: 'Supporting evidence relationships' })
      .or(page.getByText('No supporting evidence edges linked to the active selection.'))
      .waitFor()
    await page
      .getByRole('table', { name: 'Contextual relationships and unlinked snapshot evidence' })
      .waitFor()
    await page.getByRole('table', { name: 'Contradictory evidence relationships' })
      .or(page.getByText(/No contradictory evidence was recorded/))
      .waitFor()

    // 5. Live Area 4: Integrity raw-byte SHA-256 verification directly against /v1/artifacts/{report_sha256}
    await page.getByRole('tab', { name: 'Integrity, Snapshot & Audit' }).click()
    await page.getByRole('button', { name: 'Verify raw artifact bytes (Web Crypto)' }).click()
    await page.getByText(/Exact raw-byte SHA-256 verified/).waitFor()
    assert.equal(await page.getByRole('button', { name: 'Download verified report JSON' }).isEnabled(), true)

    // 6. Desktop (all 4 tabs) & Mobile responsiveness against live built dist
    for (const label of [
      'Benchmarks & Run Capture',
      'Runs & Report Overview',
      'Dimension & Evidence Diagnostics',
      'Integrity, Snapshot & Audit',
    ]) {
      await page.getByRole('tab', { name: label }).click()
      const desktopOverflow = await page.evaluate(() => {
        const offenders = [...document.querySelectorAll('*')]
          .filter((el) => el.getBoundingClientRect().right > window.innerWidth + 1)
          .slice(0, 8)
          .map((el) => `${el.tagName}.${el.className}:${Math.round(el.getBoundingClientRect().right)}>${window.innerWidth}`)
        return { ok: document.documentElement.scrollWidth <= window.innerWidth + 1, scrollWidth: document.documentElement.scrollWidth, innerWidth: window.innerWidth, offenders }
      })
      assert.equal(desktopOverflow.ok, true, `Desktop overflow on ${label}: ${JSON.stringify(desktopOverflow)}`)
    }
    const mobile = await browser.newPage({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true })
    mobile.on('pageerror', (error) => errors.push(error.message))
    await connect(mobile, origin, token)
    await mobile.getByRole('button', { name: 'Quality Evaluation', exact: true }).click()
    await mobile.getByRole('heading', { name: 'Structural completeness, not scientific truth.' }).waitFor()
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
        `Mobile horizontal overflow on live Area ${label}`,
      )
    }

    assert.deepEqual(errors, [])
    console.log('Live APO-11 Playwright E2E passed against Codex backend across all four areas, canonical dimensions, pass/fail/unavailable outcomes, Web Crypto raw-byte verification, HTTP 413 backend check, and 390x844 mobile layout.')
  } finally {
    await browser?.close()
    if (server.exitCode === null && server.signalCode === null) {
      const closed = once(server, 'exit')
      server.kill()
      await closed
    }
    await rm(dataDir, { recursive: true, force: true })
  }
}

main().catch((error) => {
  console.error(error)
  process.exitCode = 1
})
