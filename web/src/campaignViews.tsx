import { useState } from 'react'
import { isRow, rows, type Claim, type Evidence, type Row, type Snapshot, type Units } from './api'
import { Badge, Empty, Fields, Panel, RecordTable, text } from './components'

export const statusColumn = (key: string, label: string) => ({
  key, label,
  render: (value: unknown) => typeof value === 'string' ? <Badge value={value} /> : <span>—</span>,
})

const time = (value: unknown) => typeof value === 'number' && Number.isFinite(value)
  ? new Date(value).toLocaleString(undefined, { dateStyle: 'medium', timeStyle: 'medium' })
  : text(value)

const eventColumns = [
  { key: 'sequence', label: 'Seq' },
  'kind',
  { key: 'occurred_at_ms', label: 'Recorded', render: time },
  'detail',
]

export function Budgets({ snapshot }: { snapshot: Snapshot }) {
  const { used, reserved, caps, halted } = snapshot.budget
  const dimensions: { key: keyof Units; label: string; unit?: string }[] = [
    { key: 'attempts', label: 'Attempts' },
    { key: 'wall_ms', label: 'Time', unit: ' ms' },
    { key: 'tokens', label: 'Tokens' },
    { key: 'micro_usd', label: 'Paid cost', unit: ' µUSD' },
  ]
  return <Panel title="Resource budget" aside={halted ? <Badge value="halted" /> : null}>
    <div className="budget-list">
      {dimensions.map(({ key, label, unit = '' }) => {
        const spent = Number(used?.[key])
        const pending = Number(reserved?.[key])
        const cap = Number(caps?.[key])
        const present = [spent, pending, cap].every(Number.isFinite)
        return <div className="budget-item" key={key}>
          <div className="budget-item-label"><span>{label}</span>
            <strong className="mono">{present ? `${spent.toLocaleString()} / ${cap.toLocaleString()}${unit}` : '—'}</strong></div>
          {present && cap > 0
            ? <progress aria-label={`${label} budget used`} max={cap} value={Math.min(spent, cap)} />
            : <div className="budget-track" aria-hidden="true" />}
          {present && pending > 0 && <small>{pending.toLocaleString()}{unit} reserved</small>}
        </div>
      })}
    </div>
  </Panel>
}

export function Overview({ snapshot }: { snapshot: Snapshot }) {
  const campaign = snapshot.campaign
  const verified = snapshot.claims.filter((claim) => claim.state === 'verified').length
  const figures = [
    ['Experiments', snapshot.experiments.length],
    ['Attempts', snapshot.attempts.length],
    ['Verified claims', verified],
    ['Evidence links', snapshot.evidence.length],
  ] as const
  return <>
    <section className="figures" aria-label="Campaign counts">
      {figures.map(([label, value]) =>
        <div key={label} className="figure"><span>{label}</span><strong>{value.toLocaleString()}</strong></div>)}
    </section>
    <div className="dashboard-columns">
      <Panel title="Frozen protocol">
        <Fields data={{
          protocol_hash: campaign.protocol_hash,
          policy_hash: campaign.policy_hash,
          evaluator_hash: campaign.evaluator_hash,
          independent_replay_required: campaign.requires_replay,
        }} />
      </Panel>
      <Budgets snapshot={snapshot} />
    </div>
    <Panel title="Recent events">
      <RecordTable label="Recent events" data={[...snapshot.events].reverse().slice(0, 8)}
        columns={eventColumns} empty="No campaign events are available yet." />
    </Panel>
  </>
}

export function Experiments({ snapshot }: { snapshot: Snapshot }) {
  return <>
    <div className="dashboard-columns">
      <Panel title="Hypotheses">
        <RecordTable label="Hypotheses" data={snapshot.hypotheses} columns={['id', 'statement']} />
      </Panel>
      <Panel title="Experiment definitions">
        <RecordTable label="Experiment definitions" data={snapshot.experiments}
          columns={['id', 'task_key', 'hypothesis_id', 'definition_hash']} />
      </Panel>
    </div>
    <Panel title="Runs">
      <RecordTable label="Runs" data={snapshot.runs} columns={['id', 'action', 'experiment_id', 'parent_run_id', 'seed', 'input_hash']} />
    </Panel>
    <Panel title="Execution attempts">
      <RecordTable label="Execution attempts" data={snapshot.attempts}
        columns={['id', 'run_id', statusColumn('outcome', 'Outcome'), 'manifest_sha256', 'reason', 'usage']} />
    </Panel>
    <div className="dashboard-columns">
      <Panel title="Authoritative metrics">
        <RecordTable label="Authoritative metrics" data={snapshot.metrics}
          columns={['name', 'value', 'unit', 'attempt_id', 'evaluator_hash', 'split_hash']} />
      </Panel>
      <Panel title="Artifact inventory">
        <RecordTable label="Artifact inventory" data={snapshot.artifacts} columns={['name', 'size_bytes', 'media_type', 'sha256', 'attempt_id']} />
      </Panel>
    </div>
  </>
}

