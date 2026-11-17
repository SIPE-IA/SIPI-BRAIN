<script setup lang="ts">
import { onMounted, reactive, ref } from 'vue'
import { ApiClient, ApiError, errorMessage } from '../api'
import type { Conflict, EpistemicKind, RecordDetail, User } from '../types'

const props = defineProps<{ api: ApiClient; wikiId: string; user: User }>()
const emit = defineEmits<{ resolved: [record: RecordDetail] }>()
const conflicts = ref<Conflict[]>([])
const drafts = reactive<Record<string, { body: string; reason: string; sources: string; kind: EpistemicKind; reviewed: boolean; blocked: boolean }>>({})
const busy = ref(false)
const error = ref('')
const notice = ref('')

async function load() {
  busy.value = true
  error.value = ''
  try {
    conflicts.value = (await props.api.conflicts(props.wikiId)).items
    for (const conflict of conflicts.value) {
      const draft = drafts[conflict.id]
      if (draft) {
        draft.reviewed = false
        draft.blocked = false
      } else {
        drafts[conflict.id] = { body: '', reason: '', sources: '', kind: 'assumption', reviewed: false, blocked: false }
      }
    }
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally {
    busy.value = false
  }
}

async function resolve(conflict: Conflict) {
  const draft = drafts[conflict.id]
  if (busy.value || !draft?.reviewed || draft.blocked) return
  busy.value = true
  error.value = ''
  notice.value = ''
  try {
    const record = await props.api.resolve(props.wikiId, conflict.id, {
      expected_generation: conflict.generation, body_markdown: draft.body,
      epistemic_kind: draft.kind, reason: draft.reason,
      source_ids: draft.sources.split(/[\s,]+/).filter(Boolean),
    })
    notice.value = 'Resolution saved. All candidate proposals remain preserved.'
    emit('resolved', record)
    await load()
  } catch (cause) {
    if (cause instanceof ApiError && cause.code === 'conflict_changed') {
      draft.blocked = true
      draft.reviewed = false
    }
    error.value = errorMessage(cause)
  } finally {
    busy.value = false
  }
}

onMounted(load)
</script>

<template>
  <section aria-labelledby="conflicts-heading" :aria-busy="busy">
    <h3 id="conflicts-heading">Pending conflicts</h3>
    <p>No final winner. All candidates are preserved until an assigned administrator resolves the case manually.</p>
    <button :disabled="busy" @click="load">Reload conflicts</button>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
    <p v-if="notice" role="status">{{ notice }}</p>
    <p v-if="!busy && conflicts.length === 0">No pending conflicts.</p>
    <article v-for="conflict in conflicts" :key="conflict.id" class="conflict">
      <h4>Record {{ conflict.record_id }} · pending</h4>
      <small>Case {{ conflict.id }} · generation {{ conflict.generation }} · base {{ conflict.base_revision_id }}</small>
      <div class="candidate-grid">
        <section v-for="proposal in conflict.proposals" :key="proposal.id" class="candidate">
          <h5>Candidate · {{ proposal.revision.author_id }}</h5>
          <small>{{ proposal.revision.created_at }} · {{ proposal.revision.epistemic_kind }}</small>
          <pre>{{ proposal.revision.body_markdown }}</pre>
          <small>Proposal {{ proposal.id }} · revision {{ proposal.revision.id }}</small>
        </section>
      </div>
      <form v-if="user.role === 'admin' && drafts[conflict.id]" name="conflict-resolution" @submit.prevent="resolve(conflict)">
        <label>Manual resolution<textarea v-model="drafts[conflict.id]!.body" name="resolution-body" rows="5" required :disabled="busy" /></label>
        <label>Decision reason<textarea v-model="drafts[conflict.id]!.reason" name="resolution-reason" required :disabled="busy" /></label>
        <label>Knowledge category<select v-model="drafts[conflict.id]!.kind" name="resolution-epistemic-kind" :disabled="busy"><option value="assumption">Assumption</option><option value="decision">Decision</option><option value="verified_fact">Verified fact</option></select></label>
        <label>Source IDs (comma separated)<input v-model="drafts[conflict.id]!.sources" name="resolution-source-ids" :disabled="busy" /></label>
        <label class="checkbox"><input v-model="drafts[conflict.id]!.reviewed" name="resolution-reviewed" type="checkbox" required :disabled="busy || drafts[conflict.id]!.blocked" />I reviewed every candidate in this generation.</label>
        <button type="submit" :disabled="busy || drafts[conflict.id]!.blocked || !drafts[conflict.id]!.reviewed">Save manual resolution</button>
      </form>
    </article>
  </section>
</template>
