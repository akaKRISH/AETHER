const assert = require('node:assert/strict')
const { spawn } = require('node:child_process')
const { once } = require('node:events')
const { mkdir, mkdtemp, rm } = require('node:fs/promises')
const { join, resolve } = require('node:path')
const { tmpdir } = require('node:os')
const { randomBytes } = require('node:crypto')
const { chromium } = require(process.env.AETHER_PLAYWRIGHT_PATH || 'playwright')

const root = resolve(__dirname, '..')
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
  await page.getByRole('button', { name: 'Connect', exact: true }).click()
  await page.getByRole('button', { name: 'Disconnect' }).waitFor()
}

async function main() {
  const directory = await mkdtemp(join(tmpdir(), 'aether-browser-smoke-'))
  const token = randomBytes(32).toString('hex')
  const server = spawn(executable, ['serve', '0'], {
    cwd: root,
    env: { ...process.env, AETHER_DATA_DIR: directory, AETHER_TOKEN: token },
    stdio: ['ignore', 'ignore', 'pipe'],
    windowsHide: true,
  })
  let browser
  try {
    const origin = await address(server)
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
    await desktop.getByRole('button', { name: 'Benchmarks', exact: true }).click()
    const bench = desktop.waitForResponse(response => response.url().endsWith('/v1/benchmarks'))
    await desktop.getByRole('button', { name: 'Run default study' }).click()
    assert.equal((await bench).status(), 200)
    await desktop.getByRole('progressbar', { name: 'Mode D verified utility' }).waitFor()

    const phone = await browser.newPage({ viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true })
    phone.on('pageerror', error => browserErrors.push(error.message))
    await connect(phone, origin, token)
    await phone.getByRole('heading', { name: 'Seeded sorting verification', exact: true }).waitFor()
    assert.equal(await phone.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth + 1), true)
    if (process.env.AETHER_SCREENSHOT_DIR) {
      await phone.screenshot({ path: join(process.env.AETHER_SCREENSHOT_DIR, 'aether-mobile.png'), fullPage: true })
    }
    assert.deepEqual(browserErrors, [])
    console.log('Browser smoke passed: authentication, campaign, evidence, verification, planner, ECAR traces, report, literature, benchmark, and mobile layout.')
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
