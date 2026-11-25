import { useEffect, useState } from 'react'
import type { FormEvent } from 'react'
import { getRecord, getReport, isRow, message, proposeResearch, rows, runEvolution, runStudy, type Row, type Snapshot, type StudyReport } from './api'
import { Empty, Fields, Panel, RecordTable, errorText, externalLink, keyedRows, readableRecord, text } from './components'

export function Hypotheses({ token, snapshot }: { token: string; snapshot: Snapshot }) {
  const [proposals, setProposals] = useState<Row[]>([])
  const [error, setError] = useState('')
  const [busy, setBusy] = useState(false)

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (busy) return
    const form = new FormData(event.currentTarget)
    setBusy(true)
    setError('')
    setProposals([])
    try {
      setProposals(await proposeResearch(token, {
        objective: String(form.get('objective') ?? '').trim(),
        seed: Number(form.get('seed')),
        items: Number(form.get('items')),
      }))
    } catch (reason) {
      setError(message(reason))
    } finally {
      setBusy(false)
    }
  }
  return <>
    <Panel title="Recorded hypotheses">
      <RecordTable label="Recorded hypotheses" data={snapshot.hypotheses} columns={['id', 'statement']} />
    </Panel>
    <Panel title="Offline candidate planner">
      <form className="modal-form" onSubmit={submit}>
        <label>Planning objective<input name="objective" required maxLength={512} defaultValue={snapshot.campaign.objective} /></label>
        <div className="form-columns">
          <label>Candidate seed<input name="seed" required type="number" min={0} max={4_294_967_295} defaultValue={19} /></label>
          <label>Candidate items<input name="items" required type="number" min={1} max={128} defaultValue={128} /></label>
        </div>
        <button className="button button--primary" type="submit" disabled={busy}>{busy ? 'Planning…' : 'Generate reviewed candidates'}</button>
      </form>
      {errorText(error)}
      <RecordTable label="Reviewable proposals" data={proposals} columns={['algorithm', 'seed', 'items', 'rationale']}
        empty="Candidate proposals appear here. Use a new campaign draft to approve one for execution." />
    </Panel>
  </>
}

export function Catalog({ token, kind }: { token: string; kind: 'runtime' | 'literature' }) {
  const [data, setData] = useState<unknown>()
  const [error, setError] = useState('')
  const [request, setRequest] = useState(0)
  const [query, setQuery] = useState('')
  // biome-ignore lint/correctness/useExhaustiveDependencies: Refresh repeats the lookup even when the query is unchanged.
  useEffect(() => {
    const controller = new AbortController()
    getRecord(token, kind, controller.signal, query).then(
      (value) => { if (!controller.signal.aborted) setData(value) },
      (reason) => { if (!controller.signal.aborted) setError(message(reason)) },
    )
    return () => controller.abort()
  }, [token, kind, query, request])

  function reload() {
    setData(undefined)
    setError('')
    setRequest((count) => count + 1)
  }

  function search(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    setQuery(String(new FormData(event.currentTarget).get('query') ?? '').trim())
    reload()
  }

  const entries = rows(isRow(data) ? data.entries ?? data.items ?? data.sources : data)
  return <Panel title={kind === 'runtime' ? 'Local execution environment' : 'Research sources'} aside={
    <button type="button" className="button button--quiet" onClick={reload}>Refresh</button>
  }>
    {kind === 'literature' && <form className="toolbar" onSubmit={search}>
      <label>Search local citations<input name="query" type="search" maxLength={120} /></label>
      <button className="button" type="submit">Search</button>
    </form>}
    {error ? errorText(error) : data === undefined
      ? <p className="loading" role="status">Loading {kind}…</p>
      : kind === 'runtime'
        ? <Fields data={readableRecord(data)} />
        : entries.length
          ? <div className="bibliography">
            {keyedRows(entries).map(({ row: entry, key }) => {
              const url = externalLink(entry.url ?? entry.link ?? entry.source_url ?? entry.doi)
              const title = text(entry.title ?? entry.name ?? entry.id ?? url)
              return <article className="source-entry" key={key}>
                <h3>{url ? <a href={url} target="_blank" rel="noopener noreferrer">{title} ↗</a> : title}</h3>
                <Fields data={entry} omit={['title', 'name', 'url', 'link', 'source_url']} />
              </article>
            })}
          </div>
          : <Empty title="The service returned no literature records." />}
  </Panel>
}

