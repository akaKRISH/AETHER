import { useEffect, useState } from 'react'
import { getSkillManifest, getSkillRegistry, isRow, message, rows, type Row } from './api'
import { Empty, Fields, Panel, RecordTable, errorText, pretty, text } from './components'

export function ResearchSkills({ token }: { token: string }) {
  const [registry, setRegistry] = useState<Row>()
  const [selected, setSelected] = useState<Row>()
  const [error, setError] = useState('')
  const [detailError, setDetailError] = useState('')
  useEffect(() => {
    const controller = new AbortController()
    getSkillRegistry(token, controller.signal).then(
      (value) => { if (!controller.signal.aborted) setRegistry(value) },
      (reason) => { if (!controller.signal.aborted) setError(message(reason)) },
    )
    return () => controller.abort()
  }, [token])

  async function inspect(hash: string) {
    setSelected(undefined)
    setDetailError('')
    const controller = new AbortController()
    try { setSelected(await getSkillManifest(token, hash, controller.signal)) }
    catch (reason) { setDetailError(message(reason)) }
  }

  const skills = rows(registry?.skills)
  const skill = isRow(selected?.skill) ? selected.skill : undefined
  const definition = isRow(skill?.definition) ? skill.definition : undefined
  return <Panel title="Research skill registry">
    <p>Declarative manifests only. Registry instructions are displayed as untrusted text and are never executed.</p>
    {error ? errorText(error) : !registry ? <p role="status">Loading research skills…</p> : <>
      <RecordTable label="Registered research skills" data={skills} columns={[
        'id', 'version', 'trust', 'approval', 'required_capabilities', {
          key: 'sha256', label: 'Version digest', render: (value) => <button type="button"
            className="button button--quiet" onClick={() => inspect(text(value))}>Inspect skill</button>,
        },
      ]} empty="No research skills are registered." />
      {registry.truncated === true && <p role="status">Showing a bounded registry page.</p>}
    </>}
    {detailError ? errorText(detailError) : null}
    {skill && <details open>
      <summary>{text(skill.id)} · {text(skill.version)} · {pretty(text(skill.trust))}</summary>
      <Fields data={{ sha256: skill.sha256, artifact_sha256: skill.artifact_sha256,
        trust: skill.trust, approval: skill.approval, execution: 'never_executed' }} />
      <h4>Permission boundary</h4>
      <Fields data={{ required_capabilities: definition?.required_capabilities,
        optional_capabilities: definition?.optional_capabilities,
        dependencies: definition?.dependencies }} />
      <h4>Model and resource budget</h4>
      <Fields data={{ model: definition?.model,
        maximum_inference_budget: definition?.maximum_inference_budget,
        maximum_resources: definition?.maximum_resources }} />
      <h4>Input and output schemas</h4>
      <Fields data={{ input_schema: definition?.input_schema, output_schema: definition?.output_schema }} />
      <h4>Evidence requirements</h4>
      <Fields data={isRow(definition?.evidence) ? definition.evidence : {}} />
      <h4>Untrusted declarative instructions</h4>
      <pre>{text(definition?.instructions)}</pre>
      <Empty title="Inspection only. No model call, tool execution, approval, or verification occurred." />
    </details>}
  </Panel>
}
