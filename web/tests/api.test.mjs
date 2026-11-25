import { strict as assert } from 'node:assert'
import { test } from 'node:test'
import { ApiError, getSnapshot, post, runStudy } from '../src/api.ts'

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
