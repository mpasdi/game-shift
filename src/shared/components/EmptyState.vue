<script setup lang="ts">
  withDefaults(
    defineProps<{
      eyebrow: string
      title: string
      description: string
      label: string
      variant?: 'compact' | 'panel'
    }>(),
    {
      variant: 'compact'
    }
  )
</script>

<template>
  <section class="empty-state" :class="`empty-state--${variant}`" :aria-label="label">
    <div v-if="$slots.icon" class="empty-state__icon" aria-hidden="true">
      <slot name="icon" />
    </div>

    <div class="empty-copy">
      <p class="eyebrow">{{ eyebrow }}</p>
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
    grid-template-columns: auto minmax(0, 1fr);
    gap: 14px;
    align-items: center;
    border: 1px solid rgba(255, 255, 255, 0.075);
    border-radius: 8px;
    background:
      linear-gradient(180deg, rgba(255, 255, 255, 0.035), rgba(255, 255, 255, 0.018)), rgba(255, 255, 255, 0.018);
    padding: 18px;
    animation: section-in 180ms ease-out both;
  }

  .empty-state--panel {
    grid-template-columns: minmax(0, 1fr);
    justify-items: center;
    min-height: 220px;
    padding: 34px 24px;
    text-align: center;
  }

  .empty-state__icon {
    display: grid;
    width: 36px;
    height: 36px;
    place-items: center;
    border: 1px solid var(--accent-border);
    border-radius: 8px;
    background: rgba(124, 92, 255, 0.12);
    color: var(--accent-strong);
  }

  .empty-state--panel .empty-state__icon {
    width: 42px;
    height: 42px;
  }

  .empty-copy {
    display: grid;
    gap: 5px;
    min-width: 0;
    max-width: 520px;
  }

  .empty-copy h2 {
    margin: 0;
    color: var(--text);
    font-size: var(--font-size-lg);
    line-height: 1.25;
  }

  .empty-state--panel .empty-copy h2 {
    font-size: 22px;
  }

  .empty-copy p {
    margin: 0;
    color: var(--text-muted);
    font-size: var(--font-size-md);
    line-height: 1.55;
  }

  .eyebrow {
    color: var(--text-subtle);
    font-size: var(--font-size-xs);
    font-weight: 700;
    letter-spacing: 0;
  }

  .empty-actions {
    grid-column: 2;
    display: flex;
    flex-wrap: wrap;
    gap: 10px;
    align-items: center;
    margin-top: 2px;
  }

  .empty-state--panel .empty-actions {
    grid-column: auto;
    justify-content: center;
    margin-top: 4px;
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

  @media (max-width: 720px) {
    .empty-state,
    .empty-state--panel {
      grid-template-columns: minmax(0, 1fr);
      justify-items: center;
      text-align: center;
    }

    .empty-actions {
      grid-column: auto;
      justify-content: center;
    }
  }
</style>
