<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { ApiClient, errorMessage } from '../api'
import type { KnowledgeReview, ProposalResult, RecordDetail, RecordSummary, Source, User, Wiki } from '../types'
import RecordEditor from '../components/RecordEditor.vue'
import ConflictsView from './ConflictsView.vue'
import KnowledgeReviewView from './KnowledgeReviewView.vue'

const props = defineProps<{ api: ApiClient; wiki: Wiki; user: User }>()
const tab = ref<'knowledge' | 'conflicts' | 'maintenance'>('knowledge')
const records = ref<RecordSummary[]>([])
const selected = ref<RecordDetail | null>(null)
const offset = ref(0)
const loading = ref(false)
const error = ref('')
const notice = ref('')
const sourceBody = ref('')
const sourceOrigin = ref('manual')
const sourceRef = ref('')
const sourceProject = ref('')
const sourceId = ref('')
const openedSource = ref<Source | null>(null)
const sourceBusy = ref(false)
const targetId = ref('')
const relation = ref('related')
const linkBusy = ref(false)

async function loadIndex() {
  records.value = (await props.api.records(props.wiki.id, offset.value)).items
}

async function refreshIndex(nextOffset = offset.value) {
  loading.value = true
  error.value = ''
  offset.value = nextOffset
  try { await loadIndex() } catch (cause) { error.value = errorMessage(cause) } finally { loading.value = false }
}

async function openRecord(id: string) {
  loading.value = true
  error.value = ''
  notice.value = ''
  selected.value = null
  try { selected.value = await props.api.record(props.wiki.id, id) } catch (cause) { error.value = errorMessage(cause) } finally { loading.value = false }
}

async function saved(record: RecordDetail) {
  selected.value = record
  notice.value = 'Saved record. Confirmation received after commit.'
  try { await loadIndex() } catch (cause) { error.value = `Saved; index refresh failed. ${errorMessage(cause)}` }
}

async function proposed(result: ProposalResult) {
  notice.value = result.state === 'pending' ? 'Proposal preserved; pending administrator resolution. No final winner.' : 'Saved proposal.'
  if (result.state === 'pending' && selected.value) {
    selected.value = { ...selected.value, state: 'conflicted', generation: result.generation }
  }
  try {
    if (result.state === 'active') selected.value = await props.api.record(props.wiki.id, result.record_id)
    await loadIndex()
  } catch (cause) { error.value = `Save confirmed; refresh failed. ${errorMessage(cause)}` }
}

function reviewed(review: KnowledgeReview) {
  if (selected.value?.head_revision_id === review.revision_id) selected.value = { ...selected.value, review }
}

async function createSource() {
  if (sourceBusy.value) return
  sourceBusy.value = true
  error.value = ''
  notice.value = ''
  try {
    openedSource.value = await props.api.createSource(props.wiki.id, {
      body: sourceBody.value, origin_kind: sourceOrigin.value, origin_ref: sourceRef.value,
      project_id: sourceProject.value || null,
    })
    sourceId.value = openedSource.value.id
    sourceBody.value = ''
    notice.value = 'Source saved unchanged. Use its ID as a reference in your knowledge.'
  } catch (cause) { error.value = errorMessage(cause) } finally { sourceBusy.value = false }
}

async function readSource() {
  sourceBusy.value = true
  openedSource.value = null
  error.value = ''
  try { openedSource.value = await props.api.source(props.wiki.id, sourceId.value) } catch (cause) { error.value = errorMessage(cause) } finally { sourceBusy.value = false }
}

async function saveLink() {
  if (linkBusy.value || !selected.value) return
  linkBusy.value = true
  error.value = ''
  notice.value = ''
  try {
    const link = await props.api.link(props.wiki.id, selected.value.id, { target_record_id: targetId.value, relation: relation.value })
    selected.value = { ...selected.value, links: [...selected.value.links.filter(item => item.target_record_id !== link.target_record_id || item.relation !== link.relation), link] }
    notice.value = 'Link saved.'
  } catch (cause) { error.value = errorMessage(cause) } finally { linkBusy.value = false }
}

onMounted(() => refreshIndex(0))
</script>

