<template>
  <div class="auth-page">
    <div class="auth-card">
      <div class="auth-header">
        <span class="brand-icon">🎵</span>
        <h1 class="brand-name">Trecker</h1>
        <p class="auth-subtitle">Create your account</p>
      </div>

      <form @submit.prevent="handleRegister" class="auth-form">
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
            placeholder="Minimum 8 characters"
            :feedback="false"
            toggleMask
            :disabled="authStore.loading"
            class="w-full"
            inputClass="w-full"
          />
        </div>

        <div class="field">
          <label for="confirmPassword">Confirm Password</label>
          <Password
            id="confirmPassword"
            v-model="confirmPassword"
            placeholder="Repeat your password"
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
          label="Create account"
          :loading="authStore.loading"
          class="w-full"
        />
      </form>

      <p class="auth-footer">
        Already have an account?
        <RouterLink to="/login">Sign in</RouterLink>
      </p>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref } from 'vue'
import { useRouter, RouterLink } from 'vue-router'
import InputText from 'primevue/inputtext'
import Password from 'primevue/password'
import Button from 'primevue/button'
import Message from 'primevue/message'
import { useAuthStore } from '@/stores/auth'

const authStore = useAuthStore()
const router = useRouter()

const email = ref('')
const password = ref('')
const confirmPassword = ref('')
const error = ref('')

async function handleRegister() {
  error.value = ''

  if (password.value !== confirmPassword.value) {
    error.value = 'Passwords do not match'
    return
  }

  if (password.value.length < 8) {
    error.value = 'Password must be at least 8 characters'
    return
  }

  try {
    await authStore.register({ email: email.value, password: password.value })
    router.push('/queue')
  } catch {
    error.value = 'Registration failed. Please try again.'
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

.auth-subtitle {
  margin: 0.25rem 0 0;
  font-size: 0.875rem;
  color: var(--p-text-muted-color);
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
