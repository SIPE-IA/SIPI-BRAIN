<script setup lang="ts">
import { ref } from 'vue'
import type { LoginInput } from '../types'

defineProps<{ busy: boolean }>()
const emit = defineEmits<{ submit: [input: LoginInput] }>()
const username = ref('')
const password = ref('')

function submit() {
  emit('submit', { username: username.value, password: password.value })
  password.value = ''
}
</script>

<template>
  <section aria-labelledby="login-heading" class="login">
    <h2 id="login-heading">Sign in</h2>
    <p>Use the account created locally by your administrator.</p>
    <form :aria-busy="busy" @submit.prevent="submit">
      <label>Username<input v-model="username" name="username" autocomplete="username" required :disabled="busy" /></label>
      <label>Password<input v-model="password" name="password" type="password" autocomplete="current-password" required :disabled="busy" /></label>
      <button type="submit" :disabled="busy">{{ busy ? 'Signing in…' : 'Sign in' }}</button>
    </form>
  </section>
</template>

<style scoped>
.login { max-width: 460px; margin: 1rem auto; }
</style>