<template>
  <div class="wiki-workspace">
    <h2>{{ wiki.name }}</h2>
    <small>Wiki {{ wiki.id }} · responsible agent {{ wiki.agent_id }}</small>
    <nav aria-label="Wiki tools">
      <button :aria-pressed="tab === 'knowledge'" @click="tab = 'knowledge'">Knowledge</button>
      <button :aria-pressed="tab === 'conflicts'" @click="tab = 'conflicts'">Conflicts</button>
      <button :aria-pressed="tab === 'maintenance'" @click="tab = 'maintenance'">Maintenance & events</button>
    </nav>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
    <p v-if="notice" role="status">{{ notice }}</p>
    <ConflictsView v-if="tab === 'conflicts'" :api="api" :wiki-id="wiki.id" :user="user" @resolved="record => { tab = 'knowledge'; saved(record) }" />
    <KnowledgeReviewView v-else-if="tab === 'maintenance'" :api="api" :wiki-id="wiki.id" />
    <template v-else>
      <section aria-labelledby="index-heading" :aria-busy="loading">
        <h3 id="index-heading">Knowledge index</h3>
        <p v-if="loading" role="status">Loading knowledge…</p>
        <p v-else-if="records.length === 0">No knowledge on this page.</p>
        <ul class="record-list"><li v-for="record in records" :key="record.id"><button @click="openRecord(record.id)">{{ record.title }}</button><small>{{ record.kind }} · {{ record.project_id ?? 'No project' }} · {{ record.state }}</small></li></ul>
        <div class="actions">
          <button :disabled="loading || offset === 0" @click="refreshIndex(Math.max(0, offset - 50))">Previous page</button>
          <button :disabled="loading || records.length < 50" @click="refreshIndex(offset + 50)">Next page</button>
          <button :disabled="loading" @click="refreshIndex()">Refresh index</button>
          <button :disabled="loading" @click="selected = null; notice = ''">New record</button>
        </div>
      </section>
      <section v-if="selected" aria-labelledby="detail-heading">
        <h3 id="detail-heading">{{ selected.title }}</h3>
        <p v-if="selected.state === 'conflicted'" class="warning"><strong>Pending conflict. No final winner.</strong> The following revision is an antecedent, not current final knowledge.</p>
        <dl class="metadata">
          <dt>Record / kind</dt><dd>{{ selected.id }} / {{ selected.kind }}</dd>
          <dt>Project / client</dt><dd>{{ selected.project_id ?? 'No project' }} / {{ selected.client_id ?? 'No external client' }}</dd>
          <dt>Author / date</dt><dd>{{ selected.revision.author_id }} / {{ selected.revision.created_at }}</dd>
          <dt>Revision / generation</dt><dd>{{ selected.head_revision_id }} / {{ selected.generation }}</dd>
          <dt>Knowledge category</dt><dd>{{ selected.revision.epistemic_kind }}</dd>
        </dl>
        <pre>{{ selected.revision.body_markdown }}</pre>
        <h4>Original source references</h4>
        <p v-if="selected.sources.length === 0">No source references. Verify before reuse.</p>
        <article v-for="source in selected.sources" :key="source.id" class="source">
          <strong>{{ source.id }}</strong><small>{{ source.origin_kind }} · {{ source.origin_ref }} · contributor {{ source.contributor_id }} · {{ source.received_at }}</small>
          <small>Project {{ source.project_id ?? 'not specified' }} · integrity hash {{ source.content_hash }}</small>
          <pre>{{ source.body }}</pre>
        </article>
        <small>Hashes detect content changes; they do not establish the truth or authority of a source.</small>
        <h4>Related knowledge in this wiki</h4>
        <p v-if="selected.links.length === 0">No links.</p>
        <ul><li v-for="link in selected.links" :key="`${link.target_record_id}:${link.relation}`"><button @click="openRecord(link.target_record_id)">{{ link.target_record_id }}</button> · {{ link.relation }}</li></ul>
        <form name="record-link" @submit.prevent="saveLink">
          <label>Related record ID<input v-model="targetId" name="link-target" required :disabled="linkBusy" /></label>
          <label>Relationship<input v-model="relation" name="link-relation" required :disabled="linkBusy" /></label>
          <button type="submit" :disabled="linkBusy">Add same-wiki link</button>
        </form>
      </section>
      <KnowledgeReviewView v-if="selected" :api="api" :wiki-id="wiki.id" :record="selected" @reviewed="reviewed" />
      <RecordEditor :key="selected?.id ?? 'new'" :api="api" :wiki-id="wiki.id" :record="selected" @saved="saved" @proposed="proposed" />
      <section aria-labelledby="source-heading" :aria-busy="sourceBusy">
        <h3 id="source-heading">Original sources</h3>
        <p>Sources are preserved as submitted. Knowledge synthesis never edits the original.</p>
        <form name="source-create" @submit.prevent="createSource">
          <label>Original source text<textarea v-model="sourceBody" name="source-body" rows="4" required :disabled="sourceBusy" /></label>
          <div class="field-grid">
            <label>Origin kind<input v-model="sourceOrigin" name="source-origin-kind" required :disabled="sourceBusy" /></label>
            <label>Origin reference<input v-model="sourceRef" name="source-origin-ref" required :disabled="sourceBusy" /></label>
            <label>Source project ID (optional)<input v-model="sourceProject" name="source-project" :disabled="sourceBusy" /></label>
          </div>
          <button type="submit" :disabled="sourceBusy">Preserve source</button>
        </form>
        <form name="source-read" @submit.prevent="readSource">
          <label>Source ID<input v-model="sourceId" name="source-id" required :disabled="sourceBusy" /></label>
          <button type="submit" :disabled="sourceBusy">Read immutable source</button>
        </form>
        <article v-if="openedSource" class="source">
          <h4>{{ openedSource.id }}</h4><small>{{ openedSource.origin_kind }} · {{ openedSource.origin_ref }} · contributor {{ openedSource.contributor_id }} · {{ openedSource.received_at }}</small>
          <small>Project {{ openedSource.project_id ?? 'not specified' }} · integrity hash {{ openedSource.content_hash }}</small><pre>{{ openedSource.body }}</pre>
        </article>
      </section>
    </template>
  </div>
</template>
