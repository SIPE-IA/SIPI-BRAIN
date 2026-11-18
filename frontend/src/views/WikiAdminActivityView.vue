<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { type ApiClient, errorMessage } from '../api'
import type { WikiActivitySummary } from '../types'

const { api } = defineProps<{ api: ApiClient }>()
const items = ref<WikiActivitySummary[] | null>(null)
const loading = ref(true)
const error = ref('')
const dateFormat = new Intl.DateTimeFormat(undefined, { dateStyle: 'medium', timeStyle: 'medium' })

async function load() {
  loading.value = true
  items.value = null
  error.value = ''
  try {
    items.value = (await api.wikiActivity()).items
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally {
    loading.value = false
  }
}

onMounted(load)
</script>

<template>
  <section id="wiki-admin-activity" aria-labelledby="wiki-admin-activity-title" :aria-busy="loading">
    <h2 id="wiki-admin-activity-title">Wiki activity</h2>
    <button :disabled="loading" @click="load">Refresh activity</button>
    <p v-if="loading" role="status">Loading activity…</p>
    <p v-else-if="error" role="alert" class="error">{{ error }}</p>
    <template v-else-if="items">
      <p v-if="!items.length" role="status">No wikis found.</p>
      <ul v-else class="wiki-list">
        <li v-for="wiki in items" :key="wiki.id">
          <h3>{{ wiki.name }}</h3>
          <p>Agent: {{ wiki.agent.name }}</p>
          <p>Knowledge records: {{ wiki.record_count }} · Original sources: {{ wiki.source_count }}</p>
          <p v-if="wiki.last_contribution_at">Last contribution:
            <time :datetime="wiki.last_contribution_at">{{ dateFormat.format(new Date(wiki.last_contribution_at)) }}</time>
          </p>
          <p v-else-if="!wiki.record_count && !wiki.source_count && !wiki.recent_activity.length">Wiki is empty.</p>
          <p v-else>No contribution activity recorded.</p>
          <template v-if="wiki.recent_activity.length">
            <h4>Recent activity</h4>
            <ol class="event-list">
              <li v-for="(entry, index) in wiki.recent_activity" :key="index">
                <time :datetime="entry.occurred_at">{{ dateFormat.format(new Date(entry.occurred_at)) }}</time>
                <span v-if="entry.author_name"> · {{ entry.author_name }}</span>
              </li>
            </ol>
          </template>
        </li>
      </ul>
    </template>
  </section>
</template>
