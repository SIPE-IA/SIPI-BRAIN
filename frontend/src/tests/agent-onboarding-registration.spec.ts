import { flushPromises, mount } from '@vue/test-utils'
import { expect, test, vi } from 'vitest'
import { ApiClient } from '../api'
import WikiAdminCreateView from '../views/WikiAdminCreateView.vue'
import { jsonResponse } from './setup'

test('an earlier wiki-list response cannot replace refreshed eligibility', async () => {
  const agent = { id: 'first', name: 'First agent', active: true, connection: null }
  let finishInitial!: (value: Response) => void
  const initial = new Promise<Response>(resolve => { finishInitial = resolve })
  let loads = 0
  vi.stubGlobal('fetch', vi.fn<typeof fetch>(async (path, init) => {
    if (path === '/api/session') return jsonResponse({ user: null, csrf_token: 'token' })
    if (path === '/api/admin/wikis') return loads++ === 0
      ? initial : jsonResponse({ items: [], eligible_agents: [agent] })
    if (path === '/api/admin/agents') return init?.method === 'POST'
      ? jsonResponse(agent, 201) : jsonResponse({ items: [] })
    throw new Error('Unexpected request')
  }))
  const view = mount(WikiAdminCreateView, { props: { api: new ApiClient() } })
  await view.get('button[aria-controls="agent-register-form"]').trigger('click')
  await flushPromises()
  await view.get('input[name="agent-id"]').setValue('first')
  await view.get('input[name="agent-name"]').setValue('First agent')
  await view.get('#agent-register-form').trigger('submit')
  await flushPromises()
  finishInitial(jsonResponse({ items: [], eligible_agents: [] }))
  await flushPromises()
  await view.get('button[aria-controls="wiki-create-form"]').trigger('click')
  expect(view.get('select[name="wiki-agent"] option[value="first"]').text()).toBe('First agent')
  view.unmount()
})

test('registers the first agent, preserves duplicate errors, and refreshes eligibility', async () => {
  const agent = { id: 'first', name: 'First agent', active: true, connection: null }
  let saved = false
  let duplicate = true
  let refreshFails = true
  vi.stubGlobal('fetch', vi.fn<typeof fetch>(async (path, init) => {
    if (path === '/api/session') return jsonResponse({ user: null, csrf_token: 'token' })
    if (path === '/api/admin/wikis') {
      if (saved && refreshFails) return jsonResponse({}, 500)
      return jsonResponse({ items: [], eligible_agents: saved ? [agent] : [] })
    }
    if (path === '/api/admin/agents' && init?.method === 'POST') {
      expect(JSON.parse(String(init.body))).toEqual({ id: 'first', name: 'First agent' })
      if (duplicate) return jsonResponse({ code: 'agent_already_exists' }, 409)
      saved = true
      return jsonResponse(agent, 201)
    }
    if (path === '/api/admin/agents') return jsonResponse({ items: saved ? [agent] : [] })
    throw new Error('Unexpected request')
  }))
  const view = mount(WikiAdminCreateView, { props: { api: new ApiClient() } })
  await flushPromises()
  await view.get('button[aria-controls="agent-register-form"]').trigger('click')
  await view.get('input[name="agent-id"]').setValue(' first ')
  await view.get('input[name="agent-name"]').setValue(' First agent ')
  await view.get('#agent-register-form').trigger('submit')
  await flushPromises()
  expect(view.text()).toContain('This agent ID is already registered.')
  expect(view.text()).not.toContain('Agent registered.')

  duplicate = false
  await view.get('#agent-register-form').trigger('submit')
  await flushPromises()
  expect(view.text()).toContain('Agent registered.')
  expect(view.text()).toContain('could not refresh')
  refreshFails = false
  const retry = view.findAll('button').find(button => button.text() === 'Retry list')
  expect(retry).toBeDefined()
  await retry!.trigger('click')
  await flushPromises()
  await view.get('button[aria-controls="wiki-create-form"]').trigger('click')
  expect(view.get('select[name="wiki-agent"] option[value="first"]').text()).toBe('First agent')
  view.unmount()
})