export function Events({ snapshot }: { snapshot: Snapshot }) {
  return <Panel title="Event journal" aside={<span className="muted mono">{snapshot.events.length} records</span>}>
    <RecordTable label="Event journal" data={[...snapshot.events].reverse()} columns={eventColumns} />
  </Panel>
}

export function Decisions({ snapshot }: { snapshot: Snapshot }) {
  const [sequence, setSequence] = useState('')
  const decisions = snapshot.events.filter((event) =>
    isRow(event.detail) && isRow(event.detail.trace),
  )
  const selected = decisions.find((event) => String(event.sequence) === sequence) ?? decisions.at(-1)
  const detail = isRow(selected?.detail) ? selected.detail : undefined
  const trace = isRow(detail?.trace) ? detail.trace : undefined
  return <Panel title="ECAR decision inspector">
    {trace ? <>
      <label>Decision event
        <select value={String(selected?.sequence)} onChange={(event) => setSequence(event.target.value)}>
          {decisions.map((event) => <option key={String(event.sequence)} value={String(event.sequence)}>Event {String(event.sequence)}</option>)}
        </select>
      </label>
      <Fields data={{ policy: detail?.policy, decision: trace.decision, dependent_claims: detail?.dependent_claims, resources_remaining: detail?.remaining }} />
      <RecordTable label="Candidate ranking" data={rows(trace.candidates)} columns={['action_id', 'kind', 'evaluation']} />
    </> : <Empty title="No scheduler decisions have been recorded." />}
  </Panel>
}

export function Verification({ snapshot }: { snapshot: Snapshot }) {
  return <>
    <Panel title="Current claim state">
      <RecordTable label="Current claim state" data={snapshot.claims}
        columns={['id', 'statement', statusColumn('state', 'State')]} />
    </Panel>
    <Panel title="Independent replay">
      <RecordTable label="Independent replay" data={snapshot.replications}
        columns={['original_attempt_id', 'replay_attempt_id', 'passed', 'reason']} />
    </Panel>
    <div className="dashboard-columns">
      <Panel title="Verdict history">
        <RecordTable label="Verdict history" data={snapshot.verifications}
          columns={['claim_id', statusColumn('verdict', 'Verdict'), 'rule_hash', 'evaluator_hash', 'reason']} />
      </Panel>
      <Panel title="Contradictions">
        <RecordTable label="Contradictions" data={snapshot.contradictions}
          columns={['claim_id', 'evidence_id', 'invalidates', 'reason']} />
      </Panel>
    </div>
  </>
}

type Node = {
  key: string
  item: Claim | Evidence
  kind: 'claim' | 'evidence'
  x: number
  y: number
}
const nodeWidth = 176
const nodeHeight = 58
const colGap = 72
const rowGap = 26

function evidenceLayout(snapshot: Snapshot) {
  const claims = new Map(snapshot.claims.map((claim) => [claim.id, claim]))
  const incoming = new Map(snapshot.claims.map((claim) => [claim.id, 0]))
  const children = new Map<string, string[]>()
  const rank = new Map(snapshot.claims.map((claim) => [claim.id, 0]))
  const dependencies = snapshot.dependencies.filter(({ claim_id, prerequisite_id }) =>
    claims.has(claim_id) && claims.has(prerequisite_id))

  for (const { claim_id, prerequisite_id } of dependencies) {
    incoming.set(claim_id, (incoming.get(claim_id) ?? 0) + 1)
    const group = children.get(prerequisite_id) ?? []
    group.push(claim_id)
    children.set(prerequisite_id, group)
  }

  const queue = snapshot.claims.filter((claim) => incoming.get(claim.id) === 0).map((claim) => claim.id)
  for (let index = 0; index < queue.length; index += 1) {
    const id = queue[index]
    for (const dependent of children.get(id) ?? []) {
      rank.set(dependent, Math.max(rank.get(dependent) ?? 0, (rank.get(id) ?? 0) + 1))
      const remaining = (incoming.get(dependent) ?? 0) - 1
      incoming.set(dependent, remaining)
      if (remaining === 0) queue.push(dependent)
    }
  }

  const rowsByColumn = new Map<number, number>()
  const evidenceColumn = Number(snapshot.evidence.length > 0)
  const nodes: Node[] = []
  function position(kind: Node['kind'], item: Node['item'], column: number) {
    const row = rowsByColumn.get(column) ?? 0
    rowsByColumn.set(column, row + 1)
    nodes.push({ key: `${kind}:${item.id}`, item, kind, x: 26 + column * (nodeWidth + colGap), y: 26 + row * (nodeHeight + rowGap) })
  }
  for (const evidence of snapshot.evidence) position('evidence', evidence, 0)
  for (const claim of snapshot.claims) position('claim', claim, (rank.get(claim.id) ?? 0) + evidenceColumn)

  const lookup = new Map(nodes.map((node) => [node.key, node]))
  const edges = [
    ...dependencies.map(({ prerequisite_id, claim_id }) => ({
      from: `claim:${prerequisite_id}`, to: `claim:${claim_id}`, style: 'dependency',
    })),
    ...snapshot.evidence.map((link) => ({
      from: `evidence:${link.id}`, to: `claim:${link.claim_id}`,
      style: link.invalidated ? 'invalid' : link.relation,
    })),
  ].flatMap((edge) => {
    const start = lookup.get(edge.from)
    const finish = lookup.get(edge.to)
    return start && finish ? [{ ...edge, start, finish }] : []
  })
  const bounds = nodes.reduce((size, node) => ({
    width: Math.max(size.width, node.x + nodeWidth + 26),
    height: Math.max(size.height, node.y + nodeHeight + 26),
  }), { width: 380, height: 230 })
  return { nodes, edges, ...bounds }
}

