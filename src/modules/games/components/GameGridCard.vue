<script setup lang="ts">
  import { computed } from 'vue'
  import { Pencil, Play, Trash2 } from '@lucide/vue'
  import IconButton from '../../../shared/components/IconButton.vue'
  import type { Game } from '../types/game'
  import { formatLastPlayTime } from '../utils/formatLastPlayTime'
  import GameArtwork from './GameArtwork.vue'
  import GameFavoriteToggle from './GameFavoriteToggle.vue'

  const props = withDefaults(
    defineProps<{
      game: Game
      showManageActions?: boolean
      isLaunching?: boolean
    }>(),
    {
      showManageActions: true,
      isLaunching: false
    }
  )

  const emit = defineEmits<{
    edit: [game: Game]
    launch: [game: Game]
    toggleFavorite: [game: Game]
    remove: [game: Game]
  }>()

  const lastPlayText = computed(() => formatLastPlayTime(props.game.lastPlayTime))

  const metaText = computed(() => `${lastPlayText.value} · ${props.game.playCount} 次`)
</script>

<template>
  <article class="game-grid-card">
    <div class="game-grid-card__media">
      <GameArtwork :game="props.game" variant="grid" />
      <GameFavoriteToggle :active="props.game.favorite" @toggle="emit('toggleFavorite', props.game)" />

      <div class="game-grid-card__overlay">
        <div class="game-grid-card__content">
          <h2 :title="props.game.name">{{ props.game.name }}</h2>
          <p :title="metaText">{{ metaText }}</p>
        </div>

        <div class="game-grid-card__actions">
          <IconButton
            class="game-grid-card__primary-action"
            label="启动游戏"
            variant="active"
            :disabled="props.isLaunching"
            @click="emit('launch', props.game)"
          >
            <Play :size="15" />
          </IconButton>
          <IconButton v-if="props.showManageActions" label="编辑游戏" @click="emit('edit', props.game)">
            <Pencil :size="16" />
          </IconButton>
          <IconButton
            v-if="props.showManageActions"
            label="移除游戏"
            variant="danger"
            @click="emit('remove', props.game)"
          >
            <Trash2 :size="16" />
          </IconButton>
        </div>
      </div>
    </div>
  </article>
</template>

<style scoped>
  .game-grid-card {
    min-width: 0;
    border: 1px solid var(--border);
    border-radius: 8px;
    overflow: hidden;
    background: var(--surface);
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.18);
    transition:
      border-color 170ms ease,
      background 170ms ease,
      box-shadow 170ms ease,
      transform 170ms ease;
  }

  .game-grid-card:hover {
    border-color: var(--border-strong);
    background: var(--surface-hover);
    box-shadow: 0 16px 38px rgba(0, 0, 0, 0.24);
  }

  .game-grid-card__media {
    position: relative;
    overflow: hidden;
    border-radius: inherit;
    aspect-ratio: 2 / 3;
  }

  .game-grid-card__overlay {
    position: absolute;
    right: 0;
    bottom: 0;
    left: 0;
    display: grid;
    gap: 10px;
    padding: 56px 10px 10px;
    background: linear-gradient(
      180deg,
      rgba(11, 10, 15, 0) 0%,
      rgba(11, 10, 15, 0.58) 42%,
      rgba(11, 10, 15, 0.92) 100%
    );
  }

  .game-grid-card__content {
    display: grid;
    min-width: 0;
    gap: 5px;
  }

  .game-grid-card__content h2 {
    display: -webkit-box;
    overflow: hidden;
    overflow-wrap: anywhere;
    margin: 0;
    color: var(--text);
    font-size: var(--font-size-md);
    line-height: 1.3;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.44);
    white-space: normal;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
    max-height: 36px;
  }

  .game-grid-card__content p {
    overflow: hidden;
    margin: 0;
    color: rgba(246, 243, 248, 0.72);
    font-size: var(--font-size-xs);
    line-height: 1.35;
    text-overflow: ellipsis;
    text-shadow: 0 2px 8px rgba(0, 0, 0, 0.34);
    white-space: nowrap;
  }

  .game-grid-card__actions {
    display: flex;
    justify-content: space-between;
    gap: 0;
    align-items: center;
  }

  .game-grid-card__actions :deep(.icon-button) {
    width: 30px;
    min-width: 30px;
    height: 30px;
    background: rgba(13, 12, 17, 0.58);
    backdrop-filter: blur(10px);
  }

  .game-grid-card__actions :deep(.game-grid-card__primary-action) {
    border-color: var(--accent-border);
    background: rgba(13, 12, 17, 0.72);
    color: var(--accent-strong);
  }

  .game-grid-card__actions :deep(.game-grid-card__primary-action:hover) {
    border-color: transparent;
    background: linear-gradient(180deg, #8d73ff, #6d50e8);
    color: #ffffff;
    box-shadow: 0 8px 18px rgba(73, 51, 180, 0.28);
  }
</style>
