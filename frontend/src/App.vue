<template>
  <AppLayout>
    <RouterView />
  </AppLayout>
  <Toast position="bottom-right" />
</template>

<script setup lang="ts">
import { RouterView } from 'vue-router'
import Toast from 'primevue/toast'
import AppLayout from '@/components/layout/AppLayout.vue'
</script>

<style>
:root {
  --tk-bg: #0d0f1a;
  --tk-sidebar-bg: #09090f;
  --tk-header-bg: rgba(13, 15, 26, 0.88);
  --tk-surface: #13162a;
  --tk-surface-hover: #1a1e35;
  --tk-border: rgba(255, 255, 255, 0.07);
  --tk-border-hover: rgba(255, 255, 255, 0.14);
  --tk-text: #e2e4f0;
  --tk-text-muted: #5a6080;
  --tk-accent: #00e5b0;
  --tk-accent-hover: #00ffc8;
  --tk-accent-dim: rgba(0, 229, 176, 0.12);
  --tk-accent-glow: rgba(0, 229, 176, 0.28);
  --tk-font-display: 'Syne', ui-sans-serif, system-ui, sans-serif;
  --tk-font-body: 'Outfit', ui-sans-serif, system-ui, sans-serif;
}

*, *::before, *::after {
  box-sizing: border-box;
}

html, body {
  margin: 0;
  padding: 0;
  background: var(--tk-bg);
}

body {
  font-family: var(--tk-font-body);
  font-size: 18px;
  color: var(--tk-text);
  -webkit-font-smoothing: antialiased;
  -moz-osx-font-smoothing: grayscale;
}

h1, h2, h3, h4 {
  font-family: var(--tk-font-display);
  letter-spacing: -0.025em;
  margin: 0;
}

::-webkit-scrollbar {
  width: 5px;
  height: 5px;
}
::-webkit-scrollbar-track {
  background: transparent;
}
::-webkit-scrollbar-thumb {
  background: rgba(255, 255, 255, 0.1);
  border-radius: 4px;
}
::-webkit-scrollbar-thumb:hover {
  background: rgba(0, 229, 176, 0.25);
}

/*
 * The app's dialog look: the panel surface, a teal left edge, the display font for the
 * title. Opt in with :pt="{ root: { class: 'tk-dialog' } }".
 *
 * Written as PrimeVue's own CSS variables, not as plain declarations. PrimeVue injects its
 * theme stylesheet at runtime, after this one, so a one-class rule here such as
 * `background: ...` loses to its `.p-dialog` rule and silently never applies. Its rules
 * read variables declared on :root, and a variable declared on the dialog itself wins for
 * the dialog and everything inside it, whatever order the stylesheets load in.
 */
.tk-dialog {
  --p-dialog-background: var(--tk-surface);
  --p-dialog-border-color: var(--tk-border-hover);
  --p-dialog-color: var(--tk-text);
  --p-dialog-border-radius: 14px;
  --p-dialog-shadow: 0 24px 60px rgba(0, 0, 0, 0.55), inset 3px 0 0 var(--tk-accent);
  --p-dialog-title-font-size: 1.1rem;
  --p-dialog-title-font-weight: 700;

  --p-text-muted-color: rgba(226, 228, 240, 0.55);
  --p-divider-border-color: var(--tk-border);

  /* Lists inside a dialog sit on the panel rather than on a darker form field. */
  --p-listbox-background: transparent;
  --p-listbox-border-color: var(--tk-border);
  --p-listbox-border-radius: 10px;
  --p-listbox-color: var(--tk-text);
  --p-listbox-option-color: var(--tk-text);
  --p-listbox-option-border-radius: 8px;
  --p-listbox-option-focus-background: var(--tk-surface-hover);
  --p-listbox-option-focus-color: var(--tk-text);
  --p-listbox-option-selected-background: var(--tk-accent-dim);
  --p-listbox-option-selected-color: var(--tk-accent);
  --p-listbox-option-selected-focus-background: var(--tk-accent-dim);
  --p-listbox-option-selected-focus-color: var(--tk-accent);
}

.tk-dialog .p-dialog-title {
  font-family: var(--tk-font-display);
  letter-spacing: -0.02em;
}

/* Text areas grow downwards only. Dragging one wider pushes past the dialog or panel it sits
   in, and PrimeVue's theme leaves the browser default, which allows both. */