function EvidenceRecord({ evidence, snapshot }: { evidence: Evidence; snapshot: Snapshot }) {
  const target = evidence.target
  const source: Row | undefined = target?.kind === 'metric'
    ? snapshot.metrics.find((metric) => metric.id === target.metric_id)
    : target?.kind === 'artifact'
      ? snapshot.artifacts.find((artifact) => artifact.attempt_id === target.attempt_id && artifact.sha256 === target.sha256)
      : undefined
  return <>
    <Fields data={evidence} />
    {source && <details><summary>Recorded {target.kind}</summary><Fields data={source} /></details>}
  </>
}

export function EvidenceGraph({ snapshot }: { snapshot: Snapshot }) {
  const [selected, setSelected] = useState('')
  const graph = evidenceLayout(snapshot)
  const node = graph.nodes.find((item) => item.key === selected) ?? graph.nodes[0]
  const claim = node?.kind === 'claim' ? node.item as Claim : undefined
  const related = claim ? snapshot.evidence.filter((evidence) => evidence.claim_id === claim.id) : []

  return <>
    <div className="graph-grid">
      <Panel title="Claim dependencies" aside={<span className="mono muted">{graph.nodes.length} nodes · {graph.edges.length} links</span>}>
        {graph.nodes.length ? (
          // biome-ignore lint/a11y/noNoninteractiveTabindex: Users can scroll the graph before selecting a node.
          <section className="graph-scroll" tabIndex={0} aria-label="Interactive claim graph">
            <div className="graph-canvas" style={{ width: graph.width, height: graph.height }}>
              <svg className="graph-lines" width={graph.width} height={graph.height} aria-hidden="true">
                {graph.edges.map(({ from, to, start, finish, style }) => {
                  const x1 = start.x + nodeWidth, y1 = start.y + nodeHeight / 2
                  const x2 = finish.x, y2 = finish.y + nodeHeight / 2
                  const bend = Math.max(24, (x2 - x1) / 2)
                  return <path key={JSON.stringify([from, to])} className={`graph-link graph-link--${style}`}
                    d={`M ${x1} ${y1} C ${x1 + bend} ${y1}, ${x2 - bend} ${y2}, ${x2} ${y2}`} />
                })}
              </svg>
              {graph.nodes.map((item) => <button key={item.key} type="button"
                aria-pressed={item.key === node?.key}
                className={`graph-node graph-node--${item.kind}`}
                style={{ left: item.x, top: item.y, width: nodeWidth, height: nodeHeight }}
                onClick={() => setSelected(item.key)}>
                <span>{item.kind === 'claim' ? (item.item as Claim).state : (item.item as Evidence).relation}</span>
                <strong title={item.item.id}>{item.item.id}</strong>
              </button>)}
            </div>
          </section>
        ) : <Empty title="No claims or evidence have been recorded." />}
      </Panel>
      <Panel title="Evidence inspector">
        {!node ? <Empty title="Select a node once evidence is available." /> : claim
          ? <>
            <Fields data={claim} />
            <RecordTable label="Prerequisites for selected claim"
              data={snapshot.dependencies.filter((edge) => edge.claim_id === claim.id)}
              columns={[{ key: 'prerequisite_id', label: 'Depends on' }]} empty="No recorded prerequisites." />
            {related.map((evidence) => <details key={evidence.id}>
              <summary>{evidence.relation} · {evidence.id}</summary>
              <EvidenceRecord evidence={evidence} snapshot={snapshot} />
            </details>)}
          </>
          : <EvidenceRecord evidence={node.item as Evidence} snapshot={snapshot} />}
      </Panel>
    </div>
    <Panel title="Verifications">
      <RecordTable label="Verifications" data={snapshot.verifications}
        columns={['claim_id', statusColumn('verdict', 'Verdict'), 'reason', 'rule_hash', 'evaluator_hash', 'replay_attempt_id']} />
    </Panel>
    <div className="dashboard-columns">
      <Panel title="Independent replays">
        <RecordTable label="Independent replays" data={snapshot.replications} columns={['original_attempt_id', 'replay_attempt_id', 'passed', 'reason']} />
      </Panel>
      <Panel title="Contradictions">
        <RecordTable label="Contradictions" data={snapshot.contradictions} columns={['claim_id', 'evidence_id', 'invalidates', 'reason']} />
      </Panel>
    </div>
  </>
}
