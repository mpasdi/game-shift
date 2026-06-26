<script setup lang="ts">
  import GameCard from './GameCard.vue'
  import type { Game } from '../types/game'

  const props = withDefaults(
    defineProps<{
      games: Game[]
      viewMode: 'grid' | 'list'
      actionMode?: 'full' | 'quick'
    }>(),
    {
      actionMode: 'full'
    }
  )

  const emit = defineEmits<{
    edit: [game: Game]
    toggleFavorite: [game: Game]
    remove: [game: Game]
  }>()
</script>

<template>
  <section class="game-area" :class="[props.viewMode, `game-area--actions-${props.actionMode}`]" aria-label="游戏列表">
    <GameCard
      v-for="game in props.games"
      :key="game.id"
      :game="game"
      :view-mode="props.viewMode"
      :action-mode="props.actionMode"
      @edit="emit('edit', $event)"
      @toggle-favorite="emit('toggleFavorite', $event)"
      @remove="emit('remove', $event)"
    />
  </section>
</template>

<style scoped>
  .game-area {
    display: grid;
    gap: 12px;
  }

  .game-area.grid {
    grid-template-columns: repeat(auto-fill, minmax(220px, 1fr));
  }

  .game-area.list {
    grid-template-columns: minmax(0, 1fr);
    gap: 0;
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.045);
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
</style>
