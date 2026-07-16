<script setup lang="ts">
  import type { Component } from 'vue'
  import { Grid2X2, LayoutList } from '@lucide/vue'
  import IconButton from '../../../shared/components/IconButton.vue'
  import type { GameViewMode } from '../composables/useGameLibraryActions'

  const props = defineProps<{
    title: string
    icon: Component
    meta: string
    viewMode: GameViewMode
  }>()

  const emit = defineEmits<{
    updateViewMode: [viewMode: GameViewMode]
  }>()
</script>

<template>
  <section class="game-collection-layout">
    <div class="game-collection-layout__heading">
      <h2 class="game-collection-layout__title">
        <component :is="props.icon" :size="14" />
        <span>{{ props.title }}</span>
        <span class="game-collection-layout__meta">{{ props.meta }}</span>
      </h2>

      <div class="game-collection-layout__actions">
        <div class="segmented" aria-label="视图切换">
          <IconButton
            label="网格视图"
            :variant="props.viewMode === 'grid' ? 'active' : 'plain'"
            @click="emit('updateViewMode', 'grid')"
          >
            <Grid2X2 :size="17" />
          </IconButton>
          <IconButton
            label="列表视图"
            :variant="props.viewMode === 'list' ? 'active' : 'plain'"
            @click="emit('updateViewMode', 'list')"
          >
            <LayoutList :size="17" />
          </IconButton>
        </div>
      </div>
    </div>

    <slot />
  </section>
</template>

<style scoped>
  .game-collection-layout {
    display: grid;
    gap: 14px;
    animation: section-in 180ms ease-out both;
  }

  .game-collection-layout__heading {
    display: flex;
    gap: 14px;
    align-items: center;
    justify-content: space-between;
  }

  .game-collection-layout__title {
    display: inline-flex;
    gap: 7px;
    align-items: center;
    margin: 0;
    color: var(--text);
    font-size: var(--font-size-md);
    font-weight: 700;
    line-height: 1.2;
  }

  .game-collection-layout__title svg {
    width: 15px;
    height: 15px;
    color: var(--accent-strong);
  }

  .game-collection-layout__meta {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    font-weight: 600;
  }

  .game-collection-layout__actions {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .segmented {
    display: inline-flex;
    gap: 6px;
    height: 36px;
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
    .game-collection-layout__heading {
      align-items: stretch;
      flex-direction: column;
    }

    .game-collection-layout__actions {
      width: 100%;
    }
  }
</style>
