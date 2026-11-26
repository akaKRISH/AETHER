import { useState } from 'react'
import { getResearchArtifact, isRow, rows, type Campaign, type ResearchArchive, type Row, type Snapshot } from './api'
import { Badge, Empty, Fields, Panel, RecordTable, errorText, text } from './components'
import { Evolution } from './consoleViews'
import { ModelLaboratory, TrainingProposals } from './modelView'
import { DatasetLaboratory } from './datasetView'
import { ResearchSkills } from './skillView'

const numeric = (value: unknown) => typeof value === 'number' && Number.isFinite(value) ? value : null
const fmt = (value: unknown, digits = 2) => numeric(value)?.toLocaleString(undefined, { maximumFractionDigits: digits }) ?? '—'
const records = (value: unknown) => rows(value)

export function MissionControl({ campaigns, snapshot, select }: {
  campaigns: Campaign[]; snapshot?: Snapshot; select: (id: string) => void
}) {
  const active = campaigns.filter((item) => ['approved', 'running', 'paused'].includes(item.state.toLowerCase())).length
  const verified = snapshot?.claims.filter((claim) => claim.state === 'verified').length ?? 0
  const rejected = snapshot?.claims.filter((claim) => ['rejected', 'inconclusive'].includes(claim.state)).length ?? 0
  return <>
    <section className="mission-lead">
      <div><p className="eyebrow">Research operations / 01</p><h2>Evidence before confidence.</h2>
        <p>Track frozen objectives, campaign state and verified evidence from the local AETHER controller.</p></div>
      <div className="mission-current"><span>Selected objective</span>
        <strong>{snapshot?.campaign.title ?? 'No campaign selected'}</strong>
        <small>{snapshot?.campaign.objective ?? 'Create a draft to begin a reviewed experiment.'}</small>
      </div>
    </section>
    <section className="figures mission-stats" aria-label="Current campaign summary">
      {[['Campaigns', campaigns.length], ['Active or approved', active], ['Verified claims', verified], ['Rejected or inconclusive', rejected]].map(([label, value]) =>
        <div key={label} className="figure"><span>{label}</span><strong>{value}</strong></div>)}
    </section>
    <Panel title="Campaign register" aside={<span className="mono muted">{campaigns.length} records</span>}>
      {campaigns.length ? <div className="campaign-register">
        {campaigns.map((campaign) => <button className="campaign-row" type="button" key={campaign.id} onClick={() => select(campaign.id)}>
          <span className="campaign-row-title"><strong>{campaign.title}</strong><code>{campaign.id}</code></span>
          <span className="campaign-row-objective">{campaign.objective}</span><Badge value={campaign.state} />
        </button>)}
      </div> : <Empty title="No campaigns are available." />}
    </Panel>
    {snapshot && <div className="dashboard-columns">
      <Panel title="Frozen objective"><Fields data={{ objective: snapshot.campaign.objective, protocol_sha256: snapshot.campaign.protocol_hash,
        policy_sha256: snapshot.campaign.policy_hash, evaluator_sha256: snapshot.campaign.evaluator_hash }} /></Panel>
      <Panel title="Current budget"><Fields data={{ caps: snapshot.budget.caps, used: snapshot.budget.used,
        reserved: snapshot.budget.reserved, halted: snapshot.budget.halted }} /></Panel>
    </div>}
    {snapshot && <Panel title="Recent journal entries"><RecordTable label="Recent journal entries"
      data={[...snapshot.events].reverse().slice(0, 6)} columns={['sequence', 'kind', 'detail']} empty="No events have been recorded." /></Panel>}
  </>
}

function usageRows(trials: Row[]) {
  return trials.map((entry) => {
    const report = isRow(entry.report) ? entry.report : entry
    const model = isRow(report.model_used) ? report.model_used : {}
    const limits = isRow(report.limits) && isRow(report.limits.model) ? report.limits.model : {}
    const verifier = isRow(report.verifier_used) ? report.verifier_used : {}
    return { strategy: report.strategy, provider: report.provider, calls: model.attempts,
      calls_cap: limits.attempts, token_cap: limits.model_tokens, tokens: model.model_tokens,
      checks: verifier.attempts, verified_candidate: report.verified_candidate !== null,
      stop: report.stop, wall_ms: model.wall_ms }
  })
}

