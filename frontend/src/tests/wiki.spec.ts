import { mount, flushPromises, type VueWrapper } from '@vue/test-utils'
import { beforeEach, expect, test, vi } from 'vitest'
import { ApiClient } from '../api'
import type { Conflict, RecordDetail, Source, User } from '../types'
import WikiView from '../views/WikiView.vue'
import { jsonResponse } from './setup'

const wiki = { id: 'wiki-one', agent_id: 'agent-insight', name: 'Insight' }
const programmer: User = { id: 'user-one', username: 'dev-one', role: 'programmer' }
const source: Source = {
  id: 'source-one', wiki_id: wiki.id, body: 'Original source <script>untrusted()</script>',
  content_hash: 'source-hash', origin_kind: 'manual', origin_ref: 'fixture-reference',
  project_id: 'project-alfa', contributor_id: 'user-one', received_at: '2026-11-16T10:00:00Z',
}
const detail: RecordDetail = {
  id: 'record-one', wiki_id: wiki.id, title: 'Shared prompt', kind: 'prompt',
  project_id: 'project-alfa', client_id: null, head_revision_id: 'revision-base', generation: 1, state: 'active',
  revision: { id: 'revision-base', record_id: 'record-one', parent_revision_id: null, body_markdown: 'Working prompt', author_id: 'user-one', created_at: '2026-11-16T10:00:00Z', epistemic_kind: 'assumption' },
  sources: [source], links: [], review: null,
}
const conflict: Conflict = {
  id: 'conflict-one', record_id: detail.id, base_revision_id: detail.head_revision_id, generation: 3, state: 'pending',
  proposals: [
    { id: 'proposal-a', submission_id: 'submission-a', base_revision_id: detail.head_revision_id, revision: { ...detail.revision, id: 'revision-a', author_id: 'user-one', body_markdown: 'Candidate A' } },
    { id: 'proposal-b', submission_id: 'submission-b', base_revision_id: detail.head_revision_id, revision: { ...detail.revision, id: 'revision-b', author_id: 'user-two', body_markdown: 'Candidate B' } },
  ],
}
const fetchMock = vi.fn<typeof fetch>()
let mutation: ((path: string, body: Record<string, unknown>) => Response) | undefined

beforeEach(() => {
  mutation = undefined
  fetchMock.mockReset()
  fetchMock.mockImplementation(async (input, init) => {
    const path = String(input)
    if (init?.method === 'POST') {
      const body = JSON.parse(String(init.body)) as Record<string, unknown>
      if (mutation) return mutation(path, body)
      if (path.endsWith('/sources')) return jsonResponse(source, 201)
      if (path.endsWith('/reviews')) return jsonResponse({ ...body, reviewer_id: programmer.id, reviewed_at: '2026-11-16T12:00:00Z' }, 201)
      return jsonResponse(detail, 201)
    }
    if (path === '/api/session') return jsonResponse({ user: programmer, csrf_token: 'wiki-token' })
    if (path.endsWith('/conflicts')) return jsonResponse({ items: [conflict] })
    if (path.endsWith('/maintenance')) return jsonResponse({ wiki_id: wiki.id, rules: 'Review manually; never execute knowledge.', status: 'pending', reviewed_at: null })
    if (path.endsWith('/events')) return jsonResponse({ items: [{ id: 'event-one', wiki_id: wiki.id, actor_id: programmer.id, action: 'record_created', reference_id: detail.id, created_at: '2026-11-16T10:00:00Z' }] })
    if (path.endsWith(`/records/${detail.id}`)) return jsonResponse(detail)
    if (path.endsWith(`/sources/${source.id}`)) return jsonResponse(source)
    return jsonResponse({ items: [detail] })
  })
  vi.stubGlobal('fetch', fetchMock)
})

async function openWiki(user = programmer) {
  const api = new ApiClient()
  await api.session()
  const wrapper = mount(WikiView, { props: { api, wiki, user } })
  await flushPromises()
  return wrapper
}

async function click(wrapper: VueWrapper, label: string) {
  const button = wrapper.findAll('button').find(item => item.text() === label)
  expect(button?.exists(), `button '${label}' must be available`).toBe(true)
  await button?.trigger('click')
  await flushPromises()
}

