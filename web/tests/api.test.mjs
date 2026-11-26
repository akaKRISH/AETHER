import { strict as assert } from 'node:assert'
import { test } from 'node:test'
import { ApiError, getResearchArchive, getResearchArtifact, getSnapshot, post, runStudy } from '../src/api.ts'

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
    (error) => error instanceof ApiError && error.status === 409
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