export function GpuTelemetry({ token }: { token: string }) {
  const [request, setRequest] = useState(0)
  const [data, setData] = useState<Row>()
  const [error, setError] = useState('')

  // biome-ignore lint/correctness/useExhaustiveDependencies: The button requests a fresh driver observation.
  useEffect(() => {
    const controller = new AbortController()
    getRecord(token, 'hardware', controller.signal).then(
      (value) => {
        if (!controller.signal.aborted) setData(readableRecord(value))
      },
      (reason) => {
        if (!controller.signal.aborted) setError(message(reason))
      },
    )
    return () => controller.abort()
  }, [token, request])

  return <Panel title="NVIDIA driver telemetry" aside={
    <button type="button" className="button button--quiet" onClick={() => {
      setData(undefined)
      setError('')
      setRequest((count) => count + 1)
    }}>Refresh GPU reading</button>
  }>
    {error ? errorText(error) : data === undefined
      ? <p className="loading" role="status">Reading driver telemetry…</p>
      : <RecordTable label="Observed devices" data={rows(data.devices)}
          columns={['name', 'driver', 'memory_total_mib', 'memory_free_mib', 'utilization_percent']}
          empty="NVIDIA driver telemetry is unavailable." />}
  </Panel>
}

const modes = ['A', 'B', 'C', 'D', 'E']

function Comparison({ results }: { results: Row[] }) {
  const values = modes.flatMap((mode) => {
    const group = results.filter((row) => row.mode === mode)
    const points = group.reduce((sum, row) => sum + (typeof row.verified_utility_points === 'number' ? row.verified_utility_points : 0), 0)
    const max = group.reduce((sum, row) => sum + (typeof row.maximum_utility_points === 'number' ? row.maximum_utility_points : 0), 0)
    return group.length && max > 0 ? [{ mode, points, max, samples: group.length }] : []
  })
  return values.length ? <div className="comparison">
    {values.map(({ mode, points, max, samples }) => <div key={mode} className="comparison-row">
      <strong>Mode {mode}</strong>
      <progress aria-label={`Mode ${mode} verified utility`} max={max} value={points} />
      <span className="mono">{points}/{max} · {samples} campaigns</span>
    </div>)}
  </div> : <Empty title="The benchmark returned no comparable utility scores." />
}

export function Benchmarks({ token }: { token: string }) {
  const [report, setReport] = useState<StudyReport>()
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [mode, setMode] = useState('')
  const [seed, setSeed] = useState('')
  const seeds = [...new Set(report?.campaigns.map((row) => text(row.seed)) ?? [])]
  const availableModes = [...new Set(report?.campaigns.map((row) => text(row.mode)) ?? [])]
  const filtered = report?.campaigns.filter((row) => (!mode || row.mode === mode) && (!seed || text(row.seed) === seed)) ?? []

  async function submit() {
    setError('')
    setBusy(true)
    try {
      setReport(await runStudy(token))
      setMode('')
      setSeed('')
    } catch (reason) {
      setError(message(reason))
    } finally {
      setBusy(false)
    }
  }
  return <>
    <Panel title="Run a scheduler benchmark">
      <div className="toolbar">
        <button type="button" onClick={() => void submit()} className="button button--primary" disabled={busy}>{busy ? 'Running…' : 'Run default study'}</button>
      </div>
      {errorText(error)}
    </Panel>
    {report && <>
      <Panel title="Verified utility">
        <p className="panel-note">{report.evidence_scope}</p>
        <div className="toolbar">
          <label>Mode<select value={mode} onChange={(event) => setMode(event.target.value)}>
            <option value="">All modes</option>
            {availableModes.map((value) => <option key={value}>{value}</option>)}
          </select></label>
          <label>Seed<select value={seed} onChange={(event) => setSeed(event.target.value)}>
            <option value="">All seeds</option>
            {seeds.map((value) => <option key={value}>{value}</option>)}
          </select></label>
        </div>
        <Comparison results={filtered} />
      </Panel>
      <Panel title="Recorded benchmark runs">
        <RecordTable label="Recorded benchmark runs" data={filtered}
          columns={['mode', 'task', 'seed', 'verified_utility_points', 'verified_claims', 'rejected_claims', 'inconclusive_claims', 'usage', 'stop_reason']} />
      </Panel>
      <Panel title="Benchmark parameters">
        <Fields data={{ protocol: report.protocol, caps: report.caps, verification_reserve: report.verification_reserve, weights: report.weights }} />
      </Panel>
    </>}
  </>
}

