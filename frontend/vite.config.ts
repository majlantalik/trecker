import { defineConfig } from 'vite'
import vue from '@vitejs/plugin-vue'
import { fileURLToPath, URL } from 'node:url'

export default defineConfig({
  plugins: [vue()],
  resolve: {
    alias: {
      '@': fileURLToPath(new URL('./src', import.meta.url))
    }
  },
  build: {
    // One stylesheet linked from index.html, instead of per-component chunks that Vite
    // injects at runtime with a generated <link>. Those injected stylesheets were not
    // taking effect inside the Tauri webview: HalfStarRating's rules never landed, so its
    // wrappers fell back to block layout and the star row rendered stacked vertically.
    //
    // Splitting CSS is a network optimisation and there is no network here; every asset
    // is inside the binary. One file is strictly better: nothing injected at runtime, no
    // ordering to get wrong, and no flash of unstyled content.
    cssCodeSplit: false
  },
  // The dev server no longer proxies anything. There is no API to reach.
  server: {},
  test: {
    environment: 'happy-dom',
    globals: true,
    setupFiles: ['./src/test-setup.ts']
  }
})
