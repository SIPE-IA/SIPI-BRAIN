<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ApiClient, errorMessage } from '../api'
import type { AssignmentIndex } from '../types'

const props = defineProps<{ api: ApiClient }>()
const index = ref<AssignmentIndex>({ users: [], wikis: [], items: [] })
const userId = ref('')
const wikiId = ref('')
const busy = ref(false)
const error = ref('')
const notice = ref('')

async function load() {
  index.value = await props.api.assignments()
}

async function update(action: 'grant' | 'revoke') {
  busy.value = true
  error.value = ''
  notice.value = ''
  try {
    await props.api.setAssignment({ user_id: userId.value, wiki_id: wikiId.value, action })
    notice.value = action === 'grant' ? 'Assignment granted.' : 'Assignment revoked.'
    await load()
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally {
    busy.value = false
  }
}

onMounted(async () => {
  busy.value = true
  try { await load() } catch (cause) { error.value = errorMessage(cause) } finally { busy.value = false }
})
</script>

<template>
  <section aria-labelledby="assignments-heading" :aria-busy="busy">
    <h2 id="assignments-heading">Wiki assignments</h2>
    <p>Assignments grant wiki access. Administrator status alone does not grant access to knowledge.</p>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
    <p v-if="notice" role="status">{{ notice }}</p>
    <form @submit.prevent="update('grant')">
      <label>User<select v-model="userId" name="assignment-user" required :disabled="busy"><option value="" disabled>Select a user</option><option v-for="user in index.users" :key="user.id" :value="user.id">{{ user.username }} ({{ user.role }})</option></select></label>
      <label>Wiki<select v-model="wikiId" name="assignment-wiki" required :disabled="busy"><option value="" disabled>Select a wiki</option><option v-for="wiki in index.wikis" :key="wiki.id" :value="wiki.id">{{ wiki.name }}</option></select></label>
      <div class="actions">
        <button type="submit" :disabled="busy || !userId || !wikiId">Grant access</button>
        <button type="button" :disabled="busy || !userId || !wikiId" @click="update('revoke')">Revoke access</button>
      </div>
    </form>
    <p v-if="index.items.length === 0">No assignments.</p>
    <table v-else>
      <caption>Current assignments</caption>
      <thead><tr><th scope="col">User</th><th scope="col">Wiki</th></tr></thead>
      <tbody><tr v-for="item in index.items" :key="`${item.user_id}:${item.wiki_id}`"><td>{{ index.users.find(user => user.id === item.user_id)?.username ?? item.user_id }}</td><td>{{ index.wikis.find(wiki => wiki.id === item.wiki_id)?.name ?? item.wiki_id }}</td></tr></tbody>
    </table>
  </section>
</template>
