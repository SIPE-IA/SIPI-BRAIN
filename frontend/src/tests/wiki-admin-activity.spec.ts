import { flushPromises, mount } from '@vue/test-utils'
import { expect, test, vi } from 'vitest'
import App from '../App.vue'
import { jsonResponse } from './setup'

test('activity distinguishes loading, empty, and failure and replaces revoked author data', async () => {
  const date = '2026-11-18T10:30:00Z'
  let assigned = true
  let fail = false
  const fetchMock = vi.fn<typeof fetch>(async (path) => {
    if (path === '/api/session') return jsonResponse({ user: { id: 'admin', username: 'admin', role: 'admin' }, csrf_token: 'token' })
    if (path === '/api/wikis') return jsonResponse({ items: [] })
    if (path !== '/api/admin/wiki-activity') throw new Error('Unexpected request')
    if (fail) return jsonResponse({}, 500)
    return jsonResponse({ items: [
      { id: 'first', name: 'First wiki', agent: { id: 'a', name: 'First agent' }, record_count: 1, source_count: 2,
        last_contribution_at: date, recent_activity: [{ occurred_at: date, ...(assigned ? { author_name: 'synthetic-author' } : {}) }] },
      { id: 'second', name: 'Second wiki', agent: { id: 'b', name: 'Second agent' }, record_count: 0, source_count: 0,
        last_contribution_at: null, recent_activity: [] },
    ] })
  })
  vi.stubGlobal('fetch', fetchMock)
  const app = mount(App)
  await flushPromises()
  await app.get('button[aria-controls="wiki-admin-activity"]').trigger('click')
  await flushPromises()
  expect(app.text()).toContain('First agent')
  expect(app.text()).toContain('Knowledge records: 1')
  expect(app.text()).toContain('Original sources: 2')
  expect(app.text()).toContain('Wiki is empty.')
  expect(app.text()).toContain('synthetic-author')

  assigned = false
  await app.get('#wiki-admin-activity button').trigger('click')
  await flushPromises()
  expect(app.text()).not.toContain('synthetic-author')
  expect(app.findAll(`time[datetime="${date}"]`)).toHaveLength(2)

  fail = true
  await app.get('#wiki-admin-activity button').trigger('click')
  await flushPromises()
  expect(app.find('#wiki-admin-activity [role="alert"]').exists()).toBe(true)
  expect(app.text()).not.toContain('Wiki is empty.')
  expect(fetchMock.mock.calls.every(([path]) => !String(path).includes('/records'))).toBe(true)
  app.unmount()
})
