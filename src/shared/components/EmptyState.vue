<script setup lang="ts">
  defineProps<{
    eyebrow: string
    title: string
    description: string
    label: string
  }>()
</script>

<template>
  <section class="empty-state" :aria-label="label">
    <div class="empty-copy">
      <p class="eyebrow">
        <span v-if="$slots.icon" class="eyebrow__icon"><slot name="icon" /></span>
        {{ eyebrow }}
      </p>
      <h2>{{ title }}</h2>
      <p>{{ description }}</p>
    </div>
    <div v-if="$slots.actions" class="empty-actions">
      <slot name="actions" />
    </div>
  </section>
</template>
