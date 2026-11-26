import { useEffect, useState } from 'react'
import { getWorkflowExecutions, getWorkflowManifest, getWorkflowRegistry, isRow, message, rows, type Row } from './api'
import { Empty, Fields, Panel, RecordTable, errorText, pretty, text } from './components'

export function ResearchWorkflows({ token }: { token: string }) {
  const [registry, setRegistry] = useState<Row>()
  const [executions, setExecutions] = useState<Row>()
  const [selected, setSelected] = useState<Row>()
  const [error, setError] = useState('')
  const [detailError, setDetailError] = useState('')
  useEffect(() => {
    const controller = new AbortController()
    Promise.all([getWorkflowRegistry(token, controller.signal), getWorkflowExecutions(token, controller.signal)]).then(
      ([workflows, receipts]) => {
        if (!controller.signal.aborted) { setRegistry(workflows); setExecutions(receipts) }
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

  const workflows = rows(registry?.workflows)
  const runs = rows(executions?.executions).map((entry) => {
    const receipt = isRow(entry.receipt) ? entry.receipt : {}
    return { workflow_execution_sha256: entry.workflow_execution_sha256, workflow_sha256: receipt.workflow_sha256,
      outcome: receipt.outcome, execution_order: receipt.execution_order, consumed_budget: receipt.consumed_budget,
      evidence_status: receipt.evidence_status }
  })
  const workflow = isRow(selected?.workflow) ? selected.workflow : undefined
  const definition = isRow(workflow?.definition) ? workflow.definition : undefined
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
  </Panel>
}
