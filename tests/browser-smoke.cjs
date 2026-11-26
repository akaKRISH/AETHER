const assert = require('node:assert/strict')
const { spawn } = require('node:child_process')
const { once } = require('node:events')
const { mkdir, mkdtemp, rm } = require('node:fs/promises')
const { join, resolve } = require('node:path')
const { tmpdir } = require('node:os')
const { randomBytes } = require('node:crypto')
const root = resolve(__dirname, '..')
const playwright = process.env.AETHER_PLAYWRIGHT_PATH || join(root, 'web', 'node_modules', 'playwright')
const { chromium } = require(playwright)
const executable = join(root, 'target', 'release', process.platform === 'win32' ? 'aether-lab.exe' : 'aether-lab')

function address(server) {
  return new Promise((resolveAddress, reject) => {
    let output = ''
    const timer = setTimeout(() => reject(new Error('local server startup timed out')), 12_000)
    server.once('error', error => { clearTimeout(timer); reject(error) })
    server.once('exit', code => { clearTimeout(timer); reject(new Error('local server exited: ' + code + '\n' + output)) })
    server.stderr.on('data', text => {
      output += text
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
  const directory = await mkdtemp(join(tmpdir(), 'aether-browser-smoke-'))
  const token = randomBytes(32).toString('hex')
  const server = spawn(executable, ['serve', '0'], {
    cwd: root,
    env: {
      ...process.env,
      AETHER_DATA_DIR: directory, AETHER_TOKEN: token,
      AETHER_ALLOW_CROSSREF: '0', AETHER_ALLOW_OPENALEX: '0', AETHER_ALLOW_ARXIV: '0',
    },
    stdio: ['ignore', 'ignore', 'pipe'],
    windowsHide: true,
  })
  let browser
  try {
    const origin = await address(server)
    assert.equal((await fetch(origin + '/v1/hardware')).status, 401)
    assert.equal((await fetch(origin + '/assets/%2e%2e%2findex.html')).status, 404)
    for (const provider of ['crossref', 'openalex', 'arxiv']) {
      const denied = await fetch(origin + '/v1/literature/lookup', {
        method: 'POST',
        headers: { Authorization: 'Bearer ' + token, 'Content-Type': 'application/json' },
        body: JSON.stringify({ provider, query: 'deterministic evaluation' }),
      })
      assert.equal(denied.status, 409)
    }
    browser = await chromium.launch({
      executablePath: process.env.AETHER_CHROMIUM_PATH || undefined,
    })
    const desktop = await browser.newPage({ viewport: { width: 1366, height: 900 } })
    const browserErrors = []
    desktop.on('pageerror', error => browserErrors.push(error.message))
    await connect(desktop, origin, token)
    await desktop.getByRole('button', { name: 'New campaign', exact: true }).click()
    await desktop.getByLabel('Campaign ID').fill('browser-smoke')
    await desktop.getByLabel('Title', { exact: true }).fill('Seeded sorting verification')
    await desktop.getByLabel('Research objective').fill('Check a reviewed sorting candidate, then replay and challenge the frozen result.')
    await desktop.getByRole('button', { name: 'Create draft' }).click()
    await desktop.getByRole('button', { name: 'Approve protocol' }).click()
    await desktop.getByRole('button', { name: 'Approve this hash' }).click()
    await desktop.getByRole('button', { name: 'Start campaign' }).click()

    await desktop.waitForFunction(async ({ origin, token }) => {
      const response = await fetch(origin + '/v1/campaigns/browser-smoke', {
        headers: { Authorization: 'Bearer ' + token },
      })
      const data = await response.json()
      return data.campaign?.state === 'completed'
        && data.claims?.some(claim => claim.state === 'verified')
        && data.dependencies?.length === 1
        && data.replications?.some(replay => replay.passed)
    }, { origin, token })

    await desktop.getByRole('button', { name: 'Evidence graph', exact: true }).click()
    await desktop.getByRole('heading', { name: 'Claim dependencies', exact: true }).waitFor()
    await desktop.locator('.campaign-meta').getByText('Completed', { exact: true }).waitFor()
    await desktop.getByRole('table', { name: 'Verifications' })
      .getByText('Verified', { exact: true }).waitFor()
    const verified = desktop.getByRole('region', { name: 'Interactive claim graph' })
      .getByRole('button', { name: 'verified browser-smoke-correct', exact: true })
    await verified.click()
    assert.equal(await verified.getAttribute('aria-pressed'), 'true')
    await desktop.getByRole('table', { name: 'Independent replays' })
      .getByText('Yes', { exact: true }).waitFor()
    if (process.env.AETHER_SCREENSHOT_DIR) {
      await mkdir(process.env.AETHER_SCREENSHOT_DIR, { recursive: true })
      await desktop.screenshot({ path: join(process.env.AETHER_SCREENSHOT_DIR, 'aether-evidence.png') })
    }
    await desktop.getByRole('button', { name: 'Report', exact: true }).click()
    await desktop.getByRole('button', { name: 'Load report' }).click()
    await desktop.getByRole('button', { name: 'Download JSON' }).waitFor()
    await desktop.getByRole('button', { name: 'Hypotheses', exact: true }).click()
    await desktop.getByRole('button', { name: 'Generate reviewed candidates' }).click()
    await desktop.getByRole('table', { name: 'Reviewable proposals' }).waitFor()
    await desktop.getByRole('button', { name: 'ECAR decisions', exact: true }).click()
    await desktop.getByRole('heading', { name: 'ECAR decision inspector' }).waitFor()
    await desktop.getByLabel('Decision event').selectOption({ index: 0 })
    await desktop.getByRole('table', { name: 'Candidate ranking' }).waitFor()
    await desktop.getByRole('button', { name: 'Verification', exact: true }).click()
    await desktop.getByRole('table', { name: 'Independent replay' }).waitFor()
    await desktop.getByRole('button', { name: 'Literature', exact: true }).click()
    await desktop.getByRole('link', { name: /CAVEAT/ }).waitFor()
    const caveat = desktop.locator('article').filter({ has: desktop.getByRole('link', { name: /CAVEAT/ }) })
    await caveat.getByText('Metadata versions and passages').click()
    await caveat.getByRole('table', { name: 'Metadata snapshots' }).waitFor()
    await caveat.getByText('Full text is unavailable in the local archive.').waitFor()
    await desktop.getByRole('button', { name: 'Benchmarks', exact: true }).click()
    const bench = desktop.waitForResponse(response => response.url().endsWith('/v1/benchmarks'))
    await desktop.getByRole('button', { name: 'Run default study' }).click()
    assert.equal((await bench).status(), 200)
    await desktop.getByRole('progressbar', { name: 'Mode D verified utility' }).waitFor()
    await desktop.getByRole('button', { name: 'Evolution explorer', exact: true }).click()
    await desktop.getByRole('button', { name: 'Compare search policies' }).click()
    const policies = desktop.getByRole('table', { name: 'Compared search policies' })
    await policies.waitFor()
    assert.equal(await policies.getByRole('row').count(), 5)
    await desktop.getByRole('table', { name: 'Candidate archive' }).waitFor()
    await desktop.getByRole('button', { name: 'Runtime', exact: true }).click()
    await desktop.getByRole('heading', { name: 'NVIDIA driver telemetry' }).waitFor()
    await desktop.getByRole('table', { name: 'Observed devices' })
      .or(desktop.getByText('NVIDIA driver telemetry is unavailable.')).waitFor()

    await desktop.getByRole('button', { name: 'Mission Control', exact: true }).click()
    await desktop.getByRole('heading', { name: 'Mission Control', exact: true }).waitFor()
    await desktop.getByRole('heading', { name: 'Campaign register', exact: true }).waitFor()
    await desktop.getByRole('button', { name: 'Discovery Laboratory', exact: true }).click()
    await desktop.getByRole('heading', { name: 'Search, inspect, replay.' }).waitFor()
    await desktop.getByRole('table', { name: 'Archived hybrid discovery comparison' }).waitFor()
    await desktop.getByRole('button', { name: 'Intelligence Observatory', exact: true }).click()
    await desktop.getByRole('heading', { name: 'Inference is advisory. Verification decides.' }).waitFor()
    await desktop.getByRole('table', { name: 'Local model inference strategies' }).waitFor()
    await desktop.getByRole('button', { name: 'Compute Observatory', exact: true }).click()
    await desktop.getByRole('heading', { name: 'Measure the control plane.' }).waitFor()
    await desktop.getByRole('table', { name: 'DAE-07 component measurements' }).waitFor()
    await desktop.getByRole('button', { name: 'Research Archive', exact: true }).click()
    await desktop.getByRole('heading', { name: 'Keep the negative results visible.' }).waitFor()
    await desktop.getByText('0.1731350994365088').waitFor()
    await desktop.getByRole('button', { name: 'Security Centre', exact: true }).click()
    await desktop.getByRole('heading', { name: 'Boundaries are part of the result.' }).waitFor()
    await desktop.getByText(/does not deny filesystem access or network egress/).waitFor()

    await desktop.getByRole('button', { name: 'Overview', exact: true }).click()
    await desktop.getByRole('button', { name: 'New campaign', exact: true }).click()
    await desktop.getByLabel('Campaign ID').fill('opt-browser')
    await desktop.getByLabel('Title', { exact: true }).fill('Maximum subarray verification')
    await desktop.getByLabel('Research objective').fill('Check the best contiguous interval and look for a singleton baseline counterexample.')
    await desktop.getByLabel('Task domain').selectOption('optimization')
    await desktop.getByLabel('Random seed').fill('19')
    await desktop.getByLabel('Challenge seed').fill('281')
    await desktop.getByRole('button', { name: 'Create draft' }).click()
    await desktop.getByRole('button', { name: 'Approve protocol' }).click()
    await desktop.getByRole('button', { name: 'Approve this hash' }).click()
    await desktop.getByRole('button', { name: 'Start campaign' }).click()
    await desktop.getByRole('button', { name: 'Verification', exact: true }).click()
    await desktop.locator('.campaign-meta').getByText('Completed', { exact: true }).waitFor()
    const verdicts = desktop.getByRole('table', { name: 'Verdict history' })
    await verdicts.getByText('Verified', { exact: true }).waitFor()
    await verdicts.getByText('Rejected', { exact: true }).waitFor()
    if (process.env.AETHER_SCREENSHOT_DIR) {
      await desktop.screenshot({ path: join(process.env.AETHER_SCREENSHOT_DIR, 'aether-optimization.png') })
    }

    const phone = await browser.newPage({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true })
    phone.on('pageerror', error => browserErrors.push(error.message))
    await connect(phone, origin, token)
    await phone.getByRole('heading', { name: 'Mission Control', exact: true }).waitFor()
    assert.equal(await phone.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1), true)
    if (process.env.AETHER_SCREENSHOT_DIR) {
      await phone.screenshot({ path: join(process.env.AETHER_SCREENSHOT_DIR, 'aether-mobile.png'), fullPage: true })
    }
    assert.deepEqual(browserErrors, [])
    console.log('Browser smoke passed: existing campaign workflows, all seven research command pages, archived pilot data, API-backed benchmark views, security limits, and mobile layout.')
  } finally {
    await browser?.close()
    if (server.exitCode === null && server.signalCode === null) {
      const closed = once(server, 'exit')
      server.kill()
      await closed
    }
    await rm(directory, { recursive: true, force: true })
  }
}

main().catch(error => { console.error(error); process.exitCode = 1 })
