import { useEffect, useState } from 'react'
import type { FormEvent } from 'react'
import { getExperimentDesigns, getGraphHypotheses, getKnowledge, message, rows, searchKnowledge, type Row } from './api'
import { Empty, Fields, Panel, RecordTable, errorText, pretty, readableRecord, text } from './components'

const kinds = ['paper', 'claim', 'method', 'dataset', 'metric', 'experiment', 'model', 'evidence',
  'limitation', 'hypothesis', 'research_question', 'campaign']

export function KnowledgeGraph({ token }: { token: string }) {
  const [filter, setFilter] = useState({ query: '', kind: '' })
  const [results, setResults] = useState<Row>()
  const [searchError, setSearchError] = useState('')
  const [selected, setSelected] = useState('')
  const [depth, setDepth] = useState('1')
  const [snapshot, setSnapshot] = useState<Row>()
  const [graphError, setGraphError] = useState('')

  useEffect(() => {
    const controller = new AbortController()
    searchKnowledge(token, filter.query, filter.kind, controller.signal).then(
      (value) => { if (!controller.signal.aborted) setResults(value) },
      (error) => { if (!controller.signal.aborted) setSearchError(message(error)) },
    )
    return () => controller.abort()
  }, [token, filter])

  useEffect(() => {
    if (!selected) return
    const controller = new AbortController()
    getKnowledge(token, selected, depth, controller.signal).then(
      (value) => { if (!controller.signal.aborted) setSnapshot(value) },
      (error) => { if (!controller.signal.aborted) setGraphError(message(error)) },
    )
    return () => controller.abort()
  }, [token, selected, depth])

  function search(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const form = new FormData(event.currentTarget)
    setResults(undefined)
    setSearchError('')
    setFilter({ query: String(form.get('query') ?? ''), kind: String(form.get('kind') ?? '') })
  }

  function choose(id: string) {
    if (id === selected) return
    setSnapshot(undefined)
    setGraphError('')
    setSelected(id)
  }

  const graph = readableRecord(snapshot?.graph)
  const nodes = rows(graph.nodes)
  const edges = rows(graph.edges)
  const current = readableRecord(nodes.find((node) => node.id === selected)?.current)
  const matches = rows(results?.nodes)
  const label = (id: unknown) => text(readableRecord(nodes.find((node) => node.id === id)?.current).label ?? id)
  function nodeLink(id: unknown) {
    return typeof id === 'string'
      ? <button type="button" className="button button--quiet" onClick={() => choose(id)}>{label(id)}</button>
      : text(id)
  }

  return <div className="dashboard-columns">
    <Panel title="Research entities">
      <form className="toolbar" onSubmit={search}>
        <label>Title prefix or node ID<input name="query" type="search" maxLength={128} /></label>
        <label>Entity type<select name="kind"><option value="">All types</option>
          {kinds.map((kind) => <option value={kind} key={kind}>{pretty(kind)}</option>)}
        </select></label>
        <button className="button" type="submit">Find entities</button>
      </form>
      {searchError ? errorText(searchError) : !results ? <p role="status">Loading entities…</p> : matches.length ? <>
        <RecordTable label="Knowledge graph search results" data={matches} columns={[
          { key: 'id', label: 'Entity', render: (id, row) => <button type="button" className="button button--quiet"
            aria-pressed={id === selected} onClick={() => choose(String(id))}>{text(readableRecord(row.current).label)}</button> },
          'kind', { key: 'current', label: 'Status', render: (value) => pretty(text(readableRecord(value).status)) },
        ]} />
        {results.truncated === true && <p className="muted">First 32 matches. Narrow the title or entity type.</p>}
      </> : <Empty title="No matching entities." />}
    </Panel>
    <Panel title="Graph inspector">
      {!selected ? <Empty title="Select an entity to explore its relationships." /> : <>
        <div className="toolbar">
          <label>Traversal depth<select value={depth} onChange={(event) => {
            setSnapshot(undefined); setGraphError(''); setDepth(event.target.value)
          }}>{['0', '1', '2', '3'].map((value) => <option key={value}>{value}</option>)}</select></label>
        </div>
        {graphError ? errorText(graphError) : !snapshot ? <p role="status">Loading graph…</p> : <>
          <h3>{text(current.label)}</h3>
          <p className="muted">Campaign observations show the status at capture.</p>
          <Fields data={current} />
          {graph.truncated === true && <p role="status">This view reached its result limit. Select a closer entity to continue.</p>}
          <RecordTable label="Knowledge relationships" data={edges} columns={[
            { key: 'origin', label: 'From', render: nodeLink }, 'relation',
            { key: 'target', label: 'To', render: nodeLink }, 'status', 'confidence_bps', 'source', 'rationale',
            'origin_revision', 'target_revision',
          ]} empty="No relationships in this traversal." />
          <RecordTable label="Unresolved citations" data={rows(graph.unresolved_citations)}
            columns={['target_identifier', 'relation', 'source']} empty="No unresolved citation links." />
          <details><summary>Revision history</summary>
            <RecordTable label="Entity versions" data={rows(graph.root_history)}
              columns={['label', 'statement', 'status', 'source', 'sha256']} />
          </details>
          <details><summary>Source groups and exact duplicates</summary>
            <Fields data={{ source_group: current.source_group, content_sha256: current.content_sha256 }} />
            <RecordTable label="Source and duplicate peers" data={[
              ...(Array.isArray(graph.source_group_peers) ? graph.source_group_peers : []).map((id) => ({ id, match: 'Same source group' })),
              ...(Array.isArray(graph.exact_duplicate_peers) ? graph.exact_duplicate_peers : []).map((id) => ({ id, match: 'Exact passage bytes' })),
            ]} columns={[{ key: 'id', label: 'Entity', render: nodeLink }, 'match']} empty="No matching peers." />
          </details>
          <Fields data={{ graph_sha256: snapshot.graph_sha256 }} />
          <GraphHypotheses key={selected} token={token} id={selected} />
        </>}
      </>}
    </Panel>
  </div>
}

