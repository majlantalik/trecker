import { createApp } from 'vue'
import { createPinia } from 'pinia'
import PrimeVue from 'primevue/config'
import Aura from '@primevue/themes/aura'
import ToastService from 'primevue/toastservice'
import ConfirmationService from 'primevue/confirmationservice'
import 'primeicons/primeicons.css'

// Fonts are bundled, not fetched from Google at runtime. A local-first app must render
// correctly offline, and the alternative would mean opening up the Tauri CSP.
// Weights must stay in sync with --tk-font-display / --tk-font-body in App.vue.
import '@fontsource/syne/600.css'
import '@fontsource/syne/700.css'
import '@fontsource/syne/800.css'
import '@fontsource/outfit/300.css'
import '@fontsource/outfit/400.css'
import '@fontsource/outfit/500.css'
import '@fontsource/outfit/600.css'

import router from './router'
import App from './App.vue'

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

// Nothing to resolve before mounting any more: there is no session to fetch and no
// navigation guard to satisfy. The data lives on this machine.
app.use(router)
app.mount('#app')
