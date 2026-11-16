import { mount, flushPromises } from '@vue/test-utils'
import { expect, test, vi } from 'vitest'
import App from '../App.vue'
import { jsonResponse } from './setup'

test('mounts the application shell', async () => {
  vi.stubGlobal('fetch', vi.fn().mockResolvedValue(jsonResponse({ user: null, csrf_token: 'setup-token' })))
  const wrapper = mount(App)
  await flushPromises()
  expect(wrapper.get('h1').text()).toBe('SIPI-brain')
  wrapper.unmount()
})