export function Evolution({ token }: { token: string }) {
  const [report, setReport] = useState<Row>()
  const [arm, setArm] = useState('random')
  const [error, setError] = useState('')
  const [busy, setBusy] = useState(false)
  const arms = rows(report?.arms)
  const selected = arms.find((entry) => entry.mode === arm)

  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    if (busy) return
    const form = new FormData(event.currentTarget)
    setError('')
    setBusy(true)
    try {
      setReport(await runEvolution(token, Number(form.get('seed')), Number(form.get('evaluations'))))
    } catch (reason) {
      setError(message(reason))
    } finally {
      setBusy(false)
    }
  }

  return <>
    <Panel title="Knapsack candidate search">
      <p className="panel-note">Development comparison on generated items. The oracle checks the optimum and reports the difference from each method’s best value.</p>
      <form className="toolbar" onSubmit={submit}>
        <label>Fixture seed<input name="seed" type="number" min={0} max={4_294_967_295} step={1} defaultValue={19} required /></label>
        <label>Evaluation slots per policy<input name="evaluations" type="number" min={8} max={96} step={1} defaultValue={48} required /></label>
        <button className="button button--primary" type="submit" disabled={busy}>{busy ? 'Running…' : 'Compare search policies'}</button>
      </form>
      {errorText(error)}
    </Panel>
    {report && <>
      <Panel title="Search results">
        <Fields data={{ seed: report.seed, capacity: report.capacity, oracle_optimal_value: report.oracle_optimal_value }} />
        <RecordTable label="Compared search policies" data={arms}
          columns={['mode', 'proposal_count', 'verification_count', 'unique_genotypes', 'best', 'verified', 'value_regret']} />
      </Panel>
      <Panel title="Candidate archive">
        <label>Search policy<select value={arm} onChange={(event) => setArm(event.target.value)}>
          {arms.map((entry) => text(entry.mode)).map((mode) => <option key={mode} value={mode}>{mode.replaceAll('_', ' ')}</option>)}
        </select></label>
        <RecordTable label="Candidate archive" data={rows(selected?.archive)}
          columns={['kind', 'candidate', 'parent_slot', 'disposition', 'improves_incumbent', 'candidate_slot', 'passed']} />
      </Panel>
    </>}
  </>
}

export function Report({ token, campaignId }: { token: string; campaignId: string }) {
  const [report, setReport] = useState('')
  const [error, setError] = useState('')
  const [busy, setBusy] = useState(false)

  async function load() {
    setBusy(true)
    setError('')
    try {
      setReport(await getReport(token, campaignId))
    } catch (reason) {
      setError(message(reason))
    } finally {
      setBusy(false)
    }
  }

  function download() {
    const url = URL.createObjectURL(new Blob([report], { type: 'application/json' }))
    const link = document.createElement('a')
    link.href = url
    link.download = `${campaignId}-report.json`
    link.click()
    window.setTimeout(() => URL.revokeObjectURL(url), 200)
  }

  return <Panel title="Campaign report" aside={<div className="actions">
    <button className="button" type="button" disabled={busy} onClick={() => void load()}>{busy ? 'Loading…' : 'Load report'}</button>
    {report && <button className="button button--primary" type="button" onClick={download}>Download JSON</button>}
  </div>}>
    {errorText(error)}
    {report ? (
      // biome-ignore lint/a11y/noNoninteractiveTabindex: Keep the raw export scrollable with the keyboard.
      <section className="report-view" tabIndex={0} aria-label="Loaded campaign report"><pre>{report}</pre></section>
    ) : <Empty title="Load the campaign report to inspect its protocol, evidence and limitations." />}
  </Panel>
}
