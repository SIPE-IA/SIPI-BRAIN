<script setup lang="ts">
import { onMounted, ref, watch } from 'vue'
import { ApiClient, errorMessage } from '../api'
import type { KnowledgeReview, MaintenanceInput, RecordDetail, ReviewResult, WikiEvent, WikiMaintenance } from '../types'

const props = defineProps<{ api: ApiClient; wikiId: string; record?: RecordDetail | null }>()
const emit = defineEmits<{ reviewed: [review: KnowledgeReview] }>()
const result = ref<ReviewResult>('needs_review')
const note = ref('')
const review = ref<KnowledgeReview | null>(null)
const maintenance = ref<WikiMaintenance | null>(null)
const rules = ref('')
const status = ref<MaintenanceInput['status']>('pending')
const events = ref<WikiEvent[]>([])
const busy = ref(false)
const error = ref('')
const notice = ref('')

watch(() => props.record?.head_revision_id, () => {
  review.value = props.record?.review ?? null
  result.value = props.record?.review?.result ?? 'needs_review'
  note.value = ''
  notice.value = ''
}, { immediate: true })

async function saveReview() {
  if (busy.value || !props.record) return
  busy.value = true
  error.value = ''
  notice.value = ''
  try {
    review.value = await props.api.review(props.wikiId, props.record.id, {
      revision_id: props.record.head_revision_id, result: result.value, note: note.value,
    })
    emit('reviewed', review.value)
    notice.value = 'Review saved for this revision only.'
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally { busy.value = false }
}

async function loadMaintenance() {
  busy.value = true
  error.value = ''
  try {
    const [current, history] = await Promise.all([props.api.maintenance(props.wikiId), props.api.events(props.wikiId)])
    maintenance.value = current
    rules.value = current.rules
    status.value = current.status
    events.value = history.items
  } catch (cause) { error.value = errorMessage(cause) } finally { busy.value = false }
}

async function saveMaintenance() {
  if (busy.value) return
  busy.value = true
  error.value = ''
  notice.value = ''
  try {
    maintenance.value = await props.api.setMaintenance(props.wikiId, { rules: rules.value, status: status.value })
    notice.value = 'Maintenance saved.'
    events.value = (await props.api.events(props.wikiId)).items
  } catch (cause) { error.value = errorMessage(cause) } finally { busy.value = false }
}

onMounted(() => { if (!props.record) return loadMaintenance() })
</script>

<template>
  <section :aria-busy="busy" :aria-label="record ? 'Knowledge review' : 'Maintenance and events'">
    <p v-if="error" role="alert" class="error">{{ error }}</p>
    <p v-if="notice" role="status">{{ notice }}</p>
    <template v-if="record">
      <h3>Human validity review</h3>
      <p v-if="!review" class="warning">Not reviewed for this revision. Check sources and current validity before reuse.</p>
      <div v-else><p>Validity: <strong>{{ review.result }}</strong> · {{ review.reviewer_id }} · {{ review.reviewed_at }}</p><pre>{{ review.note }}</pre></div>
      <p>A knowledge category does not replace review. A new revision does not inherit this assessment.</p>
      <form name="knowledge-review" @submit.prevent="saveReview">
        <label>Review result<select v-model="result" name="review-result" :disabled="busy"><option value="needs_review">Needs review</option><option value="verified">Verified</option><option value="outdated">Outdated</option></select></label>
        <label>Review note<textarea v-model="note" name="review-note" rows="3" required :disabled="busy" /></label>
        <button type="submit" :disabled="busy">Save review</button>
      </form>
    </template>
    <template v-else>
      <h3>Wiki maintenance rules</h3>
      <p>Rules are context, not executable instructions. Review is manual; this view does not analyze or delete knowledge.</p>
      <small v-if="maintenance">Status {{ maintenance.status }} · reviewed {{ maintenance.reviewed_at ?? 'not reviewed' }}</small>
      <form name="wiki-maintenance" @submit.prevent="saveMaintenance">
        <label>Maintenance rules<textarea v-model="rules" name="maintenance-rules" rows="7" :disabled="busy" /></label>
        <label>Manual review status<select v-model="status" name="maintenance-status" :disabled="busy"><option value="pending">Pending</option><option value="reviewed">Reviewed</option></select></label>
        <button type="submit" :disabled="busy">Save maintenance</button>
      </form>
      <h3>Chronological events</h3>
      <p>Metadata only; original bodies and credentials do not appear here.</p>
      <p v-if="events.length === 0">No events.</p>
      <ol class="event-list"><li v-for="event in events" :key="event.id"><time>{{ event.created_at }}</time> · {{ event.action }}<small>Actor {{ event.actor_id }} · reference {{ event.reference_id }}</small></li></ol>
      <button :disabled="busy" @click="loadMaintenance">Refresh maintenance & events</button>
    </template>
  </section>
</template>
