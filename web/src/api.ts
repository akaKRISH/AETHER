export type Row = Record<string, unknown>

export interface Units extends Row {
  attempts: number
  wall_ms: number
  tokens: number
  micro_usd: number
}

export interface Campaign extends Row {
  id: string
  title: string
  objective: string
  state: string
  protocol_hash: string
  policy_hash: string
  evaluator_hash: string
  requires_replay: boolean
  caps: Units
}

export interface Claim extends Row {
  id: string
  statement: string
  state: string
}

export interface Dependency extends Row {
  claim_id: string
  prerequisite_id: string
}

export interface Evidence extends Row {
  id: string
  claim_id: string
  relation: string
  invalidated: boolean
  target: { kind: 'metric'; metric_id: string } | { kind: 'artifact'; attempt_id: string; sha256: string }
}

export interface Budget extends Row {
  caps: Units
  used: Units
  reserved: Units
  halted: boolean
}

export interface Snapshot {
  campaign: Campaign
  budget: Budget
  hypotheses: Row[]
  experiments: Row[]
  runs: Row[]
  attempts: Row[]
  metrics: Row[]
  artifacts: Row[]
  claims: Claim[]
  evidence: Evidence[]
  dependencies: Dependency[]
  verifications: Row[]
  contradictions: Row[]
  replications: Row[]
  events: Row[]
}

interface CampaignInputs {
  id: string
  title: string
  objective: string
  seed: number
  items: number
  attempts: number
  wall_ms: number
}

export type NewCampaign = CampaignInputs & (
  { algorithm: string; optimization?: never } |
  { algorithm?: never; optimization: { challenge_seed: number } }
)

export interface StudyReport {
  protocol: string
  evidence_scope: string
  caps: Row
  verification_reserve: Row
  weights: Row
  campaigns: Row[]
}

export class ApiError extends Error {
  readonly status: number

  constructor(status: number, message: string) {
    super(message)
    this.status = status
  }
}

export function message(error: unknown) {
  return error instanceof Error ? error.message : 'An unexpected error occurred.'
}

export function isRow(value: unknown): value is Row {
  return typeof value === 'object' && value !== null && !Array.isArray(value)
}

async function request(token: string, path: string, options?: RequestInit) {
  const headers = new Headers(options?.headers)
  headers.set('Authorization', `Bearer ${token}`)
  if (options?.body) headers.set('Content-Type', 'application/json')
  let response: Response
  try {
    response = await fetch(`/v1${path}`, { ...options, headers, redirect: 'error' })
  } catch (error) {
    if (options?.signal?.aborted) throw error
    throw new Error('The API could not be reached. Check the local service and try again.')
  }

  if (!response.ok) {
    if (response.status === 401 || response.status === 403) {
      throw new ApiError(response.status, 'API access denied. Check the Bearer token and permissions.')
    }
    let detail: unknown
    try {
      detail = await response.json()
    } catch {
      detail = null
    }
    const text = isRow(detail)
      ? [detail.message, detail.error].find((value) => typeof value === 'string')
      : null
    throw new ApiError(response.status, text || `API request failed (HTTP ${response.status}).`)
  }
  return response
}

async function json(token: string, path: string, options?: RequestInit): Promise<unknown> {
  const response = await request(token, path, options)
  try {
    return await response.json()
  } catch {
    throw new Error('The API returned an unreadable response. Check the service and proxy configuration.')
  }
}

export function rows(value: unknown): Row[] {
  return Array.isArray(value) ? value.filter(isRow) : []
}

function isCampaign(value: unknown): value is Campaign {
  return isRow(value) && typeof value.id === 'string' && typeof value.title === 'string'
    && typeof value.state === 'string' && typeof value.protocol_hash === 'string'
}

export async function listCampaigns(token: string, signal?: AbortSignal): Promise<Campaign[]> {
  const value = await json(token, '/campaigns', { signal })
  const list = isRow(value) ? value.campaigns : value
  if (!Array.isArray(list) || !list.every(isCampaign)) throw new Error('Invalid campaign list returned by the API.')
  return list
}

export function campaignPath(id: string) {
  return `/campaigns/${encodeURIComponent(id)}`
}

export async function getSnapshot(token: string, id: string, signal?: AbortSignal): Promise<Snapshot> {
  const value = await json(token, campaignPath(id), { signal })
  const body = isRow(value) && isRow(value.snapshot) ? { ...value.snapshot, events: value.events ?? value.snapshot.events } : value
  if (!isRow(body) || !isCampaign(body.campaign) || !isRow(body.budget)) {
    throw new Error('Invalid campaign snapshot returned by the API.')
  }

  const fields = [
    'hypotheses', 'experiments', 'runs', 'attempts', 'metrics', 'artifacts', 'claims', 'evidence',
    'dependencies', 'verifications', 'contradictions', 'replications', 'events',
  ] as const
  const lists = Object.fromEntries(fields.map((key) => [key, rows(body[key])]))
  return { ...body, ...lists } as unknown as Snapshot
}

export async function post(token: string, path: string, data?: unknown) {
  await request(token, path, { method: 'POST', body: data === undefined ? undefined : JSON.stringify(data) })
}

export async function getRecord(token: string, path: 'runtime' | 'literature' | 'hardware', signal: AbortSignal, query = ''): Promise<unknown> {
  const search = path === 'literature' && query ? `?${new URLSearchParams({ query })}` : ''
  return json(token, `/${path}${search}`, { signal })
}

export async function runStudy(token: string): Promise<StudyReport> {
  const result = await json(token, '/benchmarks', { method: 'POST' })
  const report = isRow(result) && isRow(result.report) ? result.report : result
  if (!isRow(report) || !Array.isArray(report.campaigns)) throw new Error('Invalid benchmark report returned by the API.')
  return report as unknown as StudyReport
}

export async function runEvolution(token: string, seed: number, evaluations: number): Promise<Row> {
  const result = await json(token, '/evolution', { method: 'POST', body: JSON.stringify({ seed, evaluations }) })
  if (!isRow(result) || result.scope !== 'aether.knapsack-development/1' || !Array.isArray(result.arms)) {
    throw new Error('Invalid evolution report returned by the API.')
  }
  return result
}

export async function proposeResearch(token: string, request: { objective: string; seed: number; items: number }) {
  return rows(await json(token, '/proposals', { method: 'POST', body: JSON.stringify(request) }))
}

export async function getReport(token: string, id: string): Promise<string> {
  const response = await request(token, `${campaignPath(id)}/report`)
  const contentType = response.headers.get('content-type') || ''
  if (contentType.includes('text/html')) throw new Error('The API returned an unexpected HTML report.')
  if (contentType.includes('application/json')) {
    const value: unknown = await response.json()
    if (typeof value === 'string') return value
    if (isRow(value) && typeof value.report === 'string') return value.report
    return JSON.stringify(value, null, 2)
  }
  return response.text()
}
