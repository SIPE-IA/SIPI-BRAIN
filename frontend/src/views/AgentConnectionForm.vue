<script setup lang="ts">
import { computed, onMounted, ref, watch } from 'vue'
import { type ApiClient, errorMessage } from '../api'
import type { AgentOnboardingItem, ConnectionDestination, ConnectionResult } from '../types'

const { api } = defineProps<{ api: ApiClient }>()
const agents = ref<AgentOnboardingItem[]>([])
const destinations = ref<ConnectionDestination[]>([])
const selected = ref('')
const destination = ref('')
const identity = ref('')
const loading = ref(true)
const saving = ref(false)
const checking = ref(false)
const error = ref('')
const result = ref<ConnectionResult['status']>()
let request = 0
const current = computed(() => agents.value.find(agent => agent.id === selected.value))
const saved = computed(() => current.value?.connection)
const allowed = computed(() => destinations.value.some(item => item.id === destination.value))
const canTest = computed(() => allowed.value && saved.value?.destination_id === destination.value
  && saved.value.expected_identity === identity.value.trim())
const messages: Record<ConnectionResult['status'], string> = {
  verified: 'Connection verified.',
  unreachable: 'Destination did not respond.',
  identity_unverified: 'Agent identity could not be verified.',
  not_testable: 'Connection cannot be tested.',
}

function invalidate() {
  request++
  checking.value = false
  result.value = undefined
}
watch([destination, identity], invalidate, { flush: 'sync' })
watch(selected, () => {
  destination.value = saved.value?.destination_id ?? ''
  identity.value = saved.value?.expected_identity ?? ''
  error.value = ''
  invalidate()
}, { flush: 'sync' })

async function load() {
  loading.value = true
  error.value = ''
  invalidate()
  try {
    const [list, approvals] = await Promise.all([api.adminAgents(), api.connectionDestinations()])
    agents.value = list.items
    destinations.value = approvals.items
    destination.value = saved.value?.destination_id ?? ''
    identity.value = saved.value?.expected_identity ?? ''
  } catch (cause) {
    destinations.value = []
    error.value = errorMessage(cause)
  } finally {
    loading.value = false
  }
}

async function save() {
  const agent = current.value
  if (!agent || loading.value || saving.value || !allowed.value || !identity.value.trim()) return
  saving.value = true
  error.value = ''
  invalidate()
  try {
    agent.connection = await api.saveConnection(agent.id, { destination_id: destination.value, expected_identity: identity.value.trim() })
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally {
    saving.value = false
  }
}

async function test() {
  const revision = saved.value?.revision
  if (!revision || !canTest.value || checking.value || saving.value || loading.value) return
  const tag = ++request
  checking.value = true
  result.value = undefined
  error.value = ''
  try {
    const reply = await api.testConnection(selected.value, revision)
    if (tag === request && reply.revision === saved.value?.revision) result.value = reply.status
  } catch (cause) {
    if (tag === request) error.value = errorMessage(cause)
  } finally {
    if (tag === request) checking.value = false
  }
}

onMounted(load)
</script>

<template>
  <section id="agent-connections" aria-label="Agent connections">
    <p v-if="loading" role="status">Loading connections…</p>
    <p v-if="error" role="alert">{{ error }}</p>
    <button type="button" :disabled="loading || saving" @click="load">Refresh connections</button>
    <label>Connection agent
      <select v-model="selected" name="connection-agent" :disabled="loading || saving">
        <option value="">Select an agent</option>
        <option v-for="agent in agents" :key="agent.id" :value="agent.id">{{ agent.name }}</option>
      </select>
    </label>
    <p v-if="!loading && !destinations.length" role="status">No approved destinations.</p>
    <form v-if="current" id="agent-connection-form" @submit.prevent="save">
      <label>Approved destination
        <select v-model="destination" name="connection-destination" :disabled="loading || saving">
          <option value="">Select a destination</option>
          <option v-if="destination && !allowed" :value="destination" disabled>Saved destination unavailable</option>
          <option v-for="item in destinations" :key="item.id" :value="item.id">{{ item.label }}</option>
        </select>
      </label>
      <label>Expected identity <input v-model="identity" name="expected-identity" required :disabled="loading || saving"></label>
      <button type="submit" :disabled="loading || saving || !allowed || !identity.trim()">Save connection</button>
      <button type="button" :disabled="loading || saving || checking || !canTest" @click="test">Test connection</button>
      <p v-if="result" role="status">{{ messages[result] }}</p>
      <p v-else-if="saved && !allowed" role="status">Connection cannot be tested.</p>
    </form>
  </section>
</template>
