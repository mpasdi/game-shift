<script setup lang="ts">
  import { computed } from 'vue'
  import { Play } from '@lucide/vue'
  import IconButton from '../../../shared/components/IconButton.vue'
  import type { Game } from '../types/game'
  import GameArtwork from './GameArtwork.vue'
  import GameFavoriteToggle from './GameFavoriteToggle.vue'

  const props = withDefaults(
    defineProps<{
      game: Game
      isLaunching?: boolean
    }>(),
    {
      isLaunching: false
    }
  )

  const emit = defineEmits<{
    launch: [game: Game]
    toggleFavorite: [game: Game]
  }>()

  const exeFileName = computed(() => props.game.exePath.split(/[\\/]/).pop() ?? props.game.exePath)
</script>

<template>
  <article class="game-quick-card">
    <GameArtwork :game="props.game" variant="quick" />
    <GameFavoriteToggle
      class="game-quick-card__favorite"
      :active="props.game.favorite"
      @toggle="emit('toggleFavorite', props.game)"
    />

    <div class="game-quick-card__content">
      <h2 :title="props.game.name">{{ props.game.name }}</h2>
      <p :title="props.game.exePath">{{ exeFileName }}</p>
    </div>

    <div class="game-quick-card__actions">
      <IconButton
        class="game-quick-card__primary-action"
        label="启动游戏"
        variant="active"
        :disabled="props.isLaunching"
        @click="emit('launch', props.game)"
      >
        <Play :size="15" />
      </IconButton>
    </div>
  </article>
</template>

<style scoped>
  .game-quick-card {
    position: relative;
    display: grid;
    grid-template-columns: 1fr;
    gap: 9px;
    min-height: 190px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    padding: 10px;
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.18);
    transition:
      border-color 170ms ease,
      background 170ms ease,
      box-shadow 170ms ease;
  }

  .game-quick-card:hover {
    border-color: var(--border-strong);
    background: var(--surface-hover);
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.2);
  }

  .game-quick-card__content {
    display: grid;
    min-width: 0;
    gap: 3px;
    padding-right: 34px;
  }

  .game-quick-card__content h2 {
    display: -webkit-box;
    overflow: hidden;
    overflow-wrap: anywhere;
    margin: 0;
    color: var(--text);
    font-size: var(--font-size-md);
    line-height: 1.3;
    white-space: normal;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    max-height: 36px;
  }

  .game-quick-card__content p {
    display: flex;
    overflow: hidden;
    align-items: center;
    min-height: 28px;
    margin: 0;
    color: var(--text-muted);
    font-size: var(--font-size-xs);
    line-height: 1.3;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .game-quick-card__actions {
    display: contents;
  }

  .game-quick-card__actions :deep(.icon-button) {
    position: absolute;
    right: 8px;
    bottom: 8px;
    width: 28px;
    min-width: 28px;
    height: 28px;
    min-height: 28px;
    border-radius: 8px;
  }

  .game-quick-card__actions :deep(.game-quick-card__primary-action) {
    border-color: var(--accent-border);
    background: rgba(13, 12, 17, 0.64);
    color: var(--accent-strong);
    backdrop-filter: blur(10px);
  }

  .game-quick-card__actions :deep(.game-quick-card__primary-action:hover) {
    background: rgba(33, 29, 47, 0.82);
  }

  @media (max-width: 720px) {
    .game-quick-card {
      grid-template-columns: 48px minmax(0, 1fr);
    }

    .game-quick-card__actions {
      display: flex;
      flex-wrap: wrap;
    }
  }
</style>
