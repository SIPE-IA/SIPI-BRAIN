<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { type ApiClient, errorMessage } from '../api'
import type { WikiAdminIndex } from '../types'

const { api } = defineProps<{ api: ApiClient }>()
const index = ref<WikiAdminIndex | null>(null)
const loading = ref(false)
const saving = ref(false)
const showForm = ref(false)
const name = ref('')
const agentId = ref('')
const error = ref('')
const confirmed = ref(false)

async function load() {
  loading.value = true
  error.value = ''
  try {
    index.value = await api.adminWikis()
  } catch (cause) {
    error.value = confirmed.value ? 'Wiki saved, but could not refresh the list. Retry.' : errorMessage(cause)
  } finally {
    loading.value = false
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

onMounted(load)
</script>

<template>
  <section id="wiki-admin-create" aria-labelledby="wiki-admin-create-title">
    <h2 id="wiki-admin-create-title">Manage wikis</h2>
    <p v-if="loading" role="status">Loading wikis…</p>
    <p v-if="confirmed" role="status">Wiki created.</p>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
    <button v-if="error && (!index || confirmed)" :disabled="loading || saving" @click="load">Retry list</button>
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
