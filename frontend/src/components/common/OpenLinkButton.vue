<template>
  <!-- Always rendered, so every row's actions line up; disabled when there is no link.
       `.stop`, or a library row's own click handler opens the album on top of the link. -->
  <Button
    icon="pi pi-external-link"
    size="small"
    severity="secondary"
    text
    :disabled="!choice"
    v-tooltip.bottom="choice?.label"
    :aria-label="choice?.label ?? 'No streaming link'"
    @click.stop="choice && open(choice)"
  />
</template>

<script setup lang="ts">
import { computed } from 'vue'
import Button from 'primevue/button'
import { useOpenLink } from '@/composables/useOpenLink'

const props = defineProps<{ links: Record<string, string> | null | undefined }>()

const { choose, open } = useOpenLink()
const choice = computed(() => choose(props.links))
</script>
