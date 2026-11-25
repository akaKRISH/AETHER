const assert = require('node:assert/strict')
const { execFileSync, spawn } = require('node:child_process')
const { once } = require('node:events')
const { mkdtemp, rm } = require('node:fs/promises')
const { join, resolve } = require('node:path')
const { arch, platform, tmpdir } = require('node:os')
const { randomBytes, createHash } = require('node:crypto')
const { readFileSync } = require('node:fs')
const { performance } = require('node:perf_hooks')

const root = resolve(__dirname, '..')
const exe = join(root, 'target', 'release', process.platform === 'win32' ? 'aether-lab.exe' : 'aether-lab')
const rounds = 50

function windowsPeakWorkingSet(pid) {
  if (platform() !== 'win32' || !process.env.SystemRoot) return null
  const powershell = join(process.env.SystemRoot, 'System32', 'WindowsPowerShell', 'v1.0', 'powershell.exe')
  const bytes = Number(execFileSync(powershell, [
    '-NoProfile', '-NonInteractive', '-Command', `(Get-Process -Id ${pid} -ErrorAction Stop).PeakWorkingSet64`,
  ], { encoding: 'utf8', windowsHide: true, timeout: 5000 }).trim())
  assert.ok(Number.isSafeInteger(bytes) && bytes > 0)
  return bytes
}

function startAddress(server) {
  return new Promise((resolveAddress, reject) => {
    let log = ''
    const timer = setTimeout(() => reject(new Error('release service startup timed out')), 12_000)
    server.once('error', (error) => { clearTimeout(timer); reject(error) })
    server.once('exit', (code) => { clearTimeout(timer); reject(new Error(`release service exited: ${code}\n${log}`)) })
    server.stderr.on('data', (chunk) => {
      log += chunk
      const match = log.match(/AETHER listening on (http:\/\/127\.0\.0\.1:\d+)/)
      if (!match) return
      clearTimeout(timer)
      resolveAddress(match[1])
    })
  })
}

async function measure(origin, token, path) {
  const samples = []
  let responseBytes
  for (let index = 0; index < rounds + 10; index++) {
    const start = performance.now()
    const response = await fetch(origin + path, {
      headers: { Authorization: `Bearer ${token}` },
      signal: AbortSignal.timeout(5000),
    })
    assert.equal(response.status, 200)
    responseBytes = (await response.arrayBuffer()).byteLength
    if (index >= 10) samples.push(performance.now() - start)
  }
  const ordered = [...samples].sort((left, right) => left - right)
  const percentile = (fraction) => ordered[Math.ceil(ordered.length * fraction) - 1]
  return { path, response_bytes: responseBytes, p50_ms: percentile(.5), p95_ms: percentile(.95), samples_ms: samples }
}

async function main() {
  const sandbox = await mkdtemp(join(tmpdir(), 'aether-loopback-timing-'))
  const token = randomBytes(32).toString('hex')
  const env = { ...process.env, AETHER_DATA_DIR: sandbox, AETHER_TOKEN: token }
  let service
  try {
    const setup = JSON.parse(execFileSync(exe, ['demo', 'latency-sample', '19'], { cwd: root, env, encoding: 'utf8' }))
    assert.equal(setup.evidence.campaign.state, 'completed')
    service = spawn(exe, ['serve', '0'], { cwd: root, env, windowsHide: true, stdio: ['ignore', 'ignore', 'pipe'] })
    const origin = await startAddress(service)
    const cases = []
    for (const path of ['/health', '/v1/campaigns/latency-sample', '/v1/campaigns/latency-sample/events?after=0']) {
      cases.push(await measure(origin, token, path))
    }
    console.log(JSON.stringify({
      scope: 'Sequential local development timings. Includes round trip and response body read.',
      host: { platform: platform(), arch: arch(), node: process.version },
      binary_sha256: createHash('sha256').update(readFileSync(exe)).digest('hex'),
      warmups_per_route: 10,
      samples_per_route: rounds,
      server_peak_working_set_bytes: windowsPeakWorkingSet(service.pid),
      cases,
    }, null, 2))
  } finally {
    if (service?.pid && service.exitCode === null && service.signalCode === null) {
      const done = once(service, 'exit')
      service.kill()
      await done
    }
    await rm(sandbox, { recursive: true, force: true })
  }
}

main().catch((error) => { console.error(error); process.exitCode = 1 })
