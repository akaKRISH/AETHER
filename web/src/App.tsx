import { useEffect, useState } from 'react'
import type { FormEvent, ReactNode } from 'react'
import { campaignPath, getSnapshot, listCampaigns, message, post, type Campaign, type NewCampaign, type Snapshot } from './api'
import { Budgets, Decisions, Events, EvidenceGraph, Experiments, Overview, Verification } from './campaignViews'
import { Benchmarks, Catalog, Evolution, GpuTelemetry, Hypotheses, Report } from './consoleViews'
import { Badge, Empty, Modal, Panel, errorText, integer, readNumber } from './components'

const campaignViews = [
  ['overview', 'Overview'],
  ['hypotheses', 'Hypotheses'],
  ['experiments', 'Experiments'],
  ['evidence', 'Evidence graph'],
  ['budget', 'Resource budget'],
  ['events', 'Event journal'],
  ['decisions', 'ECAR decisions'],
  ['verification', 'Verification'],
  ['report', 'Report'],
] as const
const labViews = [
  ['benchmarks', 'Benchmarks'],
  ['evolution', 'Evolution explorer'],
  ['runtime', 'Runtime'],
  ['literature', 'Literature'],
  ['settings', 'Security settings'],
] as const

type View = (typeof campaignViews)[number][0] | (typeof labViews)[number][0]
type DialogName = 'connect' | 'create' | 'approve' | 'cancel'

function ConnectionForm({ connect, close }: { connect: (token: string) => Promise<void>; close: () => void }) {
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const token = String(new FormData(event.currentTarget).get('token') ?? '').trim()
    if (!token) {
      setError('Enter the API Bearer token.')
      return
    }
    setBusy(true)
    setError('')
    try {
      await connect(token)
      close()
    } catch (reason) {
      setError(message(reason))
      setBusy(false)
    }
  }
  return <Modal title="Connect to AETHER" onClose={close}>
    <form className="modal-form" onSubmit={submit}>
      <p className="muted">Use the value configured as <code>AETHER_TOKEN</code> on your local API. The console keeps it in this tab until you disconnect or reload.</p>
      <label>API Bearer token<input name="token" type="password" required autoComplete="off" data-initial-focus /></label>
      {errorText(error)}
      <button type="submit" className="button button--primary" disabled={busy}>{busy ? 'Connecting…' : 'Connect'}</button>
    </form>
  </Modal>
}