function GraphHypotheses({ token, id }: { token: string; id: string }) {
  const [requested, setRequested] = useState(false)
  const [report, setReport] = useState<Row>()
  const [error, setError] = useState('')
  useEffect(() => {
    if (!requested) return
    const controller = new AbortController()
    getGraphHypotheses(token, id, controller.signal).then(
      (value) => { if (!controller.signal.aborted) setReport(value) },
      (reason) => { if (!controller.signal.aborted) setError(message(reason)) },
    )
    return () => controller.abort()
  }, [token, id, requested])
  if (!requested) return <div className="toolbar">
    <button className="button" type="button" onClick={() => setRequested(true)}>Discover hypotheses</button>
  </div>
  return <section aria-label="Hypothesis discovery">
    <h3>Ranked hypotheses</h3>
    {error ? errorText(error) : !report ? <p role="status">Scanning local evidence…</p> : <>
      <p className="panel-note">{text(report.coverage)}</p>
      <Fields data={{ graph_sha256: report.graph_sha256, ranking_formula: report.ranking_formula }} />
      {rows(report.proposals).length === 0 && <Empty title="No grounded hypothesis qualified in this graph slice." />}
      {rows(report.proposals).map((proposal) => <details key={text(proposal.id)}>
        <summary>{pretty(text(proposal.generator))} · Unverified · Score {text(readableRecord(proposal.ranking).score)}</summary>
        <p className="panel-note">{text(proposal.proposition)}</p>
        <Fields data={proposal} omit={['proposition', 'status', 'generator', 'supports', 'contradictory_evidence', 'context']} />
        <RecordTable label="Supporting evidence" data={rows(proposal.supports)}
          columns={['node', 'revision_sha256', 'status', 'source', 'statement']} />
        <RecordTable label="Counterevidence and opposing assertions" data={rows(proposal.contradictory_evidence)}
          columns={['node', 'revision_sha256', 'status', 'source', 'statement']} empty="No applicable counterevidence in this slice." />
        <RecordTable label="Scope and design context" data={rows(proposal.context)}
          columns={['node', 'revision_sha256', 'status', 'source', 'statement']} empty="No additional graph context." />
      </details>)}
      <details><summary>Scan diagnostics</summary><Fields data={readableRecord(report.diagnostics)} /></details>
      {currentArchive(report) && <ExperimentDesigns token={token} id={id} />}
    </>}
  </section>
}

function currentArchive(report: Row) {
  return rows(report.proposals).some((proposal) => rows(proposal.supports).some((reference) =>
    readableRecord(reference.source).artifact === 'dae-hybrid-development'))
}

function ExperimentDesigns({ token, id }: { token: string; id: string }) {
  const [requested, setRequested] = useState(false)
  const [report, setReport] = useState<Row>()
  const [error, setError] = useState('')
  useEffect(() => {
    if (!requested) return
    const controller = new AbortController()
    getExperimentDesigns(token, id, controller.signal).then(
      (value) => { if (!controller.signal.aborted) setReport(value) },
      (reason) => { if (!controller.signal.aborted) setError(message(reason)) },
    )
    return () => controller.abort()
  }, [id, requested, token])
  if (!requested) return <div className="toolbar">
    <button className="button" type="button" onClick={() => setRequested(true)}>Review experiment designs</button>
  </div>
  return <section aria-label="Experimental design review">
    <h3>Experimental design review</h3>
    {error ? errorText(error) : !report ? <p role="status">Loading experiment designs…</p> : <>
      {rows(report.designs).length === 0 && <Empty title="No experiment designs available." />}
      {rows(report.designs).map((design) => {
        const draft = readableRecord(design.protocol)
        return <details key={text(design.protocol_sha256)}>
          <summary>{pretty(text(draft.recipe))} · {pretty(text(draft.mode))} · {pretty(text(design.approval_status))}</summary>
          <p className="panel-note">{text(draft.proposition)}</p>
          <Fields data={{ protocol_sha256: design.protocol_sha256, ...draft }}
            omit={['schema', 'recipe', 'mode', 'proposition', 'supports', 'counterevidence', 'context']} />
          <RecordTable label="Design provenance" data={[
            ...rows(draft.supports).map((item) => ({ ...item, role: 'Support' })),
            ...rows(draft.counterevidence).map((item) => ({ ...item, role: 'Counterevidence' })),
            ...rows(draft.context).map((item) => ({ ...item, role: 'Context' })),
          ]} columns={['role', 'node', 'revision_sha256', 'source']} />
        </details>
      })}
    </>}
  </section>
}
