import { createApp } from 'vue'
import { createPinia } from 'pinia'
import PrimeVue from 'primevue/config'
import Aura from '@primevue/themes/aura'
import ToastService from 'primevue/toastservice'
import ConfirmationService from 'primevue/confirmationservice'
import 'primeicons/primeicons.css'

import router from './router'
import App from './App.vue'
import { useAuthStore } from './stores/auth'

const app = createApp(App)
const pinia = createPinia()

app.use(pinia)
app.use(PrimeVue, {
  theme: {
    preset: Aura,
    options: {
      darkModeSelector: 'system',
      cssLayer: false
    }
  }
})
app.use(ToastService)
app.use(ConfirmationService)

// Resolve auth state before installing router so the initial navigation guard sees correct isAuthenticated
const authStore = useAuthStore()
authStore.fetchMe().finally(() => {
  app.use(router)
  app.mount('#app')
})
