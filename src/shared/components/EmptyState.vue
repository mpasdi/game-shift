<script setup lang="ts">
  withDefaults(
    defineProps<{
      eyebrow: string
      title: string
      description: string
      label: string
      variant?: 'compact' | 'panel' | 'plain'
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

  .empty-state--plain {
    grid-template-columns: minmax(0, 1fr);
    grid-auto-rows: max-content;
    align-content: center;
    justify-items: center;
    min-height: clamp(280px, calc(100vh - 190px), 900px);
    border: 0;
    background: transparent;
    padding: 36px 24px;
    text-align: center;
    translate: 0 -2.5vh;
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

  .empty-state--plain .empty-state__icon {
    width: 52px;
    height: 52px;
    border-color: rgba(157, 140, 255, 0.24);
    border-radius: 14px;
    background: rgba(124, 92, 255, 0.1);
    box-shadow: 0 16px 40px rgba(57, 38, 150, 0.16);
  }

  .empty-state--plain .empty-state__icon :deep(svg) {
    width: 22px;
    height: 22px;
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

  .empty-state--plain .empty-copy {
    gap: 8px;
  }

  .empty-state--plain .empty-copy h2 {
    font-size: clamp(20px, 2vw, 26px);
  }

  .empty-state--plain .empty-copy p {
    max-width: 460px;
  }

  .empty-state--plain .eyebrow {
    color: var(--accent-strong);
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

  .empty-state--plain .empty-actions {
    grid-column: auto;
    justify-content: center;
    margin-top: 6px;
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
    .empty-state--panel,
    .empty-state--plain {
      grid-template-columns: minmax(0, 1fr);
      justify-items: center;
      text-align: center;
    }

    .empty-actions {
      grid-column: auto;
      justify-content: center;
    }

    .empty-state--plain {
      min-height: 280px;
      padding: 28px 16px;
    }
  }
</style>
