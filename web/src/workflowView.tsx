import { useEffect, useState } from 'react'
import { getWorkflowBenchmark, getWorkflowBenchmarks, getWorkflowExecutions, getWorkflowManifest,
  getWorkflowRegistry, isRow, message, rows, runWorkflowBenchmark, type Row } from './api'
import { Empty, Fields, Panel, RecordTable, errorText, pretty, text } from './components'

export function ResearchWorkflows({ token }: { token: string }) {
  const [registry, setRegistry] = useState<Row>()
  const [executions, setExecutions] = useState<Row>()
  const [benchmark, setBenchmark] = useState<Row>()
  const [benchmarkReport, setBenchmarkReport] = useState<Row>()
  const [benchmarkError, setBenchmarkError] = useState('')
  const [benchmarkBusy, setBenchmarkBusy] = useState(false)
  const [selected, setSelected] = useState<Row>()
  const [error, setError] = useState('')
  const [detailError, setDetailError] = useState('')
  useEffect(() => {
    const controller = new AbortController()
    Promise.all([getWorkflowRegistry(token, controller.signal), getWorkflowExecutions(token, controller.signal),
      getWorkflowBenchmarks(token, controller.signal)]).then(
      ([workflows, receipts, evaluation]) => {
        if (!controller.signal.aborted) { setRegistry(workflows); setExecutions(receipts); setBenchmark(evaluation) }
      }, (reason) => { if (!controller.signal.aborted) setError(message(reason)) },
    )
    return () => controller.abort()
  }, [token])

  async function inspect(hash: string) {
    setSelected(undefined)
    setDetailError('')
    const controller = new AbortController()
    try { setSelected(await getWorkflowManifest(token, hash, controller.signal)) }
    catch (reason) { setDetailError(message(reason)) }
  }

  async function inspectBenchmark(hash: string) {
    setBenchmarkReport(undefined)
    setBenchmarkError('')
    try { setBenchmarkReport(await getWorkflowBenchmark(token, hash)) }
    catch (reason) { setBenchmarkError(message(reason)) }
  }

  async function runBenchmark() {
    setBenchmarkBusy(true)
    setBenchmarkError('')
    try {
      await runWorkflowBenchmark(token)
      setBenchmark(await getWorkflowBenchmarks(token, new AbortController().signal))
    } catch (reason) { setBenchmarkError(message(reason)) }
    finally { setBenchmarkBusy(false) }
  }

  const workflows = rows(registry?.workflows)
  const runs = rows(executions?.executions).map((entry) => {
    const receipt = isRow(entry.receipt) ? entry.receipt : {}
    return { workflow_execution_sha256: entry.workflow_execution_sha256, workflow_sha256: receipt.workflow_sha256,
      outcome: receipt.outcome, execution_order: receipt.execution_order, consumed_budget: receipt.consumed_budget,
      evidence_status: receipt.evidence_status }
  })
  const workflow = isRow(selected?.workflow) ? selected.workflow : undefined
  const definition = isRow(workflow?.definition) ? workflow.definition : undefined
  const benchmarkSpec = isRow(benchmark?.specification) && isRow(benchmark.specification.specification)
    ? benchmark.specification.specification : undefined
  const benchmarkReports = rows(benchmark?.reports)
  const selectedReport = isRow(benchmarkReport?.report) ? benchmarkReport.report : undefined
  return <Panel title="Research workflow registry">
    <p>Immutable declarative DAGs run sequentially through APO-09. Local approval authorizes execution only; evidence remains unverified until an independent evidence process verifies it.</p>
    {error ? errorText(error) : !registry || !executions ? <p role="status">Loading research workflows and receipts…</p> : <>
      <RecordTable label="Registered research workflows" data={workflows} columns={[
        'id', 'version', 'approval', 'maximum_budget', {
          key: 'sha256', label: 'Workflow digest', render: (value) => <button type="button"
            className="button button--quiet" onClick={() => inspect(text(value))}>Inspect workflow</button>,
        },
      ]} empty="No research workflow fixtures are registered." />
      {registry.truncated === true && <p role="status">Showing a bounded registry page.</p>}
      <RecordTable label="Workflow execution receipts" data={runs}
        columns={['workflow_execution_sha256', 'workflow_sha256', 'outcome', 'execution_order', 'consumed_budget', 'evidence_status']}
        empty="No workflow executions have been recorded." />
    </>}
    {detailError ? errorText(detailError) : null}
    {workflow && <details open>
      <summary>{text(definition?.id)} · {text(definition?.version)} · {pretty(text(selected?.approval))}</summary>
      <Fields data={{ sha256: workflow.sha256, approval: selected?.approval,
        execution: selected?.execution, maximum_budget: definition?.maximum_budget }} />
      <h4>Typed workflow definition</h4>
      <Fields data={{ input_schema: definition?.input_schema, output_schema: definition?.output_schema,
        nodes: definition?.nodes, outputs: definition?.outputs }} />
      <Empty title="Inspection only. No workflow was approved, executed, or scientifically verified." />
    </details>}
    <section aria-labelledby="apo10-heading" className="workflow-benchmark">
      <header className="panel-heading"><h3 id="apo10-heading">APO-10 workflow contract evaluation</h3>
        <button className="button button--quiet" type="button" onClick={runBenchmark} disabled={benchmarkBusy}>
          {benchmarkBusy ? 'Running fixed fixtures…' : 'Run APO-10 fixture evaluation'}
        </button>
      </header>
      <p>Nine controlled local cases run through APO-09 in disposable fixture databases. Two fresh processes are compared. The report tests system behavior; it does not score scientific discovery.</p>
      {benchmarkError ? errorText(benchmarkError) : !benchmark ? <p role="status">Loading benchmark specification and reports…</p> : <>
        {benchmarkSpec && <Fields data={{ id: benchmarkSpec.id, version: benchmarkSpec.version,
          specification_sha256: benchmark.specification_sha256, scope: benchmarkSpec.scope,
          fixture_count: rows(benchmarkSpec.fixtures).length,
          adversarial_fixture_count: rows(benchmarkSpec.adversarial_fixtures).length }} />}
        <RecordTable label="APO-10 content-addressed reports" data={benchmarkReports.map((entry) => {
          const report = isRow(entry.report) ? entry.report : {}
          return { report_sha256: entry.report_sha256, passed: report.passed,
            workflow_fixtures: report.fixture_count, adversarial_fixtures: report.adversarial_fixture_count,
            process_replay: isRow(report.fresh_process_replay) ? report.fresh_process_replay.worker_reports_match : null }
        })} columns={['passed', 'workflow_fixtures', 'adversarial_fixtures', 'process_replay', {
          key: 'report_sha256', label: 'Report', render: (value) => <button type="button"
            className="button button--quiet" onClick={() => inspectBenchmark(text(value))}>Inspect report</button>,
        }]} empty="No APO-10 evaluation report has been recorded." />
      </>}
      {benchmarkReport && <details open>
        <summary>Report {text(benchmarkReport.report_sha256)} · {text(selectedReport?.passed ? 'passed' : 'failed')}</summary>
        <Fields data={{ benchmark: selectedReport?.benchmark, configuration: selectedReport?.configuration,
          fresh_process_replay: selectedReport?.fresh_process_replay, system_metrics: selectedReport?.system_metrics,
          scientific_quality_metrics: selectedReport?.scientific_quality_metrics,
          limitations: selectedReport?.limitations, failures: selectedReport?.failures }} />
        <RecordTable label="APO-10 fixture results" data={rows(selectedReport?.fixtures)}
          columns={['fixture_id', 'outcome', 'execution_order', 'budget_compliant', 'kernel_evidence_unverified', 'passed']} />
        <RecordTable label="APO-10 adversarial fixtures" data={rows(selectedReport?.adversarial_fixtures)}
          columns={['id', 'passed', 'rejected']} />
      </details>}
      {benchmarkError ? errorText(benchmarkError) : null}
    </section>
  </Panel>
}