export function IntelligenceObservatory({ archive, token }: { archive: ResearchArchive; token: string }) {
  const fixture = records(archive.inference_fixture.trials)
  const localStudy = isRow(archive.inference_local.study) ? archive.inference_local.study : {}
  const local = records(localStudy.trials)
  const liveRows = usageRows(local)
  const liveVerified = liveRows.filter((row) => row.verified_candidate === true).length
  const liveTokens = liveRows.reduce((sum, row) => sum + (numeric(row.tokens) ?? 0), 0)
  const selected = local.find((entry) => isRow(entry.report) && entry.report.strategy === 'adaptive')
  const trace = isRow(selected?.report) ? records(selected.report.steps) : []
  return <>
    <section className="mission-lead compact-lead"><div><p className="eyebrow">Model oversight / Historical pilot</p>
      <h2>Inference is advisory. Verification decides.</h2><p>Token and action records are from the saved DAE-06 fixture and local Qwen pilot; no live model is running here.</p></div>
      <div className="mission-current"><span>Local Qwen · one development task</span><strong>{liveVerified} / {liveRows.length} verified</strong>
        <small>{liveTokens.toLocaleString()} reported model tokens · reasoning tokens were unavailable</small></div>
    </section>
    <ModelLaboratory token={token} />
    <TrainingProposals token={token} />
    <DatasetLaboratory token={token} />
    <ResearchSkills token={token} />
    <div className="dashboard-columns">
      <Panel title="Local model pilot" aside={<Badge value="historical" />}>
        {liveRows.length ? <RecordTable label="Local model inference strategies" data={liveRows}
          columns={['strategy', 'calls', 'calls_cap', 'tokens', 'token_cap', 'checks', 'verified_candidate', 'stop', 'wall_ms']} />
          : <Empty title="The local model report has no trial rows." />}
        <p className="panel-note">All four strategies proposed the same suboptimal interval; the independent verifier rejected each. The saved pilot used the cloud-disabled Qwen service.</p>
      </Panel>
      <Panel title="Deterministic strategy fixture" aside={<span className="mono muted">{fixture.length} trials</span>}>
        <RecordTable label="Deterministic inference strategies" data={usageRows(fixture)}
          columns={['strategy', 'calls', 'calls_cap', 'tokens', 'token_cap', 'checks', 'verified_candidate', 'stop']} empty="No fixture trials are archived." />
        <p className="panel-note">Scripted provider usage. Model inference, native execution and trusted verification have separate budget records.</p>
      </Panel>
    </div>
    <Panel title="Adaptive decision trace" aside={<span className="muted">Saved Qwen trial · no inference replay</span>}>
      <RecordTable label="Adaptive inference decisions" data={trace}
        columns={['action', 'parent', 'outcome', 'candidate', 'wall_ms', 'usage']} empty="No adaptive trace was recorded." />
    </Panel>
    <Panel title="Authority boundary"><div className="authority-strip">
      <span>Model output → typed candidate</span><b aria-hidden="true">→</b><span>Trusted evaluator → verdict</span><b aria-hidden="true">→</b><span>Controller → stop or continue</span>
    </div></Panel>
  </>
}

