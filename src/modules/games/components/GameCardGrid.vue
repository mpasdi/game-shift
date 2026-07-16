<script setup lang="ts">
  const props = defineProps<{
    variant: 'full' | 'quick'
    ariaLabel?: string
  }>()
</script>

<template>
  <section
    class="game-card-grid"
    :class="`game-card-grid--${props.variant}`"
    :aria-label="props.ariaLabel ?? '游戏卡片列表'"
  >
    <slot />
  </section>
</template>

<style scoped>
  .game-card-grid {
    display: grid;
    gap: 12px;
  }

  .game-card-grid--full {
    grid-template-columns: repeat(auto-fill, 168px);
    align-items: start;
  }

  .game-card-grid--quick {
    grid-template-columns: var(--game-grid-columns, repeat(auto-fill, 128px));
  }

  @media (max-width: 720px) {
    .game-card-grid--full {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
