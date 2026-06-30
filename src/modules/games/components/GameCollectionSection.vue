<script setup lang="ts">
  import type { Component } from 'vue'
  import { Grid2X2, LayoutList, Search } from '@lucide/vue'
  import EmptyState from '../../../shared/components/EmptyState.vue'
  import IconButton from '../../../shared/components/IconButton.vue'
  import GameList from './GameList.vue'
  import type { Game } from '../types/game'
  import type { GameViewMode } from '../composables/useGameLibraryActions'

  const props = withDefaults(
    defineProps<{
      title: string
      icon: Component
      meta: string
      games: Game[]
      viewMode: GameViewMode
      actionMode?: 'full' | 'quick'
      showManageActions?: boolean
      launchingGameIds?: string[]
      emptyTitle: string
      emptyDescription: string
      listClass?: string | Record<string, boolean>
    }>(),
    {
      actionMode: 'full',
      showManageActions: true,
      launchingGameIds: () => [],
      listClass: ''
    }
  )

  const emit = defineEmits<{
    updateViewMode: [viewMode: GameViewMode]
    edit: [game: Game]
    launch: [game: Game]
    toggleFavorite: [game: Game]
    remove: [game: Game]
  }>()
</script>

<template>
  <section class="library-section">
    <div class="section-heading">
      <h2 class="section-title">
        <component :is="props.icon" :size="14" />
        <span>{{ props.title }}</span>
      </h2>
      <div class="section-actions">
        <span>{{ props.meta }}</span>
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

    <EmptyState
      v-if="props.games.length === 0"
      label="没有匹配的游戏"
      eyebrow="无结果"
      :title="props.emptyTitle"
      :description="props.emptyDescription"
    >
      <template #icon><Search :size="15" /></template>
    </EmptyState>

    <GameList
      v-else
      :class="props.listClass"
      :games="props.games"
      :view-mode="props.viewMode"
      :action-mode="props.actionMode"
      :show-manage-actions="props.showManageActions"
      :launching-game-ids="props.launchingGameIds"
      @edit="emit('edit', $event)"
      @launch="emit('launch', $event)"
      @toggle-favorite="emit('toggleFavorite', $event)"
      @remove="emit('remove', $event)"
    />
  </section>
</template>

<style scoped>
  .library-section {
    display: grid;
    gap: 14px;
    animation: section-in 180ms ease-out both;
  }

  .section-heading {
    display: flex;
    gap: 14px;
    align-items: center;
    justify-content: space-between;
  }

  .section-title {
    display: inline-flex;
    gap: 7px;
    align-items: center;
    margin: 0;
    color: var(--text);
    font-size: var(--font-size-md);
    font-weight: 700;
    line-height: 1.2;
  }

  .section-title svg {
    width: 15px;
    height: 15px;
    color: var(--accent-strong);
  }

  .section-actions {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .section-actions > span {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
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
    .section-heading {
      align-items: stretch;
      flex-direction: column;
    }

    .section-actions {
      width: 100%;
    }
  }
</style>