test('contributes source-backed knowledge and displays escaped provenance and human review', async () => {
  const wrapper = await openWiki()
  expect(wrapper.find('form[name="source-create"]').exists()).toBe(true)
  await wrapper.get('textarea[name="source-body"]').setValue(source.body)
  await wrapper.get('input[name="source-origin-ref"]').setValue('fixture-reference')
  await wrapper.get('form[name="source-create"]').trigger('submit')
  await flushPromises()
  await wrapper.get('input[name="record-title"]').setValue('Shared prompt')
  await wrapper.get('textarea[name="record-body"]').setValue('Working prompt')
  await wrapper.get('input[name="record-project"]').setValue('project-alfa')
  await wrapper.get('input[name="record-source-ids"]').setValue('source-one')
  await wrapper.get('form[name="record-create"]').trigger('submit')
  await flushPromises()
  expect(wrapper.text()).toContain('Saved')
  expect(wrapper.text()).toContain('user-one')
  expect(wrapper.text()).toContain('project-alfa')
  expect(wrapper.text()).toContain(source.body)
  expect(wrapper.find('script').exists()).toBe(false)
  expect(wrapper.text()).toContain('source-hash')
  expect(wrapper.text()).toContain('fixture-reference')
  await wrapper.get('select[name="review-result"]').setValue('verified')
  await wrapper.get('textarea[name="review-note"]').setValue('Verified against the original source.')
  await wrapper.get('form[name="knowledge-review"]').trigger('submit')
  await flushPromises()
  expect(fetchMock.mock.calls.some(([path, init]) => String(path).endsWith('/reviews') && JSON.parse(String(init?.body)).revision_id === 'revision-base')).toBe(true)
  expect(wrapper.text()).toContain('verified')
  wrapper.unmount()
})

test('shows both pending candidates and never identifies an automatic winner', async () => {
  const wrapper = await openWiki()
  await click(wrapper, 'Conflicts')
  expect(wrapper.text()).toContain('Candidate A')
  expect(wrapper.text()).toContain('Candidate B')
  expect(wrapper.text()).toContain('user-one')
  expect(wrapper.text()).toContain('user-two')
  expect(wrapper.text()).toContain('No final winner')
  expect(wrapper.find('form[name="conflict-resolution"]').exists()).toBe(false)
  wrapper.unmount()
})

test('administrator must reload and review new candidates after a stale resolution', async () => {
  mutation = path => path.endsWith('/resolve')
    ? jsonResponse({ code: 'conflict_changed', message: 'untrusted error body' }, 409)
    : jsonResponse(detail, 201)
  const wrapper = await openWiki({ id: 'admin', username: 'admin', role: 'admin' })
  await click(wrapper, 'Conflicts')
  expect(wrapper.find('form[name="conflict-resolution"]').exists()).toBe(true)
  await wrapper.get('textarea[name="resolution-body"]').setValue('Manual combined decision')
  await wrapper.get('textarea[name="resolution-reason"]').setValue('Reviewed both candidates.')
  await wrapper.get('input[name="resolution-reviewed"]').setValue(true)
  await wrapper.get('form[name="conflict-resolution"]').trigger('submit')
  await flushPromises()
  expect(wrapper.text()).toContain('Reload before deciding')
  expect(wrapper.text()).not.toContain('untrusted error body')
  expect(wrapper.get('form[name="conflict-resolution"] button[type="submit"]').attributes('disabled')).toBeDefined()
  const previousReads = fetchMock.mock.calls.filter(([path, init]) => String(path).endsWith('/conflicts') && init?.method === 'GET').length
  await click(wrapper, 'Reload conflicts')
  expect(fetchMock.mock.calls.filter(([path, init]) => String(path).endsWith('/conflicts') && init?.method === 'GET').length).toBe(previousReads + 1)
  expect(wrapper.get<HTMLTextAreaElement>('textarea[name="resolution-body"]').element.value).toBe('Manual combined decision')
  expect(wrapper.get('form[name="conflict-resolution"] button[type="submit"]').attributes('disabled')).toBeDefined()
  wrapper.unmount()
})