function CampaignForm({ create, close }: { create: (value: NewCampaign) => Promise<void>; close: () => void }) {
  const [busy, setBusy] = useState(false)
  const [error, setError] = useState('')
  const [optimization, setOptimization] = useState(false)
  async function submit(event: FormEvent<HTMLFormElement>) {
    event.preventDefault()
    const data = new FormData(event.currentTarget)
    const task = optimization
      ? { optimization: { challenge_seed: readNumber(data.get('challenge_seed')) } }
      : { algorithm: String(data.get('algorithm')) }
    const campaign: NewCampaign = {
      id: String(data.get('id') ?? '').trim(), title: String(data.get('title') ?? '').trim(),
      objective: String(data.get('objective') ?? '').trim(),
      seed: readNumber(data.get('seed')), items: readNumber(data.get('items')),
      attempts: readNumber(data.get('attempts')), wall_ms: readNumber(data.get('wall_ms')),
      ...task,
    }
    const bytes = new TextEncoder()
    if (!/^[a-z0-9][a-z0-9-]{0,31}$/.test(campaign.id) || !campaign.title || !campaign.objective
      || bytes.encode(campaign.title).length > 120 || bytes.encode(campaign.objective).length > 512
      || !integer(campaign.seed, 0, 4_294_967_295) || !integer(campaign.items, 1, 128)
      || !integer(campaign.attempts, 1, 12) || !integer(campaign.wall_ms, 2_000, 30_000)) {
      setError('Use a lowercase campaign ID, a short title and objective, and values within the shown limits.')
      return
    }
    if (campaign.optimization && (!integer(campaign.optimization.challenge_seed, 0, 4_294_967_295)
      || campaign.optimization.challenge_seed === campaign.seed)) {
      setError('Choose a valid challenge seed different from the original seed.')
      return
    }
    setBusy(true)
    setError('')
    try {
      await create(campaign)
      close()
    } catch (reason) {
      setError(message(reason))
      setBusy(false)
    }
  }
  return <Modal title="New campaign" onClose={close}>
    <form className="modal-form" onSubmit={submit}>
      <div className="form-columns">
        <label>Campaign ID<input name="id" required maxLength={32} pattern="[a-z0-9][a-z0-9\-]*" placeholder="sorting-study" data-initial-focus /></label>
        <label>Title<input name="title" maxLength={120} required placeholder="Sorting study" /></label>
      </div>
      <label>Research objective<textarea name="objective" rows={3} maxLength={512} required /></label>
      <div className="form-columns">
        <label>Task domain<select value={optimization ? 'optimization' : 'sort'}
          onChange={(event) => setOptimization(event.target.value === 'optimization')}>
          <option value="sort">Seeded sorting</option>
          <option value="optimization">Maximum subarray</option>
        </select></label>
        {optimization
          ? <label>Challenge seed<input name="challenge_seed" type="number" min={0} max={4_294_967_295} step={1} defaultValue={281} required /></label>
          : <label>Algorithm<select name="algorithm" defaultValue="insertion">
            <option value="insertion">Insertion sort</option><option value="selection">Selection sort</option><option value="standard">Standard sort</option>
          </select></label>}
        <label>Random seed<input name="seed" type="number" min={0} max={4_294_967_295} step={1} defaultValue={11} required /></label>
        <label>Fixture items<input name="items" type="number" min={1} max={128} step={1} defaultValue={128} required /></label>
        <label>Attempt cap<input name="attempts" type="number" min={1} max={12} step={1} defaultValue={5} required /></label>
        <label>Time cap (ms)<input name="wall_ms" type="number" min={2_000} max={30_000} step={1} defaultValue={12_000} required /></label>
      </div>
      {errorText(error)}
      <button type="submit" className="button button--primary" disabled={busy}>{busy ? 'Creating…' : 'Create draft'}</button>
    </form>
  </Modal>
}

function Sidebar({ campaigns, selected, select, view, navigate, connected }: {
  campaigns: Campaign[]; selected: string; select: (id: string) => void; view: View;
  navigate: (view: View) => void; connected: boolean
}) {
  function navigation(entries: typeof campaignViews | typeof labViews, label: string) {
    return <nav className="rail-nav" aria-label={label}>
      <p>{label}</p>
      {entries.map(([id, title]) => <button type="button" key={id} aria-current={view === id ? 'page' : undefined}
        onClick={() => navigate(id)}>{title}</button>)}
    </nav>
  }

  return <aside className="rail">
    <div className="campaign-picker">
      <label htmlFor="campaign-picker">Campaign</label>
      <select id="campaign-picker" value={selected} disabled={!campaigns.length} onChange={(event) => select(event.target.value)}>
        {!campaigns.length && <option value="">{connected ? 'No campaigns' : 'Connect to load'}</option>}
        {campaigns.map((campaign) => <option key={campaign.id} value={campaign.id}>{campaign.title}</option>)}
      </select>
    </div>
    {navigation(campaignViews, 'Campaign workspace')}
    {navigation(labViews, 'Laboratory')}
  </aside>
}

function CampaignPane({ token, view, snapshot }: { token: string; view: View; snapshot: Snapshot }) {
  switch (view) {
    case 'hypotheses': return <Hypotheses key={snapshot.campaign.id} token={token} snapshot={snapshot} />
    case 'experiments': return <Experiments snapshot={snapshot} />
    case 'evidence': return <EvidenceGraph key={snapshot.campaign.id} snapshot={snapshot} />
    case 'budget': return <Budgets snapshot={snapshot} />
    case 'events': return <Events snapshot={snapshot} />
    case 'decisions': return <Decisions snapshot={snapshot} />
    case 'verification': return <Verification snapshot={snapshot} />
    case 'report': return <Report key={snapshot.campaign.id} token={token} campaignId={snapshot.campaign.id} />
    default: return <Overview snapshot={snapshot} />
  }
}