textarea {
  resize: vertical;
}

/*
 * Suggestion lists that PrimeVue appends to <body>, outside any themed ancestor, so they take
 * their colours from variables on themselves. Opt in with :pt="{ overlay: { class: 'tk-overlay' } }".
 */
.tk-overlay {
  --p-autocomplete-overlay-background: var(--tk-surface);
  --p-autocomplete-overlay-border-color: var(--tk-border-hover);
  --p-autocomplete-overlay-color: var(--tk-text);
  --p-autocomplete-overlay-border-radius: 10px;
  --p-autocomplete-overlay-shadow: 0 12px 32px rgba(0, 0, 0, 0.5);
  --p-autocomplete-option-color: var(--tk-text);
  --p-autocomplete-option-focus-background: var(--tk-surface-hover);
  --p-autocomplete-option-focus-color: var(--tk-text);
  --p-autocomplete-option-selected-background: var(--tk-accent-dim);
  --p-autocomplete-option-selected-color: var(--tk-accent);
  --p-autocomplete-option-selected-focus-background: var(--tk-accent-dim);
  --p-autocomplete-option-selected-focus-color: var(--tk-accent);
  --p-autocomplete-option-border-radius: 6px;
}

/*
 * Frosted glass behind every modal: the page stays visible but blurred and dimmed, so the
 * dialog reads as a layer above it rather than a hole punched through it.
 *
 * `--px-mask-background` is PrimeVue's own override for the mask colour, which its
 * fade-in animation also uses. The theme sets no backdrop-filter, so that part needs no
 * variable. WebKitGTK supports it unprefixed; the prefix is for older WebKit on macOS.
 */
.p-overlay-mask.p-dialog-mask {
  --px-mask-background: rgba(8, 9, 16, 0.4);
  backdrop-filter: blur(10px) saturate(130%);
  -webkit-backdrop-filter: blur(10px) saturate(130%);
}

/*
 * The blur has to fade with the tint. PrimeVue animates only the mask's colour, so on
 * closing the tint faded while the blur stayed at full strength, then vanished at once
 * when the mask was removed.
 *
 * The fade is on the mask's opacity, never on the blur itself. Animating backdrop-filter
 * makes WebKitGTK's compositor blur the whole layer, dialog included, instead of only what
 * is behind it. Fading the mask also fades the dialog inside it, which is already fading
 * on its own, so the two move together.
 *
 * `animation` is a single property, so these rules repeat PrimeVue's own colour fade next
 * to the opacity fade rather than replacing it. Two classes, to outrank its one-class
 * rules, which load later. While closing the mask carries both classes, and the leave
 * rule wins by coming second.
 */
.p-dialog-mask.p-overlay-mask-enter-active {
  animation:
    p-animate-overlay-mask-enter var(--p-mask-transition-duration, 0.3s) forwards,
    tk-mask-fade-in var(--p-mask-transition-duration, 0.3s) forwards;
}

.p-dialog-mask.p-overlay-mask-leave-active {
  animation:
    p-animate-overlay-mask-leave var(--p-mask-transition-duration, 0.3s) forwards,
    tk-mask-fade-out var(--p-mask-transition-duration, 0.3s) forwards;
}

@keyframes tk-mask-fade-in {
  from {
    opacity: 0;
  }
}

@keyframes tk-mask-fade-out {
  to {
    opacity: 0;
  }
}

/* A form field's label, in the uppercase style of the library filters' labels. Brighter
   than theirs, which use --tk-text-muted and are too faint to read on a dialog panel. */
.tk-label {
  font-size: 0.72rem;
  font-weight: 600;
  letter-spacing: 0.07em;
  text-transform: uppercase;
  color: rgba(226, 228, 240, 0.55);
}

/* Keyboard shortcut hints. Wrapped in :where() so it adds no specificity, and a dialog's
   own scoped kbd style, such as the shortcuts help's larger keys, still wins. */
:where(.tk-dialog) kbd {
  font-family: inherit;
  font-size: 0.68rem;
  padding: 0.1rem 0.35rem;
  border-radius: 4px;
  border: 1px solid var(--tk-border);
  background: rgba(255, 255, 255, 0.04);
  color: rgba(226, 228, 240, 0.7);
}
</style>
