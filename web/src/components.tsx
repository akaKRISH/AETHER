import { useEffect, useRef } from 'react'
import type { ReactNode } from 'react'
import { isRow, type Row } from './api'

export function pretty(value: string) {
  return value.replaceAll('_', ' ').replace(/^./, (start) => start.toUpperCase())
}

export function text(value: unknown): string {
  if (value === null || value === undefined) return '—'
  if (typeof value === 'boolean') return value ? 'Yes' : 'No'
  if (typeof value === 'object') return JSON.stringify(value)
  return String(value)
}

export function readNumber(value: FormDataEntryValue | null) {
  const input = String(value ?? '').trim()
  return /^\d+$/.test(input) ? Number(input) : NaN
}

export function integer(value: number, min: number, max = Number.MAX_SAFE_INTEGER) {
  return Number.isSafeInteger(value) && value >= min && value <= max
}

const good = new Set(['approved', 'verified', 'succeeded', 'completed', 'connected'])
const structural = new Set(['pass', 'structural_pass'])
const caution = new Set(['running', 'supported', 'paused', 'inconclusive', 'stale', 'partial', 'abstained'])
const fault = new Set([
  'failed', 'rejected', 'revoked', 'cancelled', 'expired', 'disconnected', 'fail', 'structural_fail', 'invalid',
])

export function Badge({ value, label }: { value: string; label?: string }) {
  const state = value.toLowerCase()
  const tone = good.has(state)
    ? 'good'
    : structural.has(state)
      ? 'structural'
      : caution.has(state)
        ? 'warn'
        : fault.has(state)
          ? 'fault'
          : 'neutral'
  return <span className={`badge badge--${tone}`}><span aria-hidden="true" />{label ?? pretty(value)}</span>
}

export function Empty({ title, children }: { title: string; children?: ReactNode }) {
  return <div className="empty"><p>{title}</p>{children && <div className="muted">{children}</div>}</div>
}

export function Panel({ title, children, aside, className = '' }: {
  title: string; children: ReactNode; aside?: ReactNode; className?: string
}) {
  return <section className={`panel ${className}`}>
    <header className="panel-heading"><h2>{title}</h2>{aside}</header>
    {children}
  </section>
}

type Column = string | { key: string; label: string; render?: (value: unknown, row: Row) => ReactNode }

export function keyedRows(data: Row[]) {
  const seen = new Map<string, number>()
  return data.map((row) => {
    const identity = JSON.stringify(row.id ?? row.sequence ?? row.url ?? row)
    const occurrence = (seen.get(identity) ?? 0) + 1
    seen.set(identity, occurrence)
    return { key: `${identity}|${occurrence}`, row }
  })
}

export function RecordTable({ label, data, columns, empty = 'No records have been written.' }: {
  label: string; data: Row[]; columns: Column[]; empty?: string
}) {
  if (!data.length) return <Empty title={empty} />
  const normalized = columns.map((column) => typeof column === 'string'
    ? { key: column, label: pretty(column), render: undefined } : column)
  // biome-ignore lint/a11y/noNoninteractiveTabindex: Wide tables need a keyboard-focusable scroll container.
  return <section className="table-scroll" tabIndex={0} aria-label={label}>
    <table>
      <caption className="sr-only">{label}</caption>
      <thead><tr>{normalized.map(({ key, label }) => <th key={key} scope="col">{label}</th>)}</tr></thead>
      <tbody>{keyedRows(data).map(({ row, key }) =>
        <tr key={key}>
          {normalized.map(({ key, render }) =>
            <td key={key} className={key.includes('hash') || key === 'sha256' || key === 'id' ? 'mono data-id' : undefined}>
              {render ? render(row[key], row) : text(row[key])}
            </td>)}
        </tr>)}</tbody>
    </table>
  </section>
}

export function Fields({ data, omit = [] }: { data: Row; omit?: string[] }) {
  return <dl className="fields">
    {Object.entries(data).filter(([key]) => !omit.includes(key)).map(([key, value]) =>
      <div key={key}><dt>{pretty(key)}</dt><dd className={key.includes('hash') || key.includes('sha256') ? 'mono' : undefined}>
        {typeof value === 'object' && value !== null
          ? <pre>{JSON.stringify(value, null, 2)}</pre> : text(value)}
      </dd></div>)}
  </dl>
}

export function Modal({ title, children, onClose }: { title: string; children: ReactNode; onClose: () => void }) {
  const dialog = useRef<HTMLDialogElement>(null)
  useEffect(() => {
    const element = dialog.current
    if (!element) return
    element.showModal()
    element.querySelector<HTMLElement>('[data-initial-focus]')?.focus()
    return () => { if (element.open) element.close() }
  }, [])

  return <dialog ref={dialog} aria-label={title} onCancel={(event) => { event.preventDefault(); onClose() }}>
    <header className="modal-heading">
      <h2>{title}</h2>
      <button className="button button--quiet" onClick={onClose} type="button" aria-label="Close dialog">×</button>
    </header>
    {children}
  </dialog>
}

export function errorText(value: string) {
  return value ? <p className="error" role="alert">{value}</p> : null
}

export function externalLink(value: unknown): string | null {
  if (typeof value !== 'string') return null
  try {
    const url = new URL(value)
    return url.protocol === 'https:' || url.protocol === 'http:' ? url.href : null
  } catch {
    return null
  }
}

export function readableRecord(value: unknown): Row {
  return isRow(value) ? value : { value }
}
