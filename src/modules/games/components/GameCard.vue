<script setup lang="ts">
  import { computed } from 'vue'
  import { Heart, Pencil, Play, Trash2 } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import IconButton from '../../../shared/components/IconButton.vue'
  import type { Game } from '../types/game'

  const props = withDefaults(
    defineProps<{
      game: Game
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

  const initial = computed(() => props.game.name.trim().slice(0, 1).toUpperCase() || 'G')
  const exeFileName = computed(() => props.game.exePath.split(/[\\/]/).pop() ?? props.game.exePath)
  const lastPlayText = computed(() => {
    if (!props.game.lastPlayTime) return '尚未启动'
    return new Intl.DateTimeFormat('zh-CN', {
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    }).format(new Date(props.game.lastPlayTime))
  })
</script>

<template>
  <article class="game-card" :class="[`game-card--${props.viewMode}`, `game-card--actions-${props.actionMode}`]">
    <div class="game-card__icon" aria-hidden="true">{{ initial }}</div>

    <div class="game-card__content">
      <div class="game-card__title-row">
        <h2>{{ props.game.name }}</h2>
      </div>
      <p class="game-card__path" :title="props.game.exePath">{{ exeFileName }}</p>
      <dl class="game-card__stats">
        <div>
          <dt>最近</dt>
          <dd>{{ lastPlayText }}</dd>
        </div>
        <div>
          <dt>次数</dt>
          <dd>{{ props.game.playCount }}</dd>
        </div>
      </dl>
    </div>

    <div class="game-card__actions">
      <IconButton
        :label="props.game.favorite ? '取消收藏' : '收藏游戏'"
        :variant="props.game.favorite ? 'active' : 'plain'"
        @click="emit('toggleFavorite', props.game)"
      >
        <Heart :size="16" :fill="props.game.favorite ? 'currentColor' : 'none'" />
      </IconButton>
      <BaseButton variant="primary" size="sm" disabled>
        <template #icon><Play :size="15" /></template>
        启动
      </BaseButton>
      <IconButton v-if="props.actionMode === 'full'" label="编辑游戏" @click="emit('edit', props.game)">
        <Pencil :size="16" />
      </IconButton>
      <IconButton
        v-if="props.actionMode === 'full'"
        label="移除游戏"
        variant="danger"
        @click="emit('remove', props.game)"
      >
        <Trash2 :size="16" />
      </IconButton>
    </div>
  </article>
</template>

<style scoped>
  .game-card {
    position: relative;
    display: grid;
    gap: 13px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.07);
    padding: 14px;
    box-shadow: 0 18px 48px rgba(0, 0, 0, 0.2);
    backdrop-filter: blur(12px);
    transition:
      transform 170ms ease,
      border-color 170ms ease,
      background 170ms ease,
      box-shadow 170ms ease;
  }

  .game-card:hover {
    transform: translateY(-2px);
    border-color: rgba(167, 139, 250, 0.4);
    background: rgba(255, 255, 255, 0.11);
    box-shadow: 0 22px 58px rgba(0, 0, 0, 0.3);
  }

  .game-card--grid {
    min-height: 178px;
    grid-template-columns: 50px minmax(0, 1fr);
    align-content: start;
  }

  .game-card--list {
    grid-template-columns: 34px minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
    min-height: 48px;
    border-width: 0 0 1px;
    border-radius: 0;
    background: transparent;
    padding: 6px 10px;
    box-shadow: none;
    backdrop-filter: none;
  }

  .game-card--list:hover {
    transform: none;
    background: rgba(255, 255, 255, 0.075);
    box-shadow: none;
  }

  .game-card__icon {
    display: grid;
    width: 50px;
    height: 50px;
    place-items: center;
    border: 1px solid rgba(167, 139, 250, 0.24);
    border-radius: 8px;
    background: linear-gradient(135deg, rgba(167, 139, 250, 0.36), rgba(255, 255, 255, 0.08));
    color: #f5f3ff;
    font-size: 20px;
    font-weight: 850;
  }

  .game-card__content {
    min-width: 0;
  }

  .game-card__title-row h2 {
    overflow: hidden;
    margin: 0;
    color: var(--text);
    font-size: 15px;
    line-height: 1.28;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .game-card__path {
    overflow: hidden;
    margin: 5px 0 10px;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.3;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .game-card__stats {
    display: flex;
    gap: 16px;
    margin: 0;
  }

  .game-card__stats dt {
    color: var(--text-subtle);
    font-size: 11px;
  }

  .game-card__stats dd {
    margin: 3px 0 0;
    color: rgba(245, 242, 255, 0.84);
    font-size: 12px;
    font-weight: 700;
  }

  .game-card__actions {
    display: flex;
    gap: 8px;
    align-items: center;
  }

  .game-card--grid .game-card__actions {
    grid-column: 1 / -1;
    justify-content: space-between;
    margin-top: auto;
  }

  .game-card--list .game-card__icon {
    width: 30px;
    height: 30px;
    font-size: 13px;
  }

  .game-card--list .game-card__content {
    display: grid;
    grid-template-columns: minmax(150px, 1fr) 86px 52px;
    column-gap: 18px;
    align-items: center;
  }

  .game-card--list .game-card__path {
    margin: 2px 0 0;
  }

  .game-card--list .game-card__stats {
    display: contents;
  }

  .game-card--list .game-card__stats div {
    display: grid;
    gap: 1px;
  }

  .game-card--list .game-card__stats dt {
    font-size: 10px;
    line-height: 1.1;
  }

  .game-card--list .game-card__stats dd {
    margin: 0;
    font-size: 12px;
    line-height: 1.15;
  }

  .game-card--list .game-card__actions {
    justify-content: flex-end;
    gap: 6px;
  }

  .game-card--list .game-card__actions :deep(.icon-button) {
    width: 28px;
    min-width: 28px;
    height: 28px;
  }

  .game-card--list .game-card__actions :deep(.base-button) {
    min-height: 28px;
    padding: 0 10px;
  }

  .game-card--actions-quick {
    min-height: 190px;
    grid-template-columns: 1fr;
    gap: 9px;
    padding: 10px;
  }

  .game-card--actions-quick .game-card__icon {
    width: 100%;
    height: 104px;
    border-radius: 8px;
    background:
      linear-gradient(145deg, rgba(167, 139, 250, 0.42), rgba(71, 85, 105, 0.18)),
      linear-gradient(180deg, rgba(255, 255, 255, 0.14), rgba(255, 255, 255, 0.03));
    font-size: 32px;
  }

  .game-card--actions-quick .game-card__content {
    display: grid;
    gap: 3px;
  }

  .game-card--actions-quick .game-card__title-row h2 {
    font-size: 13px;
  }

  .game-card--actions-quick .game-card__path {
    margin: 0;
    font-size: 11px;
  }

  .game-card--actions-quick .game-card__stats {
    display: none;
  }

  .game-card--actions-quick .game-card__actions {
    position: absolute;
    right: 8px;
    bottom: 8px;
    gap: 5px;
    margin: 0;
  }

  .game-card--actions-quick .game-card__actions :deep(.icon-button) {
    width: 28px;
    min-width: 28px;
    height: 28px;
    border-radius: 8px;
    background: rgba(12, 10, 18, 0.46);
  }

  .game-card--actions-quick .game-card__actions :deep(.base-button) {
    width: 28px;
    min-width: 28px;
    height: 28px;
    min-height: 28px;
    padding: 0;
    border-radius: 8px;
  }

  .game-card--actions-quick .game-card__actions :deep(.base-button__label) {
    display: none;
  }

  @media (max-width: 720px) {
    .game-card--list,
    .game-card--grid {
      grid-template-columns: 48px minmax(0, 1fr);
    }

    .game-card--list .game-card__actions {
      grid-column: 1 / -1;
      justify-content: space-between;
    }

    .game-card__actions {
      flex-wrap: wrap;
    }
  }
</style>
