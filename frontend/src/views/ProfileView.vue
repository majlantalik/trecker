<template>
  <div class="profile-view">
    <div class="profile-header">
      <div class="profile-avatar">{{ userInitial }}</div>
      <div class="profile-header-info">
        <h1 class="profile-title">{{ profile?.displayName || profile?.email }}</h1>
        <p v-if="profile?.displayName" class="profile-subtitle">{{ profile?.email }}</p>
        <p class="profile-meta">Member since {{ memberSince }}</p>
      </div>
    </div>

    <div class="profile-section">
      <div class="section-header">
        <i class="pi pi-user section-icon" />
        <h2 class="section-title">Profile</h2>
      </div>
      <div class="section-body">
        <div class="form-group">
          <label class="form-label" for="display-name">Display name</label>
          <InputText
            id="display-name"
            v-model="displayName"
            placeholder="Your name"
            class="profile-input"
          />
          <p class="form-hint">Shown in the sidebar instead of your email address.</p>
        </div>
        <Button
          label="Save"
          :loading="savingProfile"
          @click="saveProfile"
          class="save-btn"
        />
      </div>
    </div>

    <div class="profile-section">
      <div class="section-header">
        <i class="pi pi-lock section-icon" />
        <h2 class="section-title">Security</h2>
      </div>
      <div class="section-body">
        <div class="form-group">
          <label class="form-label" for="current-password">Current password</label>
          <Password
            id="current-password"
            v-model="currentPassword"
            :feedback="false"
            toggleMask
            inputClass="profile-input"
            class="profile-password"
          />
        </div>
        <div class="form-group">
          <label class="form-label" for="new-password">New password</label>
          <Password
            id="new-password"
            v-model="newPassword"
            :feedback="false"
            toggleMask
            inputClass="profile-input"
            class="profile-password"
          />
          <p class="form-hint">Minimum 8 characters.</p>
        </div>
        <Message v-if="passwordError" severity="error" class="password-error">{{ passwordError }}</Message>
        <Button
          label="Change Password"
          :loading="savingPassword"
          severity="secondary"
          @click="changePassword"
          class="save-btn"
        />
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import InputText from 'primevue/inputtext'
import Password from 'primevue/password'
import Button from 'primevue/button'
import Message from 'primevue/message'
import { useToast } from 'primevue/usetoast'
import { profileApi } from '@/api/profile'
import { useAuthStore } from '@/stores/auth'
import type { ProfileDto } from '@/types'

const toast = useToast()
const authStore = useAuthStore()

const profile = ref<ProfileDto | null>(null)
const displayName = ref('')
const currentPassword = ref('')
const newPassword = ref('')
const savingProfile = ref(false)
const savingPassword = ref(false)
const passwordError = ref<string | null>(null)

const userInitial = computed(() => {
  const p = profile.value
  return (p?.displayName?.[0] ?? p?.email?.[0])?.toUpperCase() ?? '?'
})

const memberSince = computed(() => {
  if (!profile.value?.createdAt) return ''
  return new Date(profile.value.createdAt).toLocaleDateString('en-US', {
    year: 'numeric',
    month: 'long'
  })
})

onMounted(async () => {
  try {
    profile.value = await profileApi.getProfile()
    displayName.value = profile.value.displayName ?? ''
  } catch {
    toast.add({ severity: 'error', summary: 'Failed to load profile', life: 3000 })
  }
})

async function saveProfile() {
  savingProfile.value = true
  try {
    const updated = await profileApi.updateProfile({
      displayName: displayName.value.trim() || null
    })
    profile.value = updated
    authStore.updateDisplayName(updated.displayName)
    toast.add({ severity: 'success', summary: 'Profile saved', life: 2000 })
  } catch {
    toast.add({ severity: 'error', summary: 'Failed to save profile', life: 3000 })
  } finally {
    savingProfile.value = false
  }
}

async function changePassword() {
  passwordError.value = null

  if (!currentPassword.value) {
    passwordError.value = 'Current password is required.'
    return
  }
  if (newPassword.value.length < 8) {
    passwordError.value = 'New password must be at least 8 characters.'
    return
  }

  savingPassword.value = true
  try {
    await profileApi.changePassword({
      currentPassword: currentPassword.value,
      newPassword: newPassword.value
    })
    currentPassword.value = ''
    newPassword.value = ''
    toast.add({ severity: 'success', summary: 'Password changed', life: 2000 })
  } catch (e: any) {
    if (e.response?.status === 400) {
      passwordError.value = 'Current password is incorrect.'
    } else {
      passwordError.value = 'Failed to change password. Please try again.'
    }
  } finally {
    savingPassword.value = false
  }
}
</script>

<style scoped>
.profile-view {
  max-width: 540px;
  display: flex;
  flex-direction: column;
  gap: 2rem;
}

.profile-header {
  display: flex;
  align-items: center;
  gap: 1.25rem;
  padding-bottom: 2rem;
  border-bottom: 1px solid var(--tk-border);
}

.profile-avatar {
  width: 52px;
  height: 52px;
  border-radius: 50%;
  background: linear-gradient(135deg, rgba(0, 229, 176, 0.25), rgba(0, 229, 176, 0.08));
  border: 1px solid rgba(0, 229, 176, 0.25);
  display: flex;
  align-items: center;
  justify-content: center;
  font-size: 1.25rem;
  font-weight: 700;
  color: var(--tk-accent);
  flex-shrink: 0;
  font-family: var(--tk-font-display);
}

.profile-header-info {
  display: flex;
  flex-direction: column;
  gap: 0.15rem;
  min-width: 0;
}

.profile-title {
  font-size: 1.35rem;
  font-weight: 700;
  margin: 0;
  color: var(--tk-text);
  overflow: hidden;
  text-overflow: ellipsis;
  white-space: nowrap;
}

.profile-subtitle {
  font-size: 0.875rem;
  color: rgba(226, 228, 240, 0.55);
  margin: 0;
}

.profile-meta {
  font-size: 0.78rem;
  color: rgba(226, 228, 240, 0.4);
  margin: 0;
}

.profile-section {
  background: var(--tk-surface);
  border: 1px solid var(--tk-border);
  border-radius: 14px;
  overflow: hidden;
  box-shadow: inset 3px 0 0 var(--tk-accent);
}

.section-header {
  display: flex;
  align-items: center;
  gap: 0.6rem;
  padding: 0.875rem 1.25rem;
  border-bottom: 1px solid var(--tk-border);
  background: rgba(0, 229, 176, 0.05);
}

.section-icon {
  font-size: 0.875rem;
  color: var(--tk-accent);
}

.section-title {
  font-size: 0.7rem;
  font-weight: 600;
  letter-spacing: 0.1em;
  text-transform: uppercase;
  color: var(--tk-accent);
  margin: 0;
}

.section-body {
  padding: 1.25rem;
  display: flex;
  flex-direction: column;
  gap: 1.25rem;
}

.form-group {
  display: flex;
  flex-direction: column;
  gap: 0.4rem;
}

.form-label {
  font-size: 0.825rem;
  font-weight: 500;
  color: var(--tk-text);
}

.form-hint {
  font-size: 0.75rem;
  color: rgba(226, 228, 240, 0.45);
  margin: 0;
}

.profile-input {
  width: 100%;
}

:deep(.profile-password) {
  width: 100%;
}

:deep(.profile-password .p-password-input) {
  width: 100%;
}

.save-btn {
  align-self: flex-start;
}

.password-error {
  margin: 0;
}
</style>