export function ComputeObservatory({ archive }: { archive: ResearchArchive }) {
  const performance = archive.performance
  const metrics = records(performance.metrics)
  const max = Math.max(1, ...metrics.map((row) => numeric(row.p95_us) ?? 0))
  const host = isRow(performance.host_measurement) ? performance.host_measurement : {}
  return <>
    <section className="mission-lead compact-lead"><div><p className="eyebrow">Compute observatory / DAE-07</p>
      <h2>Measure the control plane.</h2><p>Recorded native release measurements, not a live telemetry feed. Latency is separated by component.</p></div>
      <div className="mission-current"><span>Recorded run</span><strong>{text(archive.recorded_on)}</strong>
        <small>{text(performance.suite_wall_seconds)} s measured phase · {text(performance.build && isRow(performance.build) ? performance.build.profile : null)} build</small></div>
    </section>
    <section className="host-strip" aria-label="Benchmark host details">
      <span><small>Processor</small><strong>{text(host.processor)}</strong></span>
      <span><small>Logical CPUs</small><strong>{text(host.logical_processors)}</strong></span>
      <span><small>Working set, sampled</small><strong>{fmt((numeric(host.peak_working_set_bytes_sampled) ?? 0) / 1_048_576)} MiB</strong></span>
      <span><small>Parent CPU share</small><strong>{fmt(host.cpu_percent_of_machine_capacity)}% of machine</strong></span>
    </section>
    <Panel title="Component latency" aside={<span className="muted">p50 and p95 · microseconds</span>}>
      {metrics.length ? <>
        <div className="latency-chart" aria-hidden="true">{metrics.map((metric) => <div className="latency-row" key={String(metric.name)}>
          <span>{String(metric.name).replaceAll('_', ' ')}</span><i style={{ width: `${Math.max(1, 100 * (numeric(metric.p95_us) ?? 0) / max)}%` }} />
          <code>{fmt(metric.p50_us, 1)} / {fmt(metric.p95_us, 1)}</code>
        </div>)}</div>
        <RecordTable label="DAE-07 component measurements" data={metrics}
          columns={['name', 'measured_iterations', 'p50_us', 'p95_us', 'p99_us', 'throughput_ops_per_second', 'median_ci95_us']} />
      </> : <Empty title="The benchmark record has no component measurements." />}
      <p className="panel-note">p99 is withheld because sample counts are below 1,000. CPU and memory counters cover the controller process; separate API and worker processes are excluded from those host totals.</p>
    </Panel>
    <Panel title="Build and protocol"><Fields data={{ rustc: isRow(performance.build) ? performance.build.rustc_verbose : null,
      binary_sha256: isRow(performance.build) ? performance.build.binary_sha256 : null,
      methodology: isRow(performance.configuration) ? performance.configuration.uncertainty : null,
      scope: performance.scope }} /></Panel>
  </>
}

export function DiscoveryLaboratory({ token, archive }: { token: string; archive: ResearchArchive }) {
  return <>
    <section className="mission-lead compact-lead"><div><p className="eyebrow">Discovery laboratory / Development only</p>
      <h2>Search, inspect, replay.</h2><p>Run matched seeded policies, inspect candidate parents and compare the saved hybrid study.</p></div>
      <div className="mission-current"><span>Worker authority</span><strong>Reviewed knapsack primitive</strong><small>No generated code execution</small></div>
    </section>
    <Panel title="DAE-04 saved hybrid comparison" aside={<span className="muted">historical synthetic study</span>}>
      <RecordTable label="Archived hybrid discovery comparison" data={records(archive.discovery.hybrid.arms)}
        columns={['arm', 'trials', 'optimal', 'mean_optimality_gap', 'candidate_evaluations', 'feasible_evaluations']} />
      <p className="panel-note">Results are exploratory and apply to the constructed development grid. The archive retains candidate lineage and failed proposals.</p>
    </Panel>
    <Evolution token={token} />
  </>
}

function artifactRows(archive: ResearchArchive) { return records(archive.artifacts) }

