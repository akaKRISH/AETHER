import type { KeyboardEvent } from 'react'
import { Badge, Empty, errorText } from './components'
import {
  QUALITY_AREAS,
  describeExecutionStatus,
  describeIntegrityStatus,
  describeStructuralStatus,
  formatQualityError,
  type EvidenceIntegrityOutcome,
  type ExecutionOutcome,
  type IntegrationStatus,
  type QualityActionError,
  type QualityAreaId,
  type StructuralOutcome,
} from './qualityModels'

export function EpistemicBoundaryStrip({
  humanReviewLabel = 'Human review: Not started',
  independentValidationLabel = 'Independent scientific validation: Not performed',
}: {
  readonly humanReviewLabel?: string
  readonly independentValidationLabel?: string
}) {
  return (
    <section className="eval-epistemic-strip" aria-label="Epistemic boundaries">
      <article className="eval-boundary-cell">
        <span className="eval-boundary-kicker">Layer 1 · Automated structural checks</span>
        <strong>Schema, pin &amp; reference completeness only</strong>
        <p className="muted">
          Structural passes confirm format and recorded links; they never verify scientific truth, source quality, or entailment.
        </p>
      </article>
      <article className="eval-boundary-cell">
        <span className="eval-boundary-kicker">Layer 2 · Human review</span>
        <Badge value="not_started" label={humanReviewLabel} />
        <p className="muted">No human semantic adjudication write endpoint exists in Slice 1; the local token carries no reviewer identity.</p>
      </article>
      <article className="eval-boundary-cell">
        <span className="eval-boundary-kicker">Layer 3 · External custody</span>
        <Badge value="not_performed" label={independentValidationLabel} />
        <p className="muted">Independent scientific validation cannot be requested or overridden by the UI or evaluation inputs.</p>
      </article>
    </section>
  )
}

export function IntegrationStatusNotice({ integration }: { readonly integration: IntegrationStatus }) {
  if (integration.status === 'connected') return null
  return (
    <div className="eval-integration-notice" role="status">
      <div>
        <Badge value="unavailable" label="Schema integration pending" />
        <strong>Awaiting canonical backend schema before live API binding</strong>
      </div>
      <p>
        Expected schema path: <code>{integration.expectedSchemaPath}</code>. {integration.summary}
      </p>
    </div>
  )
}

export function QualityAreaTabs({
  activeArea,
  onSelect,
}: {
  readonly activeArea: QualityAreaId
  readonly onSelect: (area: QualityAreaId) => void
}) {
  function onKeyDown(event: KeyboardEvent<HTMLButtonElement>, index: number) {
    let nextIndex = -1
    if (event.key === 'ArrowRight') nextIndex = (index + 1) % QUALITY_AREAS.length
    else if (event.key === 'ArrowLeft') nextIndex = (index - 1 + QUALITY_AREAS.length) % QUALITY_AREAS.length
    else if (event.key === 'Home') nextIndex = 0
    else if (event.key === 'End') nextIndex = QUALITY_AREAS.length - 1
    if (nextIndex < 0) return
    event.preventDefault()
    const target = QUALITY_AREAS[nextIndex]
    onSelect(target.id)
    const tablist = event.currentTarget.closest('[role="tablist"]')
    tablist?.querySelector<HTMLButtonElement>(`#eval-tab-${target.id}`)?.focus()
  }

  return (
    <div className="eval-tabs" role="tablist" aria-label="Quality evaluation workspace areas">
      {QUALITY_AREAS.map((area, index) => {
        const active = area.id === activeArea
        return (
          <button
            key={area.id}
            id={`eval-tab-${area.id}`}
            type="button"
            role="tab"
            aria-selected={active}
            aria-controls={`eval-panel-${area.id}`}
            tabIndex={active ? 0 : -1}
            className={`eval-tab ${active ? 'eval-tab--active' : ''}`}
            onClick={() => onSelect(area.id)}
            onKeyDown={(event) => onKeyDown(event, index)}
          >
            <span>{area.label}</span>
          </button>
        )
      })}
    </div>
  )
}

export function StructuralOutcomeBadge({ status }: { readonly status: StructuralOutcome }) {
  return <Badge value={status} label={describeStructuralStatus(status)} />
}

export function ExecutionOutcomeBadge({ status }: { readonly status: ExecutionOutcome }) {
  return <Badge value={status} label={describeExecutionStatus(status)} />
}

export function IntegrityOutcomeBadge({ status }: { readonly status: EvidenceIntegrityOutcome }) {
  return <Badge value={status} label={describeIntegrityStatus(status)} />
}

export function LimitationsCallout({
  limitations,
  title = 'Explicit evaluation limitations',
}: {
  readonly limitations: readonly string[]
  readonly title?: string
}) {
  return (
    <section className="eval-limitations" aria-label={title}>
      <h3>{title}</h3>
      {limitations.length ? (
        <ul>
          {limitations.map((item) => (
            <li key={item}>{item}</li>
          ))}
        </ul>
      ) : (
        <p className="muted">
          No domain-specific limitations were attached, yet structural checks still establish completeness only and never establish scientific validity.
        </p>
      )}
    </section>
  )
}

export function QualityErrorBanner({
  error,
  onRetry,
}: {
  readonly error: QualityActionError | null | undefined
  readonly onRetry?: () => void
}) {
  const formatted = formatQualityError(error)
  if (!formatted) return null
  return (
    <div className="eval-error-box">
      {errorText(formatted)}
      {onRetry && (
        <button type="button" className="button button--quiet" onClick={onRetry}>
          Retry
        </button>
      )}
    </div>
  )
}

export function EmptyOrLoading({
  loading,
  loadingText,
  emptyTitle,
  emptyBody,
}: {
  readonly loading?: boolean
  readonly loadingText: string
  readonly emptyTitle: string
  readonly emptyBody: string
}) {
  if (loading) {
    return (
      <p className="loading" role="status">
        {loadingText}
      </p>
    )
  }
  return <Empty title={emptyTitle}>{emptyBody}</Empty>
}
