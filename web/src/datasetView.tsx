import { useEffect, useState } from 'react'
import { getDatasetManifest, getDatasetPart, getDatasetRegistry, message, rows, type Row } from './api'
import { Fields, Panel, RecordTable, errorText, pretty, readableRecord, text } from './components'

export function DatasetLaboratory({ token }: { token: string }) {
  const [registry, setRegistry] = useState<Row>()
  const [selected, setSelected] = useState<Row>()
  const [error, setError] = useState('')
  const [busy, setBusy] = useState(false)
  useEffect(() => {
    const controller = new AbortController()
    getDatasetRegistry(token, controller.signal).then(
      (value) => { if (!controller.signal.aborted) setRegistry(value) },
      (reason) => { if (!controller.signal.aborted) setError(message(reason)) },
    )
    return () => controller.abort()
  }, [token])

  async function select(pin?: string) {
    setBusy(true)
    setError('')
    try {
      const value = await getDatasetManifest(token, pin)
      setSelected(value)
      if (!pin) setRegistry((current) => ({
        ...current, datasets: [value, ...rows(current?.datasets).filter((item) => item.manifest_sha256 !== value.manifest_sha256)],
      }))
    } catch (reason) { setError(message(reason)) }
    finally { setBusy(false) }
  }

  async function download(part: string, pin: string) {
    setBusy(true)
    setError('')
    try {
      const bytes = await getDatasetPart(token, pin)
      const url = URL.createObjectURL(new Blob([bytes]))
      const link = document.createElement('a')
      link.href = url
      link.download = `dataset-${part}-${pin.slice(0, 12)}.${part === 'rejections' ? 'json' : 'jsonl'}`
      link.click()
      window.setTimeout(() => URL.revokeObjectURL(url), 500)
    } catch (reason) { setError(message(reason)) }
    finally { setBusy(false) }
  }

  const manifest = readableRecord(selected?.manifest)
  const datasets = rows(registry?.datasets).map((entry) => ({
    ...readableRecord(entry.manifest), manifest_sha256: entry.manifest_sha256,
  }))
  const parts = Object.entries(readableRecord(manifest.parts)).map(([part, value]) => ({
    part, ...readableRecord(value),
  }))
  return <Panel title="Scientific dataset laboratory">
    <div className="toolbar"><button className="button" type="button" disabled={busy} onClick={() => select()}>
      Qualify development corpus
    </button>{busy && <span role="status">Working…</span>}</div>
    {error && errorText(error)}
    {!registry && !error ? <p role="status">Loading datasets…</p> : <RecordTable label="Saved datasets" data={datasets}
      columns={['name', 'qualification', 'retained_after_filtering', 'rejected', {
        key: 'manifest_sha256', label: 'Manifest', render: (pin) =>
          <button type="button" className="button button--quiet" disabled={busy} onClick={() => select(text(pin))}>Inspect dataset</button>,
      }]} empty="No dataset versions are saved." />}
    {selected && <>
      <Fields data={{ manifest_sha256: selected.manifest_sha256, scope: manifest.scope, independent_test: manifest.independent_test }} />
      <RecordTable label="Dataset coverage" data={Object.entries(readableRecord(manifest.coverage)).map(([split, value]) => ({
        split, ...readableRecord(value),
      }))} columns={['split', 'rows', 'groups', 'topics', 'origins', 'verification']} />
      <RecordTable label="Dataset evidence sources" data={rows(manifest.source_documents)}
        columns={['title', 'license', 'document', 'node', 'revision_sha256']} empty="This dataset has no extracted source rows." />
      <RecordTable label="Dataset artifact exports" data={parts} columns={['part', 'records', 'bytes', {
        key: 'sha256', label: 'Export', render: (pin, row) =>
          <button type="button" className="button button--quiet" disabled={busy} onClick={() => download(text(row.part), text(pin))}>
            Export {pretty(text(row.part))}
          </button>,
      }]} />
    </>}
  </Panel>
}
