<script setup lang="ts">
import type { ApiClient } from '../api'
import type { User, Wiki } from '../types'

defineProps<{ api: ApiClient; wiki: Wiki; user: User }>()
</script>

<template>
  <section><h2>{{ wiki.name }}</h2><p>Knowledge view pending implementation.</p></section>
</template>
