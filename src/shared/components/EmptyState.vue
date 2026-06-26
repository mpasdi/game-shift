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

<style scoped>
  .empty-state {
    display: grid;
    gap: 18px;
    place-items: center;
    min-height: 300px;
    border: 1px dashed var(--border-strong);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.035);
    padding: 42px 24px;
    text-align: center;
    animation: section-in 180ms ease-out both;
  }

  .empty-copy {
    display: grid;
    gap: 6px;
    max-width: 520px;
  }

  .empty-copy h2 {
    margin: 0;
    color: var(--text);
    font-size: 24px;
    line-height: 1.18;
  }

  .empty-copy p {
    margin: 0;
    color: var(--text-muted);
    line-height: 1.65;
  }

  .eyebrow {
    display: inline-flex;
    gap: 7px;
    align-items: center;
    margin: 0 0 7px;
    color: var(--text-subtle);
    font-size: 12px;
    font-weight: 720;
    letter-spacing: 0;
  }

  .empty-actions {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  @keyframes section-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }

    to {
      opacity: 1;
      transform: translateY(0);
    }
  }
</style>
