<template>
  <div class="view-toggle">
    <Button
      icon="pi pi-list"
      :outlined="modelValue !== 'table'"
      :severity="modelValue === 'table' ? 'primary' : 'secondary'"
      size="small"
      @click="$emit('update:modelValue', 'table')"
      aria-label="Table view"
    />
    <Button
      icon="pi pi-th-large"
      :outlined="modelValue !== 'grid'"
      :severity="modelValue === 'grid' ? 'primary' : 'secondary'"
      size="small"
      @click="$emit('update:modelValue', 'grid')"
      aria-label="Grid view"
    />
  </div>
</template>

<script setup lang="ts">
import Button from 'primevue/button'

defineProps<{ modelValue: 'table' | 'grid' }>()
defineEmits<{ 'update:modelValue': [value: 'table' | 'grid'] }>()
</script>

<style scoped>
.view-toggle {
  display: flex;
  gap: 0.25rem;
}
</style>
