import { flushPromises, mount } from '@vue/test-utils'
import { expect, test, vi } from 'vitest'
import App from '../App.vue'
import { jsonResponse } from './setup'

test('creation is administrator-only and confirms only a saved wiki', async () => {
  const agent = { id: 'eligible', name: 'Eligible agent' }
  const wiki = { id: 'wiki', name: 'New Wiki', agent }
  let role = 'programmer'
  let failSave = true
  let failRefresh = false
  let saved = false
  const fetchMock = vi.fn<typeof fetch>(async (path, init) => {
    if (path === '/api/session') return jsonResponse({ user: { id: 'u', username: 'user', role }, csrf_token: 'token' })
    if (path === '/api/wikis') return jsonResponse({ items: [] })
    if (path !== '/api/admin/wikis') throw new Error('Unexpected request')
    if (init?.method !== 'POST') return failRefresh ? jsonResponse({}, 500)
      : jsonResponse({ items: saved ? [wiki] : [], eligible_agents: saved ? [] : [agent] })
    if (failSave) return jsonResponse({ code: 'wiki_already_exists' }, 409)
    expect(JSON.parse(String(init.body))).toEqual({ name: 'New Wiki', agent_id: 'eligible' })
    failRefresh = true
    saved = true
    return jsonResponse(wiki, 201)
  })
  vi.stubGlobal('fetch', fetchMock)

  const programmer = mount(App)
  await flushPromises()
  expect(programmer.text()).not.toContain('Manage wikis')
  programmer.unmount()

  role = 'admin'
  const admin = mount(App)
  await flushPromises()
  await admin.get('button[aria-controls="wiki-admin-create"]').trigger('click')
  await flushPromises()
  await admin.get('button[aria-controls="wiki-create-form"]').trigger('click')
  await admin.get('input[name="wiki-name"]').setValue('New Wiki')
  await admin.get('select[name="wiki-agent"]').setValue('eligible')
  await admin.get('#wiki-create-form').trigger('submit')
  await flushPromises()
  expect(admin.text()).toContain('already has a wiki')
  expect(admin.text()).not.toContain('Wiki created.')
  expect(admin.find('#wiki-admin-create li').exists()).toBe(false)

  failSave = false
  await admin.get('#wiki-create-form').trigger('submit')
  await flushPromises()
  expect(admin.text()).toContain('Wiki created.')
  expect(admin.text()).toContain('could not refresh')
  expect(admin.get('#wiki-admin-create li').text()).toContain('New Wiki')
  admin.unmount()

  failRefresh = false
  const reloaded = mount(App)
  await flushPromises()
  await reloaded.get('button[aria-controls="wiki-admin-create"]').trigger('click')
  await flushPromises()
  expect(reloaded.get('#wiki-admin-create li').text()).toContain('New Wiki Eligible agent')
  reloaded.unmount()
})