export function App() {
  const [token, setToken] = useState('')
  const [campaigns, setCampaigns] = useState<Campaign[]>([])
  const [selected, setSelected] = useState('')
  const [snapshot, setSnapshot] = useState<Snapshot>()
  const [view, setView] = useState<View>('overview')
  const [dialog, setDialog] = useState<DialogName>()
  const [snapshotError, setSnapshotError] = useState('')
  const [actionError, setActionError] = useState('')
  const [notice, setNotice] = useState('')
  const [busy, setBusy] = useState(false)
  const [refresh, setRefresh] = useState(0)

  // biome-ignore lint/correctness/useExhaustiveDependencies: A mutation increments refresh to restart the poll immediately.
  useEffect(() => {
    if (!token || !selected) return
    const controller = new AbortController()
    let timer = 0
    async function poll() {
      try {
        const current = await getSnapshot(token, selected, controller.signal)
        if (controller.signal.aborted) return
        setSnapshot(current)
        setSnapshotError('')
        setCampaigns((previous) => previous.some((campaign) => campaign.id === current.campaign.id)
          ? previous.map((campaign) => campaign.id === current.campaign.id ? current.campaign : campaign)
          : [...previous, current.campaign])
      } catch (error) {
        if (!controller.signal.aborted) setSnapshotError(message(error))
      } finally {
        if (!controller.signal.aborted) timer = window.setTimeout(() => void poll(), 2500)
      }
    }
    void poll()
    return () => { controller.abort(); window.clearTimeout(timer) }
  }, [token, selected, refresh])

  function select(id: string) {
    setSelected(id)
    setView('overview')
    setSnapshot(undefined)
    setSnapshotError('')
    setActionError('')
    setNotice('')
  }

  async function connect(secret: string) {
    const list = await listCampaigns(secret)
    setCampaigns(list)
    select(list[0]?.id ?? '')
    setToken(secret)
  }

  function disconnect() {
    setToken('')
    setCampaigns([])
    select('')
  }

  async function create(value: NewCampaign) {
    await post(token, '/campaigns', value)
    select(value.id)
    setNotice('Campaign draft created.')
    setRefresh((count) => count + 1)
  }

  const current = snapshot?.campaign?.id === selected ? snapshot : undefined
  const campaign = current?.campaign ?? campaigns.find((item) => item.id === selected)
  const state = campaign?.state.toLowerCase()
  const isLab = labViews.some(([id]) => id === view)
  const title = isLab ? labViews.find(([id]) => id === view)?.[1] : campaign?.title ?? 'Research workspace'

  async function mutate(action: 'approve' | 'start' | 'cancel') {
    if (!current || busy || snapshotError) return
    setBusy(true)
    setActionError('')
    setNotice('')
    try {
      await post(token, `${campaignPath(current.campaign.id)}/${action}`,
        action === 'approve' ? { protocol_hash: current.campaign.protocol_hash } : undefined)
      setDialog(undefined)
      setNotice(action === 'start' ? 'Campaign start accepted.'
        : action === 'cancel' ? 'Cancellation accepted.' : 'Protocol approval recorded.')
      setRefresh((count) => count + 1)
    } catch (error) {
      setActionError(message(error))
    } finally {
      setBusy(false)
    }
  }

  let content: ReactNode
  if (!token) {
    content = <section className="welcome">
      <div className="welcome-copy">
        <h2>Connect your research console</h2>
        <p>Load campaigns, inspect evidence, and track execution from your local AETHER API.</p>
        <button type="button" className="button button--primary" onClick={() => setDialog('connect')}>Connect API</button>
      </div>
    </section>
  } else if (view === 'benchmarks') {
    content = <Benchmarks token={token} />
  } else if (view === 'evolution') {
    content = <Evolution token={token} />
  } else if (view === 'runtime' || view === 'literature') {
    content = <>
      <Catalog key={view} token={token} kind={view} />
      {view === 'runtime' && <GpuTelemetry token={token} />}
    </>
  } else if (view === 'settings') {
    content = <>
      <Panel title="Local session">
        <p>Campaign controls use the local bearer token held in this tab. Disconnect to clear it from the console.</p>
        <button type="button" className="button" onClick={disconnect}>Clear local session</button>
      </Panel>
      <Catalog token={token} kind="runtime" />
    </>
  } else if (!selected) {
    content = <section className="panel">
      <Empty title="No campaigns have been created.">Create a draft and review its protocol before running an experiment.</Empty>
    </section>
  } else if (!current) {
    content = <section className="panel">
      {snapshotError ? errorText(snapshotError) : <p className="loading" role="status">Loading campaign…</p>}
    </section>
  } else {
    content = <CampaignPane token={token} view={view} snapshot={current} />
  }

  return <div className="console">
    <a className="skip-link" href="#workspace">Skip to workspace</a>
    <header className="topbar">
      <div className="brand"><span className="brand-mark" aria-hidden="true" /><div><strong>AETHER</strong><span>Research console</span></div></div>
      <div className="connection">
        <Badge value={token ? snapshotError ? 'stale' : 'connected' : 'disconnected'} />
        {token && <button className="button button--quiet" type="button" onClick={disconnect}>Disconnect</button>}
      </div>
    </header>
    <div className="workspace">
      <Sidebar campaigns={campaigns} selected={selected} select={select} connected={Boolean(token)}
        view={view} navigate={setView} />
      <main id="workspace" className="stage" tabIndex={-1}>
        <div className="stage-header">
          <div>
            <h1>{title}</h1>
            {!isLab && campaign && <div className="campaign-meta"><code>{campaign.id}</code><Badge value={campaign.state} /></div>}
            {!isLab && campaign?.objective && <p className="objective">{campaign.objective}</p>}
          </div>
          {token && !isLab && <div className="actions">
            {(state === 'draft' || state === 'approved' || state === 'running' || state === 'paused') && current && <>
              {state === 'draft' && <button className="button button--primary" type="button" disabled={busy || Boolean(snapshotError)}
                onClick={() => { setActionError(''); setDialog('approve') }}>Approve protocol</button>}
              {state === 'approved' && <button className="button button--primary" type="button"
                disabled={busy || Boolean(snapshotError)} onClick={() => void mutate('start')}>Start campaign</button>}
              <button className="button button--danger" type="button"
                disabled={busy || Boolean(snapshotError)} onClick={() => { setActionError(''); setDialog('cancel') }}>Cancel campaign</button>
            </>}
            <button type="button" className="button" onClick={() => setDialog('create')}>New campaign</button>
          </div>}
        </div>
        {notice && <p className="notice" role="status">{notice}</p>}
        {actionError && dialog !== 'approve' && dialog !== 'cancel' && errorText(actionError)}
        {snapshotError && current && <p className="error" role="alert">Showing the last loaded snapshot. {snapshotError}</p>}
        {content}
      </main>
    </div>
    {dialog === 'connect' && <ConnectionForm connect={connect} close={() => setDialog(undefined)} />}
    {dialog === 'create' && <CampaignForm create={create} close={() => setDialog(undefined)} />}
    {dialog === 'approve' && current && <Modal title="Approve frozen protocol" onClose={() => { if (!busy) setDialog(undefined) }}>
      <div className="modal-form">
        <p>Confirm the exact protocol hash for {current.campaign.title}.</p>
        <code className="hash-block">{current.campaign.protocol_hash}</code>
        {errorText(actionError)}
        <button type="button" className="button button--primary" disabled={busy || Boolean(snapshotError)}
          onClick={() => void mutate('approve')}>{busy ? 'Approving…' : 'Approve this hash'}</button>
      </div>
    </Modal>}
    {dialog === 'cancel' && <Modal title="Cancel campaign" onClose={() => { if (!busy) setDialog(undefined) }}>
      <div className="modal-form">
        <p>Request cancellation of {campaign?.title}. Inspect the event journal for its recorded outcome.</p>
        {errorText(actionError)}
        <button type="button" className="button button--danger" disabled={busy || Boolean(snapshotError)}
          onClick={() => void mutate('cancel')}>{busy ? 'Cancelling…' : 'Request cancellation'}</button>
      </div>
    </Modal>}
  </div>
}
