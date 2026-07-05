<script setup lang="ts">
  import type { Game } from '../types/game'
  import GameGridCard from './GameGridCard.vue'
  import GameQuickCard from './GameQuickCard.vue'
  import GameTable from './GameTable.vue'

  const props = withDefaults(
    defineProps<{
      games: Game[]
      viewMode: 'grid' | 'list'
      actionMode?: 'full' | 'quick'
      showManageActions?: boolean
      launchingGameIds?: string[]
    }>(),
    {
      actionMode: 'full',
      showManageActions: true,
      launchingGameIds: () => []
    }
  )

  const emit = defineEmits<{
    edit: [game: Game]
    launch: [game: Game]
    toggleFavorite: [game: Game]
    remove: [game: Game]
  }>()

  function emitEdit(game: Game) {
    if (!props.showManageActions) return
    emit('edit', game)
  }

  function emitRemove(game: Game) {
    if (!props.showManageActions) return
    emit('remove', game)
  }
</script>

<template>
  <section class="game-area" :class="[props.viewMode, `game-area--actions-${props.actionMode}`]" aria-label="游戏列表">
    <template v-if="props.viewMode === 'grid' && props.actionMode === 'full'">
      <GameGridCard
        v-for="game in props.games"
        :key="game.id"
        :game="game"
        :show-manage-actions="props.showManageActions"
        :is-launching="props.launchingGameIds.includes(game.id)"
        @edit="emitEdit"
        @launch="emit('launch', $event)"
        @toggle-favorite="emit('toggleFavorite', $event)"
        @remove="emitRemove"
      />
    </template>

    <template v-else-if="props.viewMode === 'grid'">
      <GameQuickCard
        v-for="game in props.games"
        :key="game.id"
        :game="game"
        :is-launching="props.launchingGameIds.includes(game.id)"
        @launch="emit('launch', $event)"
        @toggle-favorite="emit('toggleFavorite', $event)"
      />
    </template>

    <GameTable
      v-else
      :games="props.games"
      :action-mode="props.showManageActions ? 'full' : 'quick'"
      :launching-game-ids="props.launchingGameIds"
      @edit="emitEdit"
      @launch="emit('launch', $event)"
      @toggle-favorite="emit('toggleFavorite', $event)"
      @remove="emitRemove"
    />
  </section>
</template>

<style scoped>
  .game-area {
    display: grid;
    gap: 12px;
  }

  .game-area.grid {
    grid-template-columns: repeat(auto-fill, 168px);
  }

  .game-area.grid.game-area--actions-full {
    grid-template-columns: repeat(auto-fill, 168px);
    align-items: start;
  }

  .game-area.list {
    grid-template-columns: minmax(0, 1fr);
    width: 100%;
    gap: 0;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.035);
  }

  .game-area.favorite-strip {
    display: grid;
    grid-auto-columns: 154px;
    grid-auto-flow: column;
    grid-template-columns: none;
    gap: 12px;
    overscroll-behavior-x: contain;
    overflow-x: auto;
    padding-right: 18px;
    scroll-padding-inline: 0 18px;
    scroll-snap-type: x proximity;
    scrollbar-width: none;
  }

  .game-area.favorite-strip::-webkit-scrollbar {
    display: none;
    width: 0;
    height: 0;
  }

  .game-area.favorite-strip > * {
    scroll-snap-align: start;
  }

  .game-area.home-favorite-grid {
    grid-template-columns: repeat(var(--favorite-columns, 4), minmax(0, 1fr));
    overflow: hidden;
  }

  .game-area.home-favorite-grid :deep(.game-quick-card) {
    min-height: 168px;
  }

  .game-area.home-favorite-grid :deep(.game-artwork--quick) {
    height: clamp(88px, 5vw, 104px);
  }

  @media (max-width: 720px) {
    .game-area.grid {
      grid-template-columns: minmax(0, 1fr);
    }

    .game-area.home-favorite-grid {
      grid-template-columns: repeat(var(--favorite-columns, 2), minmax(0, 1fr));
    }
  }
</style>
