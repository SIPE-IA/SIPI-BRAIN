<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { type ApiClient, errorMessage } from '../api'
import AgentConnectionForm from './AgentConnectionForm.vue'
import type { AgentOnboardingItem, WikiAdminIndex } from '../types'

const { api } = defineProps<{ api: ApiClient }>()
const index = ref<WikiAdminIndex | null>(null)
const loading = ref(false)
const saving = ref(false)
const showForm = ref(false)
const name = ref('')
const agentId = ref('')
const error = ref('')
const confirmed = ref(false)
const agents = ref<AgentOnboardingItem[]>([])
const registerOpen = ref(false)
const loadingAgents = ref(false)
const registering = ref(false)
const registered = ref(false)
const registrationId = ref('')
const registrationName = ref('')
const registrationError = ref('')
const connectionsOpen = ref(false)
let wikiListRequest = 0

async function load(afterRegistration = false) {
  const request = ++wikiListRequest
  loading.value = true
  error.value = ''
  try {
    const list = await api.adminWikis()
    if (request === wikiListRequest) index.value = list
  } catch (cause) {
    if (request !== wikiListRequest) return
    error.value = afterRegistration
      ? 'Agent saved, but could not refresh the list. Retry.'
      : confirmed.value ? 'Wiki saved, but could not refresh the list. Retry.' : errorMessage(cause)
  } finally {
    if (request === wikiListRequest) loading.value = false
  }
}

async function openRegistration() {
  registerOpen.value = !registerOpen.value
  if (!registerOpen.value) return
  registrationError.value = ''
  loadingAgents.value = true
  try {
    agents.value = (await api.adminAgents()).items
  } catch (cause) {
    registrationError.value = errorMessage(cause)
  } finally {
    loadingAgents.value = false
  }
}

async function register() {
  if (loadingAgents.value || registering.value || !registrationId.value.trim() || !registrationName.value.trim()) return
  registering.value = true
  registered.value = false
  registrationError.value = ''
  try {
    const agent = await api.registerAgent({ id: registrationId.value.trim(), name: registrationName.value.trim() })
    agents.value = [...agents.value.filter(item => item.id !== agent.id), agent]
    registered.value = true
    registrationId.value = ''
    registrationName.value = ''
    await load(true)
  } catch (cause) {
    registrationError.value = errorMessage(cause)
  } finally {
    registering.value = false
  }
}

async function create() {
  const catalog = index.value
  if (saving.value || loading.value || !catalog || !name.value.trim() || !agentId.value) return
  saving.value = true
  error.value = ''
  confirmed.value = false
  try {
    const wiki = await api.createWiki({ name: name.value.trim(), agent_id: agentId.value })
    index.value = {
      items: [...catalog.items, wiki],
      eligible_agents: catalog.eligible_agents.filter(agent => agent.id !== wiki.agent.id),
    }
    confirmed.value = true
    showForm.value = false
    name.value = ''
    agentId.value = ''
    await load()
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally {
    saving.value = false
  }
}

onMounted(() => load())
</script>

<template>
  <section id="wiki-admin-create" aria-labelledby="wiki-admin-create-title">
    <h2 id="wiki-admin-create-title">Manage wikis</h2>
    <p v-if="loading" role="status">Loading wikis…</p>
    <p v-if="confirmed" role="status">Wiki created.</p>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
    <button v-if="error && (!index || confirmed || registered)" :disabled="loading || saving || registering" @click="load(registered && !confirmed)">Retry list</button>
    <button type="button" aria-controls="agent-register-form" :aria-expanded="registerOpen" :disabled="registering || loadingAgents" @click="openRegistration">Register agent</button>
    <p v-if="registered" role="status">Agent registered.</p>
    <p v-if="registrationError" role="alert" class="error">{{ registrationError }}</p>
    <template v-if="registerOpen">
      <form id="agent-register-form" @submit.prevent="register">
        <label>Agent ID <input v-model="registrationId" name="agent-id" required :disabled="registering"></label>
        <label>Agent name <input v-model="registrationName" name="agent-name" required :disabled="registering"></label>
        <button type="submit" :disabled="registering || loadingAgents || !registrationId.trim() || !registrationName.trim()">Save agent</button>
      </form>
      <ul v-if="agents.length" class="wiki-list">
        <li v-for="agent in agents" :key="agent.id">{{ agent.name }} <small>{{ agent.id }}</small></li>
      </ul>
    </template>
    <button type="button" aria-controls="agent-connections" :aria-expanded="connectionsOpen" @click="connectionsOpen = !connectionsOpen">Configure connections</button>
    <AgentConnectionForm v-if="connectionsOpen" :api="api" />
    <template v-if="index">
      <button type="button" aria-controls="wiki-create-form" :aria-expanded="showForm" :disabled="loading || saving || !index.eligible_agents.length" @click="showForm = !showForm">Create wiki</button>
      <p v-if="!index.eligible_agents.length" role="status">No eligible agents.</p>
      <form v-if="showForm" id="wiki-create-form" @submit.prevent="create">
        <label>Wiki name <input v-model="name" name="wiki-name" required :disabled="saving"></label>
        <label>Agent
          <select v-model="agentId" name="wiki-agent" required :disabled="saving">
            <option value="">Select an agent</option>
            <option v-for="agent in index.eligible_agents" :key="agent.id" :value="agent.id">{{ agent.name }}</option>
          </select>
        </label>
        <button type="submit" :disabled="loading || saving || !name.trim() || !agentId">Save wiki</button>
      </form>
      <ul v-if="index.items.length" class="wiki-list">
        <li v-for="wiki in index.items" :key="wiki.id">{{ wiki.name }} <small>{{ wiki.agent.name }}</small></li>
      </ul>
    </template>
  </section>
</template>
