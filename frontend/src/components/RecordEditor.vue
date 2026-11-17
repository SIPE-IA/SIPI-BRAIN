<script setup lang="ts">
import { ref, watch } from 'vue'
import { ApiClient, errorMessage } from '../api'
import type { EpistemicKind, ProposalResult, RecordDetail, RecordKind } from '../types'

const props = defineProps<{ api: ApiClient; wikiId: string; record: RecordDetail | null }>()
const emit = defineEmits<{ saved: [record: RecordDetail]; proposed: [result: ProposalResult] }>()
const title = ref('')
const kind = ref<RecordKind>('prompt')
const projectId = ref('')
const clientId = ref('')
const body = ref('')
const epistemicKind = ref<EpistemicKind>('assumption')
const sourceIds = ref('')
const submissionId = ref(crypto.randomUUID())
const busy = ref(false)
const error = ref('')
const notice = ref('')

watch(() => props.record?.head_revision_id, () => {
  const record = props.record
  title.value = record?.title ?? ''
  kind.value = record?.kind ?? 'prompt'
  projectId.value = record?.project_id ?? ''
  clientId.value = record?.client_id ?? ''
  body.value = record?.revision.body_markdown ?? ''
  epistemicKind.value = record?.revision.epistemic_kind ?? 'assumption'
  sourceIds.value = record?.sources.map(source => source.id).join(', ') ?? ''
  submissionId.value = crypto.randomUUID()
  error.value = ''
  notice.value = ''
}, { immediate: true })

async function save() {
  if (busy.value) return
  busy.value = true
  error.value = ''
  notice.value = ''
  const content = {
    submission_id: submissionId.value,
    body_markdown: body.value,
    epistemic_kind: epistemicKind.value,
    source_ids: sourceIds.value.split(/[\s,]+/).filter(Boolean),
  }
  try {
    if (props.record) {
      const result = await props.api.propose(props.wikiId, props.record.id, { ...content, base_revision_id: props.record.head_revision_id })
      notice.value = result.state === 'pending' ? 'Proposal preserved; pending administrator resolution. No final winner.' : 'Saved proposal.'
      emit('proposed', result)
    } else {
      const record = await props.api.createRecord(props.wikiId, {
        ...content, title: title.value, kind: kind.value,
        project_id: projectId.value || null, client_id: clientId.value || null,
      })
      notice.value = 'Saved record.'
      emit('saved', record)
    }
  } catch (cause) {
    error.value = errorMessage(cause)
  } finally {
    busy.value = false
  }
}
</script>

<template>
  <section :aria-busy="busy" aria-labelledby="editor-heading">
    <h3 id="editor-heading">{{ record ? 'Propose an edit' : 'Contribute knowledge' }}</h3>
    <p>Ordinary drafts stay in this view until a save is confirmed. Do not enter secrets.</p>
    <p v-if="record?.state === 'conflicted'" class="warning">This record is conflicted. An edit adds a candidate; it cannot resolve the conflict.</p>
    <p v-if="error" role="alert" class="error">{{ error }}</p>
    <p v-if="notice" role="status">{{ notice }}</p>
    <form :name="record ? 'record-edit' : 'record-create'" @submit.prevent="save">
      <template v-if="!record">
        <label>Title<input v-model="title" name="record-title" required :disabled="busy" /></label>
        <label>Kind<select v-model="kind" name="record-kind" :disabled="busy"><option value="prompt">Prompt</option><option value="decision">Decision</option><option value="note">Note</option><option value="rfd">RFD — functional requirement</option></select></label>
        <div class="field-grid">
          <label>Project ID<input v-model="projectId" name="record-project" :required="kind === 'rfd'" :disabled="busy" placeholder="project-alfa" /></label>
          <label>Client ID (optional)<input v-model="clientId" name="record-client" :disabled="busy" placeholder="client-alfa" /></label>
        </div>
      </template>
      <label>Knowledge text<textarea v-model="body" name="record-body" rows="7" required :disabled="busy" /></label>
      <label>Knowledge category<select v-model="epistemicKind" name="record-epistemic-kind" :disabled="busy"><option value="assumption">Assumption</option><option value="decision">Decision</option><option value="verified_fact">Verified fact</option></select></label>
      <label>Source IDs (comma separated)<input v-model="sourceIds" name="record-source-ids" :disabled="busy" /></label>
      <small>Submission: {{ submissionId }}. Retry unchanged after an unconfirmed save; changing a confirmed submission is rejected.</small>
      <button type="submit" :disabled="busy">{{ busy ? 'Saving…' : record ? 'Save proposal' : 'Save record' }}</button>
    </form>
  </section>
</template>