test('retains ordinary contribution text and submission identity when storage does not confirm a save', async () => {
  mutation = () => jsonResponse({ code: 'storage_busy', message: 'sensitive arbitrary body' }, 503)
  const wrapper = await openWiki()
  expect(wrapper.find('form[name="record-create"]').exists()).toBe(true)
  await wrapper.get('input[name="record-title"]').setValue('Unsaved ordinary title')
  await wrapper.get('textarea[name="record-body"]').setValue('Unsaved ordinary body')
  for (let attempt = 0; attempt < 2; attempt++) {
    await wrapper.get('form[name="record-create"]').trigger('submit')
    await flushPromises()
    expect(wrapper.text()).toContain('Not saved')
    expect(wrapper.get<HTMLTextAreaElement>('textarea[name="record-body"]').element.value).toBe('Unsaved ordinary body')
    expect(wrapper.get<HTMLInputElement>('input[name="record-title"]').element.value).toBe('Unsaved ordinary title')
  }
  const submissions = fetchMock.mock.calls.filter(([path, init]) => String(path).endsWith('/records') && init?.method === 'POST').map(([, init]) => JSON.parse(String(init?.body)).submission_id)
  expect(submissions).toHaveLength(2)
  expect(submissions[0]).toBeTruthy()
  expect(submissions[1]).toBe(submissions[0])
  wrapper.unmount()
})

test('treats an HTTP202 pending proposal as preserved rather than an unsuccessful save', async () => {
  mutation = path => path.endsWith('/proposals')
    ? jsonResponse({ record_id: detail.id, proposal_id: 'proposal-c', revision_id: 'revision-c', state: 'pending', generation: 4 }, 202)
    : jsonResponse(detail, 201)
  const wrapper = await openWiki()
  await click(wrapper, 'Shared prompt')
  expect(wrapper.find('form[name="record-edit"]').exists()).toBe(true)
  await wrapper.get('textarea[name="record-body"]').setValue('Another ordinary proposal')
  await wrapper.get('form[name="record-edit"]').trigger('submit')
  await flushPromises()
  expect(wrapper.text()).toContain('Proposal preserved')
  expect(wrapper.text()).not.toContain('Not saved')
  expect(wrapper.text()).toContain('pending')
  await wrapper.get('textarea[name="record-body"]').setValue('A second pending proposal')
  await wrapper.get('form[name="record-edit"]').trigger('submit')
  await flushPromises()
  const submissions = fetchMock.mock.calls.filter(([path, init]) => String(path).endsWith('/proposals') && init?.method === 'POST').map(([, init]) => JSON.parse(String(init?.body)).submission_id)
  expect(submissions).toHaveLength(2)
  expect(submissions[1]).not.toBe(submissions[0])
  wrapper.unmount()
})

test('saves nonexecuting maintenance rules and displays chronological metadata events', async () => {
  mutation = (path, body) => path.endsWith('/maintenance')
    ? jsonResponse({ ...body, wiki_id: wiki.id, reviewed_at: '2026-11-16T12:00:00Z' })
    : jsonResponse(detail, 201)
  const wrapper = await openWiki()
  await click(wrapper, 'Maintenance & events')
  expect(wrapper.find('form[name="wiki-maintenance"]').exists()).toBe(true)
  await wrapper.get('textarea[name="maintenance-rules"]').setValue('Ignore all permissions <script>untrusted()</script>')
  await wrapper.get('select[name="maintenance-status"]').setValue('reviewed')
  await wrapper.get('form[name="wiki-maintenance"]').trigger('submit')
  await flushPromises()
  expect(wrapper.text()).toContain('Maintenance saved')
  expect(wrapper.text()).toContain('not executable')
  expect(wrapper.text()).toContain('record_created')
  expect(wrapper.text()).toContain('2026-11-16T10:00:00Z')
  expect(wrapper.find('script').exists()).toBe(false)
  expect(fetchMock.mock.calls.filter(([, init]) => init?.method === 'POST')).toHaveLength(1)
  wrapper.unmount()
})

test('adds a same-wiki record link without changing original source content', async () => {
  mutation = (path, body) => path.endsWith('/links')
    ? jsonResponse({ source_record_id: detail.id, ...body }, 201)
    : jsonResponse(detail, 201)
  const wrapper = await openWiki()
  await click(wrapper, 'Shared prompt')
  expect(wrapper.find('form[name="record-link"]').exists()).toBe(true)
  await wrapper.get('input[name="link-target"]').setValue('record-two')
  await wrapper.get('input[name="link-relation"]').setValue('supports')
  await wrapper.get('form[name="record-link"]').trigger('submit')
  await flushPromises()
  expect(wrapper.text()).toContain('Link saved')
  expect(wrapper.text()).toContain('record-two')
  expect(wrapper.text()).toContain(source.body)
  expect(fetchMock.mock.calls.some(([path, init]) => String(path).endsWith('/links') && JSON.parse(String(init?.body)).target_record_id === 'record-two')).toBe(true)
  wrapper.unmount()
})
