import { flushPromises, mount } from '@vue/test-utils'
import { expect, test, vi } from 'vitest'
import { spawn, spawnSync } from 'node:child_process'
import { once } from 'node:events'
import { existsSync, mkdtempSync, readFileSync, rmSync, rmdirSync, writeFileSync } from 'node:fs'
import { tmpdir } from 'node:os'
import { join, resolve } from 'node:path'
import { ApiClient } from '../api'
import WikiAdminCreateView from '../views/WikiAdminCreateView.vue'
import { jsonResponse } from './setup'

test('fixture revocation recovers when a competing revoke process is forcibly killed', async () => {
  const root = mkdtempSync(join(tmpdir(), 'sipibrain-revoke-'))
  const catalog = join(root, 'catalog.json')
  const marker = join(root, 'entered')
  const preload = join(root, 'pause.cjs')
  const script = resolve('../scripts/local-connection-fixture.cjs')
  const initial = JSON.stringify({ version: 1, destinations: [{ id: 'first' }, { id: 'second' }] })
  const args = (id: string) => [script, 'revoke', '--catalog', catalog, '--destination', id]
  const revoke = (id: string) => spawnSync(process.execPath, args(id), { timeout: 2000, windowsHide: true })
  writeFileSync(catalog, initial)
  writeFileSync(preload, `
    const fs = require('node:fs')
    const read = fs.readFileSync
    fs.readFileSync = function (file, ...args) {
      if (file === ${JSON.stringify(catalog)}) {
        fs.writeFileSync(${JSON.stringify(marker)}, '')
        Atomics.wait(new Int32Array(new SharedArrayBuffer(4)), 0, 0)
      }
      return read.call(this, file, ...args)
    }
  `)
  const holder = spawn(process.execPath, ['--require', preload, ...args('first')], {
    stdio: 'ignore', windowsHide: true,
  })
  const closed = once(holder, 'close')
  try {
    await vi.waitFor(() => expect(existsSync(marker)).toBe(true))
    expect(revoke('first').status).not.toBe(0)
    expect(readFileSync(catalog, 'utf8')).toBe(initial)
    expect(holder.kill('SIGKILL')).toBe(true)
    await closed
    expect(readFileSync(catalog, 'utf8')).toBe(initial)
    expect(revoke('first').status).toBe(0)
    expect(revoke('second').status).toBe(0)
    expect(JSON.parse(readFileSync(catalog, 'utf8'))).toEqual({ version: 3, destinations: [] })
  } finally {
    if (holder.exitCode === null && holder.signalCode === null) holder.kill('SIGKILL')
    await closed
    for (const file of [catalog, catalog + '.lock', marker, preload]) rmSync(file, { force: true })
    rmdirSync(root)
  }
})

test('uses a saved approved destination and discards a reply after configuration changes', async () => {
  let revision = 'r1'
  let resolveTest!: (response: Response) => void
  const firstTest = new Promise<Response>(resolve => { resolveTest = resolve })
  let status = ''
  const fetchMock = vi.fn<typeof fetch>(async (path, init) => {
    if (path === '/api/session') return jsonResponse({ user: null, csrf_token: 'token' })
    if (path === '/api/admin/wikis') return jsonResponse({ items: [], eligible_agents: [] })
    if (path === '/api/admin/agents') return jsonResponse({ items: [
      { id: 'a', name: 'Agent', active: true, connection: { destination_id: 'fixture', expected_identity: 'first', revision } },
    ] })
    if (path === '/api/admin/connection-destinations') return jsonResponse({ items: [{ id: 'fixture', label: 'Approved local fixture' }] })
    if (path === '/api/admin/agents/a/connection') {
      expect(init?.method).toBe('PUT')
      expect(JSON.parse(String(init?.body))).toEqual({ destination_id: 'fixture', expected_identity: 'second' })
      revision = 'r2'
      return jsonResponse({ destination_id: 'fixture', expected_identity: 'second', revision })
    }
    if (path === '/api/admin/agents/a/connection/test') {
      expect(JSON.parse(String(init?.body))).toEqual({ revision })
      return revision === 'r1' ? firstTest : jsonResponse({ revision, status })
    }
    throw new Error('Unexpected request')
  })
  vi.stubGlobal('fetch', fetchMock)
  const view = mount(WikiAdminCreateView, { props: { api: new ApiClient() } })
  await flushPromises()
  const trigger = (label: string) => {
    const button = view.findAll('button').find(item => item.text() === label)
    expect(button).toBeDefined()
    return button!.trigger('click')
  }
  await trigger('Configure connections')
  await flushPromises()
  await view.get('select[name="connection-agent"]').setValue('a')
  expect(view.get('select[name="connection-destination"]').text()).toContain('Approved local fixture')
  expect(fetchMock.mock.calls.some(([path]) => String(path).endsWith('/test'))).toBe(false)
  await trigger('Test connection')
  await view.get('input[name="expected-identity"]').setValue('second')
  await view.get('#agent-connection-form').trigger('submit')
  await flushPromises()
  resolveTest(jsonResponse({ revision: 'r1', status: 'verified' }))
  await flushPromises()
  expect(view.text()).not.toContain('Connection verified.')
  for (const [code, copy] of [
    ['verified', 'Connection verified.'],
    ['unreachable', 'Destination did not respond.'],
    ['identity_unverified', 'Agent identity could not be verified.'],
    ['not_testable', 'Connection cannot be tested.'],
  ]) {
    status = code!
    await trigger('Test connection')
    await flushPromises()
    expect(view.text()).toContain(copy)
  }
  view.unmount()
})
