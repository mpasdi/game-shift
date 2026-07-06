<script setup lang="ts">
  import { computed } from 'vue'
  import { Pencil, Play, Star, Trash2 } from '@lucide/vue'
  import IconButton from '../../../shared/components/IconButton.vue'
  import type { Game } from '../types/game'
  import GameArtwork from './GameArtwork.vue'

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
</script>

<template>
  <article class="game-table-row" role="row">
    <GameArtwork :game="props.game" variant="list" />

    <div class="game-table-row__name" role="cell">
      <h2 :title="props.game.name">{{ props.game.name }}</h2>
    </div>

    <div class="game-table-row__stat" role="cell" :title="lastPlayText">{{ lastPlayText }}</div>
    <div class="game-table-row__stat" role="cell" :title="`${props.game.playCount} 次`">{{ props.game.playCount }}</div>

    <div class="game-table-row__actions" role="cell">
      <IconButton
        class="game-table-row__action game-table-row__action--primary"
        label="启动游戏"
        variant="active"
        :disabled="props.isLaunching"
        @click="emit('launch', props.game)"
      >
        <Play :size="15" />
      </IconButton>
      <IconButton
        class="game-table-row__action game-table-row__action--favorite"
        :label="props.game.favorite ? '取消收藏' : '收藏游戏'"
        variant="plain"
        @click="emit('toggleFavorite', props.game)"
      >
        <Star :size="16" :fill="props.game.favorite ? 'currentColor' : 'none'" />
      </IconButton>
      <IconButton
        v-if="props.showManageActions"
        class="game-table-row__action"
        label="编辑游戏"
        variant="plain"
        @click="emit('edit', props.game)"
      >
        <Pencil :size="16" />
      </IconButton>
      <IconButton
        v-if="props.showManageActions"
        class="game-table-row__action"
        label="移除游戏"
        variant="plain"
        @click="emit('remove', props.game)"
      >
        <Trash2 :size="16" />
      </IconButton>
    </div>
  </article>
</template>

<style scoped>
  .game-table-row {
    display: grid;
    grid-template-columns: 34px minmax(180px, 2fr) minmax(150px, 1fr) minmax(54px, 0.45fr) auto;
    column-gap: 18px;
    align-items: center;
    min-height: 54px;
    border-bottom: 1px solid var(--border);
    background: transparent;
    padding: 8px 12px;
    transition: background 170ms ease;
  }

  .game-table-row:last-child {
    border-bottom: 0;
  }

  .game-table-row:hover {
    background: rgba(255, 255, 255, 0.055);
  }

  .game-table-row__name {
    min-width: 0;
  }

  .game-table-row__name h2 {
    overflow: hidden;
    margin: 0;
    color: var(--text);
    font-size: var(--font-size-md);
    line-height: 1.28;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .game-table-row__stat {
    min-width: 0;
    overflow: hidden;
    color: rgba(245, 242, 255, 0.84);
    font-size: var(--font-size-sm);
    font-weight: 600;
    line-height: 1.15;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .game-table-row__actions {
    justify-self: end;
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }

  .game-table-row__actions :deep(.icon-button) {
    width: 28px;
    min-width: 28px;
    height: 28px;
  }

  .game-table-row__actions :deep(.game-table-row__action--primary) {
    border-color: var(--accent-border);
    background: rgba(47, 34, 82, 0.72);
    color: var(--accent-strong);
  }
  .game-table-row__actions :deep(.game-table-row__action--primary:hover:not(:disabled)) {
    border-color: transparent;
    background: linear-gradient(180deg, #8d73ff, #6d50e8);
    color: #ffffff;
    box-shadow: 0 8px 18px rgba(73, 51, 180, 0.28);
  }

  .game-table-row__actions :deep(.game-table-row__action--favorite) {
    border-color: transparent;
    background: transparent;
  }

  .game-table-row__actions :deep(.game-table-row__action--favorite:hover) {
    border-color: var(--border);
    background: var(--surface);
  }

  .game-table-row__actions :deep(.game-table-row__action--favorite svg) {
    color: var(--accent-strong);
  }
</style>
