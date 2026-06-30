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
      :show-manage-actions="props.showManageActions"
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
    overflow-x: auto;
    padding-bottom: 4px;
    scrollbar-color: rgba(255, 255, 255, 0.2) transparent;
  }

  .game-area.favorite-grid {
    grid-template-columns: repeat(auto-fill, 154px);
  }

  @media (max-width: 720px) {
    .game-area.grid {
      grid-template-columns: minmax(0, 1fr);
    }
  }
</style>
