<script setup lang="ts">
  import type { Game } from '../types/game'
  import GameTableRow from './GameTableRow.vue'

  const props = withDefaults(
    defineProps<{
      games: Game[]
      showManageActions?: boolean
      launchingGameIds?: string[]
    }>(),
    {
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
</script>

<template>
  <div class="game-table" role="table" aria-label="游戏列表">
    <GameTableRow
      v-for="game in props.games"
      :key="game.id"
      :game="game"
      :show-manage-actions="props.showManageActions"
      :is-launching="props.launchingGameIds.includes(game.id)"
      @edit="emit('edit', $event)"
      @launch="emit('launch', $event)"
      @toggle-favorite="emit('toggleFavorite', $event)"
      @remove="emit('remove', $event)"
    />
  </div>
</template>

<style scoped>
  .game-table {
    min-width: 0;
  }
</style>
