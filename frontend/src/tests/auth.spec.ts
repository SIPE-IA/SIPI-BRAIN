import { mount, flushPromises } from '@vue/test-utils'
import { beforeEach, expect, test, vi } from 'vitest'
import App from '../App.vue'
import { ApiClient } from '../api'
import { jsonResponse } from './setup'

const anonymous = { user: null, csrf_token: 'prelogin-token' }
const authenticated = {
  user: { id: 'user-one', username: 'dev-one', role: 'programmer' },
  csrf_token: 'rotated-token',
}
const fetchMock = vi.fn<typeof fetch>()

beforeEach(() => {
  fetchMock.mockReset()
  vi.stubGlobal('fetch', fetchMock)
})

test('loads the prelogin session and CSRF token using same-origin cookies', async () => {
  fetchMock.mockResolvedValue(jsonResponse(anonymous))
  const api = new ApiClient()
  expect(await api.session()).toEqual(anonymous)
  expect(fetchMock).toHaveBeenCalledWith('/api/session', expect.objectContaining({ credentials: 'same-origin' }))
})

test('login and logout send the current token and accept the rotated session state', async () => {
  fetchMock
    .mockResolvedValueOnce(jsonResponse(anonymous))
    .mockResolvedValueOnce(jsonResponse(authenticated))
    .mockResolvedValueOnce(jsonResponse({ user: null, csrf_token: 'logout-token' }))
  const api = new ApiClient()
  await api.session()
  expect(await api.login({ username: 'dev-one', password: 'synthetic-password' })).toEqual(authenticated)
  const loginRequest = fetchMock.mock.calls[1]?.[1]
  expect(loginRequest).toMatchObject({ method: 'POST', credentials: 'same-origin' })
  expect(new Headers(loginRequest?.headers).get('X-CSRF-Token')).toBe('prelogin-token')
  expect(JSON.parse(String(loginRequest?.body))).toEqual({ username: 'dev-one', password: 'synthetic-password' })
  expect(await api.logout()).toEqual({ user: null, csrf_token: 'logout-token' })
  expect(new Headers(fetchMock.mock.calls[2]?.[1]?.headers).get('X-CSRF-Token')).toBe('rotated-token')
})

test('failed login uses a fixed safe message rather than reflecting server content or the password', async () => {
  fetchMock
    .mockResolvedValueOnce(jsonResponse(anonymous))
    .mockResolvedValueOnce(jsonResponse({ code: 'unauthenticated', message: '<script>synthetic-password</script>' }, 401))
  const api = new ApiClient()
  await api.session()
  await expect(api.login({ username: 'dev-one', password: 'synthetic-password' })).rejects.toThrow('Sign in failed.')
})

test('shows the accessible sign-in form for an anonymous session', async () => {
  fetchMock.mockResolvedValue(jsonResponse(anonymous))
  const wrapper = mount(App)
  await flushPromises()
  expect(wrapper.find('input[autocomplete="username"]').exists()).toBe(true)
  expect(wrapper.find('input[type="password"]').exists()).toBe(true)
  wrapper.unmount()
})

test('sign-in and sign-out update visible state and clear the password', async () => {
  fetchMock.mockImplementation(async (path) => {
    if (path === '/api/login') return jsonResponse(authenticated)
    if (path === '/api/wikis') return jsonResponse({ items: [{ id: 'wiki-one', agent_id: 'agent-one', name: 'Insight' }] })
    return jsonResponse(anonymous)
  })
  const wrapper = mount(App)
  await flushPromises()
  expect(wrapper.find('form').exists()).toBe(true)
  await wrapper.get('input[autocomplete="username"]').setValue('dev-one')
  await wrapper.get('input[type="password"]').setValue('synthetic-password')
  await wrapper.get('form').trigger('submit')
  await flushPromises()
  expect(wrapper.text()).toContain('dev-one')
  expect(wrapper.text()).toContain('Insight')
  expect(wrapper.find('input[type="password"]').exists()).toBe(false)
  expect(wrapper.text()).not.toContain('Manage assignments')
  const signOut = wrapper.findAll('button').find(button => button.text() === 'Sign out')
  expect(signOut?.exists()).toBe(true)
  await signOut?.trigger('click')
  await flushPromises()
  expect(wrapper.get<HTMLInputElement>('input[type="password"]').element.value).toBe('')
  wrapper.unmount()
})

test('administrator manages assignments without loading unassigned wiki content', async () => {
  const admin = { user: { id: 'admin', username: 'admin', role: 'admin' }, csrf_token: 'admin-token' }
  const metadata = {
    users: [authenticated.user],
    wikis: [{ id: 'wiki-one', agent_id: 'agent-one', name: 'Insight' }],
    items: [],
  }
  fetchMock.mockImplementation(async (path, init) => {
    if (path === '/api/session') return jsonResponse(admin)
    if (path === '/api/wikis') return jsonResponse({ items: [] })
    if (init?.method === 'POST') return jsonResponse({ user_id: 'user-one', wiki_id: 'wiki-one', assigned: true })
    return jsonResponse(metadata)
  })
  const wrapper = mount(App)
  await flushPromises()
  const manage = wrapper.findAll('button').find(button => button.text() === 'Manage assignments')
  expect(manage?.exists()).toBe(true)
  await manage?.trigger('click')
  await flushPromises()
  await wrapper.get('select[name="assignment-user"]').setValue('user-one')
  await wrapper.get('select[name="assignment-wiki"]').setValue('wiki-one')
  for (const [label, action] of [['Grant access', 'grant'], ['Revoke access', 'revoke']]) {
    const button = wrapper.findAll('button').find(item => item.text() === label)
    expect(button?.exists()).toBe(true)
    if (action === 'grant') await wrapper.get('form').trigger('submit')
    else await button?.trigger('click')
    await flushPromises()
    expect(fetchMock.mock.calls.some(([path, init]) => path === '/api/admin/assignments' && init?.method === 'POST' && JSON.parse(String(init.body)).action === action)).toBe(true)
  }
  expect(fetchMock.mock.calls.every(([path]) => !String(path).includes('/records'))).toBe(true)
  wrapper.unmount()
})
