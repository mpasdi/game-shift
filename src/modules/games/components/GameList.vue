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
  <section class="game-area" :class="props.viewMode" aria-label="游戏列表">
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
