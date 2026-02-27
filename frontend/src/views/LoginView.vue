<template>
  <div class="auth-page">
    <div class="auth-card">
      <div class="auth-header">
        <span class="brand-icon">🎵</span>
        <h1 class="brand-name">Trecker</h1>
      </div>

      <form @submit.prevent="handleLogin" class="auth-form">
        <div class="field">
          <label for="email">Email</label>
          <InputText
            id="email"
            v-model="email"
            type="email"
            placeholder="you@example.com"
            autocomplete="email"
            :disabled="authStore.loading"
            class="w-full"
          />
        </div>

        <div class="field">
          <label for="password">Password</label>
          <Password
            id="password"
            v-model="password"
            placeholder="Password"
            :feedback="false"
            toggleMask
            :disabled="authStore.loading"
            class="w-full"
            inputClass="w-full"
          />
        </div>

        <Message v-if="error" severity="error" :closable="false">{{ error }}</Message>

        <Button
          type="submit"
          label="Sign in"
          :loading="authStore.loading"
          class="w-full"
        />
      </form>

      <p class="auth-footer">
        Don't have an account?
        <RouterLink to="/register">Register</RouterLink>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, useRoute, RouterLink } from 'vue-router'
import InputText from 'primevue/inputtext'
import Password from 'primevue/password'
import Button from 'primevue/button'
import Message from 'primevue/message'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()
const router = useRouter()
const route = useRoute()

const email = ref('')
const password = ref('')
const error = ref('')

async function handleLogin() {
  error.value = ''
  try {
    await authStore.login({ email: email.value, password: password.value })
    const redirect = route.query.redirect as string
    router.push(redirect || '/queue')
  } catch {
    error.value = 'Invalid email or password'
  }
}
</script>

<style scoped>
.auth-page {
  min-height: 100vh;
  display: flex;
  align-items: center;
  justify-content: center;
  background: var(--p-surface-950);
}

.auth-card {
  width: 100%;
  max-width: 400px;
  padding: 2rem;
  background: var(--p-surface-900);
  border: 1px solid var(--p-surface-700);
  border-radius: 12px;
}

.auth-header {
  text-align: center;
  margin-bottom: 2rem;
}

.brand-icon {
  font-size: 2.5rem;
}

.brand-name {
  margin: 0.5rem 0 0;
  font-size: 1.75rem;
  font-weight: 700;
  color: var(--p-text-color);
}

.auth-form {
  display: flex;
  flex-direction: column;
  gap: 1rem;
}

.field {
  display: flex;
  flex-direction: column;
  gap: 0.375rem;
}

.field label {
  font-size: 0.875rem;
  font-weight: 500;
  color: var(--p-text-color);
}

.auth-footer {
  margin-top: 1.25rem;
  text-align: center;
  font-size: 0.875rem;
  color: var(--p-text-muted-color);
}

.auth-footer a {
  color: var(--p-primary-color);
  text-decoration: none;
}

.auth-footer a:hover {
  text-decoration: underline;
}
</style>