export function ResearchArchiveView({ archive, token }: { archive: ResearchArchive; token: string }) {
  const [busy, setBusy] = useState('')
  const [error, setError] = useState('')
  const reliability = isRow(archive.reliability_ablation) ? archive.reliability_ablation : {}
  const contrasts = records(reliability.contrasts).map((row) => ({
    arm: row.arm, control: row.control,
    ...(isRow(row.pooled_without_copies) ? row.pooled_without_copies : {}),
  }))
  const pilot = isRow(archive.ecar_pilot.primary_ecar_vs_adaptive_flat) ? archive.ecar_pilot.primary_ecar_vs_adaptive_flat : {}
  async function download(id: string) {
    setError(''); setBusy(id)
    try {
      const report = await getResearchArtifact(token, id)
      const link = document.createElement('a')
      link.href = URL.createObjectURL(new Blob([report], { type: 'application/json' }))
      link.download = `${id}.json`; link.click(); window.setTimeout(() => URL.revokeObjectURL(link.href), 500)
    } catch (reason) { setError(reason instanceof Error ? reason.message : 'Archive request failed.') }
    finally { setBusy('') }
  }
  return <>
    <section className="mission-lead compact-lead"><div><p className="eyebrow">Research archive / Immutable records</p>
      <h2>Keep the negative results visible.</h2><p>Historical development studies, pilot outcomes and raw reports. These records do not indicate current system status.</p></div>
      <div className="mission-current"><span>Archive index</span><strong>{text(archive.recorded_on)}</strong><small>Build-pinned records · read-only</small></div>
    </section>
    <Panel title="Search and optimization studies"><RecordTable label="Historical discovery results"
      data={[...records(archive.discovery.baseline.arms).map((row) => ({ study: 'DAE-02/03 baseline', ...row })),
        ...records(archive.discovery.hybrid.arms).map((row) => ({ study: 'DAE-04 hybrid', ...row }))]}
      columns={['study', 'arm', 'trials', 'optimal', 'mean_optimality_gap', 'candidate_evaluations', 'feasible_evaluations']} />
      <p className="panel-note">Matched synthetic development instances. These comparisons are exploratory, not evidence of general superiority.</p>
    </Panel>
    <div className="dashboard-columns">
      <Panel title="Reliability and information bonus · DAE-05B">
        <Fields data={{ outcomes_sha256: reliability.outcomes_sha256,
          prior_trace_changes: reliability.prior_study_trace_changes }} />
        <RecordTable label="Reliability and information bonus contrasts" data={contrasts}
          columns={['arm', 'control', 'delta', 'wins', 'ties', 'losses']} empty="No archived contrasts." />
      </Panel>
      <Panel title="ECAR pilot · historical result"><Fields data={{
        estimate: pilot.mean_points,
        interval_95: pilot.bootstrap_95,
        cases: archive.ecar_pilot.cases,
        interpretation: 'Pilot only; not a held-out confirmation.' }} />
        <p className="panel-note">Independent custody is not established. The held-out protocol was not run.</p>
      </Panel>
    </div>
    <Panel title="Versioned raw records" aside={<span className="mono muted">SHA-256 pinned</span>}>
      {errorText(error)}
      <div className="archive-list">{artifactRows(archive).map((item) => <article className="archive-row" key={String(item.id)}>
        <div><strong>{text(item.title)}</strong><code>{text(item.sha256)}</code><small>{fmt(item.size_bytes, 0)} bytes</small></div>
        <button type="button" className="button" disabled={busy !== ''} onClick={() => void download(String(item.id))}>
          {busy === item.id ? 'Loading…' : 'Download raw JSON'}
        </button>
      </article>)}</div>
    </Panel>
  </>
}

export function SecurityCentre() {
  const controls = [
    ['Exact protocol approval', 'The owner approves the frozen protocol digest before campaign execution.'],
    ['Reviewed task registry', 'The worker accepts typed, built-in computational tasks; arbitrary generated code is disabled.'],
    ['Independent verdict authority', 'Model proposals cannot issue verdicts or access hidden evaluator data.'],
    ['Held-out research', 'ECAR execution remains deferred until independent task custody and approval are established.'],
  ]
  return <>
    <section className="mission-lead compact-lead"><div><p className="eyebrow">Security centre / Current qualification</p>
      <h2>Boundaries are part of the result.</h2><p>Local worker controls and remaining isolation limits, documented from the qualified Windows profile.</p></div>
      <div className="mission-current"><span>Containment profile</span><strong>Reviewed tasks only</strong><small>Windows x64 · Job Object supervision</small></div>
    </section>
    <Panel title="Approval and authority"><div className="security-list">{controls.map(([title, detail]) => <article key={title}>
      <Badge value={title === 'Held-out research' ? 'deferred' : 'policy'} /><div><strong>{title}</strong><p>{detail}</p></div>
    </article>)}</div></Panel>
    <Panel title="Qualified worker limits"><div className="limit-grid">
      {[['CPU', '1 s user-mode quota'], ['Memory', '64 MiB aggregate commit'], ['Processes', '1 active process'], ['Lifecycle', 'Kill on job close']].map(([label, value]) =>
        <div key={label}><small>{label}</small><strong>{value}</strong></div>)}
    </div><p className="panel-note">The worker still runs as the desktop user. The profile does not deny filesystem access or network egress, and does not provide a restricted identity. Process supervision is not a complete OS sandbox.</p></Panel>
    <Panel title="Model and data separation"><div className="authority-strip">
      <span>Local model can advise</span><b aria-hidden="true">→</b><span>Controller validates typed output</span><b aria-hidden="true">→</b><span>Trusted evaluator rules</span>
    </div><p className="panel-note">The DAE-06 local inference record used an owned cloud-disabled Ollama process. This page does not query that service or claim it is currently running.</p></Panel>
  </>
}
