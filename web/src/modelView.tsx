import { useEffect, useState } from 'react'
import { getModelComparison, getModelRegistry, getModelReport, message, rows, type Row } from './api'
import { Empty, Fields, Panel, RecordTable, errorText, readableRecord, text } from './components'

export function ModelLaboratory({ token }: { token: string }) {
  const [registry, setRegistry] = useState<Row>()
  const [error, setError] = useState('')
  const [baseline, setBaseline] = useState('')
  const [candidate, setCandidate] = useState('')
  const [selected, setSelected] = useState<{ report: string } | { pair: [string, string] }>()
  const [detail, setDetail] = useState<Row>()
  const [detailError, setDetailError] = useState('')
  useEffect(() => {
    const controller = new AbortController()
    getModelRegistry(token, controller.signal).then(
      (value) => { if (!controller.signal.aborted) setRegistry(value) },
      (reason) => { if (!controller.signal.aborted) setError(message(reason)) },
    )
    return () => controller.abort()
  }, [token])
  useEffect(() => {
    if (!selected) return
    const controller = new AbortController()
    const request = 'report' in selected
      ? getModelReport(token, selected.report, controller.signal)
      : getModelComparison(token, ...selected.pair, controller.signal)
    request.then(
      (value) => { if (!controller.signal.aborted) setDetail(value) },
      (reason) => { if (!controller.signal.aborted) setDetailError(message(reason)) },
    )
    return () => controller.abort()
  }, [selected, token])

  function inspect(choice: NonNullable<typeof selected>) {
    setDetail(undefined)
    setDetailError('')
    setSelected(choice)
  }

  const reports = rows(registry?.reports)
  const choices = reports.filter((row) => row.complete === true).map((row) => {
    const hash = text(row.artifact_sha256)
    return <option key={hash} value={hash}>{text(row.provider)} · {hash.slice(0, 12)}</option>
  })
  return <Panel title="Open-weight model laboratory">
    {error ? errorText(error) : !registry ? <p role="status">Loading local model registry…</p> : <>
      {rows(registry.entries).length ? rows(registry.entries).map((item) =>
        <details key={text(item.provenance ?? item.identity_sha256) + text(item.manifest_sha256)}>
          <summary>{text(item.alias ?? item.file_name ?? item.identity_sha256)} · {text(item.status ?? item.integrity)}</summary>
          <Fields data={item} omit={['inspection']} />
          <Fields data={readableRecord(item.inspection)} omit={['tensors']} />
          <RecordTable label="Tensor shapes" data={rows(readableRecord(item.inspection).tensors)}
            columns={['name', 'dtype', 'shape', 'bytes']} empty="Tensor descriptors unavailable." />
        </details>) : <Empty title="No local models have been indexed." />}
      {registry.truncated === true && <p role="status">Showing a bounded set of saved records.</p>}
      <RecordTable label="Saved model evaluations" data={reports} columns={['provider', 'complete', 'summary',
        { key: 'artifact_sha256', label: 'Report', render: (value) =>
          <button className="button button--quiet" type="button" onClick={() => inspect({ report: text(value) })}>Read report</button> },
      ]} empty="No model evaluation reports have been saved." />
      {choices.length >= 2 && <div className="toolbar">
        <label>Baseline report<select value={baseline} onChange={(event) => setBaseline(event.target.value)}>
          <option value="">Select baseline</option>{choices}
        </select></label>
        <label>Candidate report<select value={candidate} onChange={(event) => setCandidate(event.target.value)}>
          <option value="">Select candidate</option>{choices}
        </select></label>
        <button className="button" type="button" disabled={!baseline || !candidate || baseline === candidate}
          onClick={() => inspect({ pair: [baseline, candidate] })}>Compare reports</button>
      </div>}
      {detailError ? errorText(detailError) : detail ? <>
        <Fields data={detail} omit={['rows', 'paired']} />
        {'paired' in detail ? <RecordTable label="Paired model results" data={rows(detail.paired)}
          columns={['case', 'baseline_passed', 'candidate_passed', 'regression', 'baseline_latency_ms', 'candidate_latency_ms']} />
          : <RecordTable label="Model evaluation cases" data={rows(detail.rows)}
            columns={['case', 'outcome', 'passed', 'latency_ms', 'usage', 'memory_before', 'memory_after']}
            empty="No inference calls were recorded for this run." />}
      </> : selected && <p role="status">Loading model research record…</p>}
    </>}
  </Panel>
}
