<script setup lang="ts">
  import { computed } from 'vue'
  import { Pencil, Play, Trash2 } from '@lucide/vue'
  import IconButton from '../../../shared/components/IconButton.vue'
  import type { Game } from '../types/game'
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

  const lastPlayText = computed(() => {
    if (!props.game.lastPlayTime) return '无启动记录'
    return new Intl.DateTimeFormat('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    }).format(new Date(props.game.lastPlayTime))
  })

  const metaText = computed(() => `${lastPlayText.value} · ${props.game.playCount} 次`)
</script>

<template>
  <article class="game-grid-card">
    <GameArtwork :game="props.game" variant="grid" />
    <GameFavoriteToggle :active="props.game.favorite" @toggle="emit('toggleFavorite', props.game)" />

    <div class="game-grid-card__content">
      <h2>{{ props.game.name }}</h2>
      <p>{{ metaText }}</p>
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
      <IconButton v-if="props.showManageActions" label="移除游戏" variant="danger" @click="emit('remove', props.game)">
        <Trash2 :size="16" />
      </IconButton>
    </div>
  </article>
</template>

<style scoped>
  .game-grid-card {
    position: relative;
    display: grid;
    grid-template-columns: minmax(0, 1fr);
    gap: 10px;
    min-height: 300px;
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

  .game-grid-card:hover {
    border-color: var(--border-strong);
    background: var(--surface-hover);
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.2);
  }

  .game-grid-card__content {
    display: grid;
    min-width: 0;
    gap: 5px;
  }

  .game-grid-card__content h2 {
    display: -webkit-box;
    min-height: 34px;
    overflow: hidden;
    overflow-wrap: anywhere;
    margin: 0;
    color: var(--text);
    font-size: 13px;
    line-height: 1.3;
    white-space: normal;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
  }

  .game-grid-card__content p {
    overflow: hidden;
    margin: 0;
    color: var(--text-muted);
    font-size: var(--font-size-xs);
    line-height: 1.35;
    text-overflow: ellipsis;
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
  }

  .game-grid-card__actions :deep(.game-grid-card__primary-action) {
    border-color: var(--accent-border);
    background: rgba(13, 12, 17, 0.64);
    color: var(--accent-strong);
    backdrop-filter: blur(10px);
  }

  .game-grid-card__actions :deep(.game-grid-card__primary-action:hover) {
    background: rgba(33, 29, 47, 0.82);
  }

  @media (max-width: 720px) {
    .game-grid-card {
      grid-template-columns: 48px minmax(0, 1fr);
    }

    .game-grid-card__actions {
      flex-wrap: wrap;
    }
  }
</style>
