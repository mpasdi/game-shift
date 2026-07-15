<script setup lang="ts">
  import { computed } from 'vue'
  import { Play } from '@lucide/vue'
  import IconButton from '../../../shared/components/IconButton.vue'
  import type { Game } from '../types/game'
  import { formatLastPlayTime } from '../utils/formatLastPlayTime'
  import GameArtwork from './GameArtwork.vue'
  import GameFavoriteToggle from './GameFavoriteToggle.vue'

  const props = withDefaults(
    defineProps<{
      game: Game
      isLaunching?: boolean
      showLastPlayTime?: boolean
    }>(),
    {
      isLaunching: false,
      showLastPlayTime: false
    }
  )

  const emit = defineEmits<{
    launch: [game: Game]
    toggleFavorite: [game: Game]
  }>()

  const lastPlayText = computed(() => formatLastPlayTime(props.game.lastPlayTime))
</script>

<template>
  <article class="game-quick-card" :class="{ 'game-quick-card--with-time': props.showLastPlayTime }">
    <GameArtwork :game="props.game" variant="quick" />
    <GameFavoriteToggle
      class="game-quick-card__favorite"
      :active="props.game.favorite"
      @toggle="emit('toggleFavorite', props.game)"
    />

    <div class="game-quick-card__content">
      <h2 :title="props.game.name">{{ props.game.name }}</h2>
      <p v-if="props.showLastPlayTime" :title="lastPlayText">{{ lastPlayText }}</p>
    </div>

    <div class="game-quick-card__actions">
      <IconButton
        class="game-quick-card__primary-action"
        label="启动游戏"
        variant="active"
        :disabled="props.isLaunching"
        @click="emit('launch', props.game)"
      >
        <Play :size="14" />
      </IconButton>
    </div>
  </article>
</template>

<style scoped>
  .game-quick-card {
    position: relative;
    min-width: 0;
    overflow: hidden;
    aspect-ratio: 2 / 3;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.18);
    transition:
      border-color 170ms ease,
      box-shadow 170ms ease,
      transform 170ms ease;
  }

  .game-quick-card::after {
    position: absolute;
    z-index: 1;
    inset: 35% 0 0;
    background: linear-gradient(180deg, rgba(11, 10, 15, 0), rgba(11, 10, 15, 0.72) 46%, rgba(11, 10, 15, 0.94));
    content: '';
    pointer-events: none;
  }

  .game-quick-card:hover {
    border-color: var(--border-strong);
    box-shadow: 0 16px 38px rgba(0, 0, 0, 0.24);
  }

  .game-quick-card:hover :deep(.game-artwork__cover) {
    transform: scale(1.025);
  }

  .game-quick-card :deep(.game-artwork--quick) {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    border: 0;
    border-radius: 0;
  }

  .game-quick-card :deep(.game-favorite-toggle) {
    top: 10px;
    right: 10px;
    width: 24px;
    height: 24px;
  }

  .game-quick-card :deep(.game-favorite-toggle svg) {
    width: 13px;
    height: 13px;
  }
  .game-quick-card__content {
    position: absolute;
    z-index: 2;
    right: 42px;
    bottom: 10px;
    left: 10px;
    display: grid;
    min-width: 0;
    gap: 4px;
  }

  .game-quick-card--with-time .game-quick-card__content {
    gap: 2px;
  }
  .game-quick-card__content h2 {
    overflow: hidden;
    margin: 0;
    color: var(--text);
    font-size: var(--font-size-md);
    line-height: 1.3;
    text-overflow: ellipsis;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.5);
    white-space: nowrap;
  }

  .game-quick-card--with-time .game-quick-card__content h2 {
    line-height: 1.15;
  }
  .game-quick-card__content p {
    overflow: hidden;
    margin: 0;
    color: rgba(246, 243, 248, 0.62);
    font-family: 'Microsoft YaHei UI', sans-serif;
    font-size: 11px;
    line-height: 1.15;
    text-overflow: ellipsis;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.42);
    white-space: nowrap;
  }
  .game-quick-card__actions :deep(.icon-button) {
    position: absolute;
    z-index: 2;
    right: 8px;
    bottom: 8px;
    width: 26px;
    min-width: 26px;
    height: 26px;
    min-height: 26px;
    border-radius: 8px;
  }

  .game-quick-card__actions :deep(.game-quick-card__primary-action) {
    border-color: var(--accent-border);
    background: rgba(13, 12, 17, 0.7);
    color: var(--accent-strong);
    backdrop-filter: blur(10px);
  }

  .game-quick-card__actions :deep(.game-quick-card__primary-action:hover) {
    border-color: transparent;
    background: linear-gradient(180deg, #8d73ff, #6d50e8);
    color: #ffffff;
    box-shadow: 0 8px 18px rgba(73, 51, 180, 0.28);
  }

  @media (prefers-reduced-motion: reduce) {
    .game-quick-card :deep(.game-artwork__cover) {
      transition: none;
    }

    .game-quick-card:hover :deep(.game-artwork__cover) {
      transform: none;
    }
  }
</style>
