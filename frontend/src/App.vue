<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ApiClient, errorMessage } from './api'
import type { LoginInput, SessionState, Wiki } from './types'
import AssignmentsView from './views/AssignmentsView.vue'
import LoginView from './views/LoginView.vue'

const api = new ApiClient()
const state = ref<SessionState | null>(null)
const wikis = ref<Wiki[]>([])
const loading = ref(true)
const busy = ref(false)
const error = ref('')
const showAssignments = ref(false)

async function loadWikis() {
  wikis.value = state.value?.user ? (await api.wikis()).items : []
}

async function initialize() {
  loading.value = true
  error.value = ''
  try {
    state.value = await api.session()
    await loadWikis()
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally {
    loading.value = false
  }
}

async function login(input: LoginInput) {
  busy.value = true
  error.value = ''
  try {
    state.value = await api.login(input)
    await loadWikis()
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally {
    busy.value = false
  }
}

async function logout() {
  busy.value = true
  error.value = ''
  try {
    state.value = await api.logout()
    wikis.value = []
    showAssignments.value = false
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally {
    busy.value = false
  }
}

onMounted(initialize)
</script>

<template>
  <div class="shell">
    <header>
      <div>
        <p class="eyebrow">SIPECOM · Local synthetic pilot</p>
        <h1>SIPI-brain</h1>
        <p>Persistent knowledge, one wiki per agent.</p>
      </div>
      <div v-if="state?.user" class="account">
        <span>{{ state.user.username }} · {{ state.user.role }}</span>
        <button :disabled="busy" @click="logout">Sign out</button>
      </div>
    </header>
    <main>
      <p v-if="loading" role="status">Loading your session…</p>
      <template v-else>
        <p v-if="error" role="alert" class="error">{{ error }}</p>
        <button v-if="!state" @click="initialize">Retry connection</button>
        <LoginView v-else-if="!state.user" :busy="busy" @submit="login" />
        <template v-else>
          <nav aria-label="Workspace">
            <button :aria-pressed="!showAssignments" @click="showAssignments = false; loadWikis().catch(cause => error = errorMessage(cause))">My wikis</button>
            <button v-if="state.user.role === 'admin'" :aria-pressed="showAssignments" @click="showAssignments = true">Manage assignments</button>
          </nav>
          <AssignmentsView v-if="showAssignments && state.user.role === 'admin'" :api="api" />
          <section v-else aria-labelledby="wiki-heading">
            <h2 id="wiki-heading">Assigned wikis</h2>
            <p>You can consult and contribute only to wikis assigned by an administrator.</p>
            <p v-if="wikis.length === 0" role="status">No wikis assigned.</p>
            <ul v-else class="wiki-list">
              <li v-for="wiki in wikis" :key="wiki.id"><strong>{{ wiki.name }}</strong><small>{{ wiki.id }}</small></li>
            </ul>
          </section>
        </template>
      </template>
    </main>
    <footer>Knowledge is context, not authority. Do not enter real data or secrets in this pilot.</footer>
  </div>
</template>

<style>
:root { font-family: system-ui, sans-serif; color: #172b37; background: #f3f6f8; line-height: 1.6; }
* { box-sizing: border-box; }
body { margin: 0; }
.shell { max-width: 1100px; margin: auto; padding: 2rem; }
header { display: flex; justify-content: space-between; align-items: start; gap: 2rem; padding-bottom: 1.5rem; border-bottom: 1px solid #cbd6dd; }
h1 { margin: 0; font-size: 2.5rem; letter-spacing: -.04em; }
h2 { margin-top: 0; }
.eyebrow { text-transform: uppercase; letter-spacing: .1em; font-size: .75rem; color: #386276; font-weight: 700; }
.account { display: flex; align-items: center; gap: 1rem; }
main { padding: 1.5rem 0; min-height: 350px; }
section { background: white; border: 1px solid #d8e1e6; border-radius: 12px; padding: 1.5rem; margin-bottom: 1rem; }
form { display: grid; gap: 1rem; }
label { display: grid; gap: .3rem; font-weight: 600; }
input, select, textarea, button { font: inherit; }
input, select, textarea { width: 100%; border: 1px solid #8b9fab; padding: .65rem .8rem; border-radius: 6px; background: white; color: inherit; }
button { cursor: pointer; border: 1px solid #2c6172; color: #174454; background: #eef6f8; padding: .55rem .9rem; border-radius: 6px; }
button:hover:not(:disabled), button[aria-pressed="true"] { background: #d8edf1; }
button[type="submit"], .primary { background: #185c70; color: white; }
button:disabled { opacity: .6; cursor: wait; }
:focus-visible { outline: 3px solid #c26a0b; outline-offset: 3px; }
nav { display: flex; gap: .7rem; margin-bottom: 1.5rem; }
.actions { display: flex; flex-wrap: wrap; gap: .7rem; }
.error { background: #fff0ed; color: #8c2b19; border-left: 4px solid #b44630; padding: .8rem 1rem; }
.wiki-list { list-style: none; padding: 0; display: grid; gap: .8rem; }
.wiki-list li { border: 1px solid #d8e1e6; border-radius: 8px; padding: 1rem; }
small { display: block; color: #536975; overflow-wrap: anywhere; }
footer { color: #536975; font-size: .8rem; border-top: 1px solid #cbd6dd; padding-top: 1rem; }
table { border-collapse: collapse; width: 100%; margin-top: 1rem; }
th, td { text-align: left; padding: .6rem; border-bottom: 1px solid #d8e1e6; overflow-wrap: anywhere; }
@media (max-width: 650px) { .shell { padding: 1rem; } header, .account { flex-direction: column; gap: .6rem; } section { padding: 1rem; } }
</style>
