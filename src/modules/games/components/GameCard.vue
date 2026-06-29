<script setup lang="ts">
  import { computed } from 'vue'
  import { convertFileSrc } from '@tauri-apps/api/core'
  import { Pencil, Play, Star, Trash2 } from '@lucide/vue'
  import IconButton from '../../../shared/components/IconButton.vue'
  import type { Game } from '../types/game'

  const props = withDefaults(
    defineProps<{
      game: Game
      viewMode: 'grid' | 'list'
      actionMode?: 'full' | 'quick'
      showManageActions?: boolean
      isLaunching?: boolean
    }>(),
    {
      actionMode: 'full',
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

  const initial = computed(() => props.game.name.trim().slice(0, 1).toUpperCase() || 'G')
  const exeFileName = computed(() => props.game.exePath.split(/[\\/]/).pop() ?? props.game.exePath)
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
  const cardMetaText = computed(() => `${lastPlayText.value} · ${props.game.playCount} 次`)
  const coverSrc = computed(() => toLocalAssetSrc(props.game.cover))
  const iconSrc = computed(() => toLocalAssetSrc(props.game.icon))

  function toLocalAssetSrc(path?: string | null) {
    return path ? convertFileSrc(path) : null
  }
</script>

<template>
  <article class="game-card" :class="[`game-card--${props.viewMode}`, `game-card--actions-${props.actionMode}`]">
    <div class="game-card__icon" aria-hidden="true">
      <img v-if="props.viewMode !== 'list' && coverSrc" class="game-card__cover-image" :src="coverSrc" alt="" />
      <img v-else-if="iconSrc" class="game-card__logo-image" :src="iconSrc" alt="" />
      <span v-else>{{ initial }}</span>
    </div>

    <div class="game-card__content">
      <div class="game-card__title-row">
        <h2>{{ props.game.name }}</h2>
      </div>
      <p v-if="props.viewMode === 'grid' && props.actionMode === 'full'" class="game-card__meta">{{ cardMetaText }}</p>
      <p v-else class="game-card__path" :title="props.game.exePath">{{ exeFileName }}</p>
      <dl v-if="props.viewMode !== 'grid' || props.actionMode !== 'full'" class="game-card__stats">
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
        <Star :size="16" :fill="props.game.favorite ? 'currentColor' : 'none'" />
      </IconButton>
      <IconButton label="启动游戏" variant="active" :disabled="props.isLaunching" @click="emit('launch', props.game)">
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
  .game-card {
    position: relative;
    display: grid;
    gap: 13px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    padding: 14px;
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.18);
    transition:
      border-color 170ms ease,
      background 170ms ease,
      box-shadow 170ms ease;
  }

  .game-card:hover {
    border-color: var(--border-strong);
    background: var(--surface-hover);
    box-shadow: 0 14px 34px rgba(0, 0, 0, 0.2);
  }

  .game-card--grid {
    grid-template-columns: minmax(0, 1fr);
    gap: 10px;
    min-height: 300px;
    padding: 10px;
  }

  .game-card--list {
    grid-template-columns: 34px minmax(220px, 360px) 150px 54px minmax(0, 1fr) auto;
    column-gap: 18px;
    align-items: center;
    min-height: 54px;
    border-width: 0 0 1px;
    border-radius: 0;
    background: transparent;
    padding: 8px 12px;
    box-shadow: none;
    backdrop-filter: none;
  }

  .game-card--list:hover {
    background: rgba(255, 255, 255, 0.055);
    box-shadow: none;
  }

  .game-card__icon {
    display: grid;
    width: 50px;
    height: 50px;
    place-items: center;
    border: 1px solid var(--accent-border);
    border-radius: 8px;
    background: linear-gradient(135deg, rgba(124, 92, 255, 0.22), rgba(255, 255, 255, 0.06));
    overflow: hidden;
    color: #f5f3ff;
    font-size: 20px;
    font-weight: 850;
  }

  .game-card__cover-image {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .game-card__logo-image {
    width: 62%;
    height: 62%;
    object-fit: contain;
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

  .game-card__meta {
    overflow: hidden;
    margin: 6px 0 0;
    color: var(--text-muted);
    font-size: 12px;
    line-height: 1.35;
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
    display: flex;
    justify-content: space-between;
    gap: 0;
    margin-top: 0;
  }

  .game-card--grid .game-card__actions :deep(.icon-button) {
    width: 30px;
    min-width: 30px;
    height: 30px;
  }

  .game-card--grid.game-card--actions-full .game-card__actions :deep(.icon-button:first-child) {
    position: absolute;
    top: 14px;
    right: 14px;
    width: 30px;
    min-width: 30px;
    height: 30px;
  }

  .game-card--grid.game-card--actions-full .game-card__icon {
    width: 100%;
    height: 190px;
    border-radius: 8px;
    background:
      radial-gradient(circle at 50% 18%, rgba(255, 255, 255, 0.13), transparent 28%),
      linear-gradient(160deg, rgba(124, 92, 255, 0.34), rgba(20, 18, 27, 0.94));
    font-size: 38px;
  }

  .game-card--grid.game-card--actions-full .game-card__logo-image {
    width: 60px;
    height: 60px;
  }

  .game-card--grid.game-card--actions-full .game-card__content {
    display: grid;
    gap: 5px;
  }

  .game-card--grid.game-card--actions-full .game-card__title-row h2 {
    display: -webkit-box;
    min-height: 34px;
    font-size: 13px;
    line-height: 1.3;
    white-space: normal;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
  }

  .game-card--grid.game-card--actions-full .game-card__meta {
    margin: 0;
    font-size: 11px;
  }

  .game-card--list .game-card__icon {
    width: 30px;
    height: 30px;
    font-size: 13px;
  }

  .game-card--list .game-card__logo-image {
    width: 22px;
    height: 22px;
  }

  .game-card--list .game-card__content {
    display: contents;
  }

  .game-card--list .game-card__title-row {
    min-width: 0;
  }

  .game-card--list .game-card__path {
    display: none;
  }

  .game-card--list .game-card__stats {
    display: contents;
  }

  .game-card--list .game-card__stats div {
    display: grid;
    gap: 1px;
    min-width: 0;
  }

  .game-card--list .game-card__stats dt {
    display: none;
  }

  .game-card--list .game-card__stats dd {
    margin: 0;
    font-size: 12px;
    line-height: 1.15;
  }

  .game-card--list .game-card__actions {
    grid-column: 6;
    display: grid;
    grid-template-columns: repeat(4, 28px);
    justify-content: flex-end;
    gap: 6px;
  }

  .game-card--list .game-card__actions :deep(.icon-button:nth-child(1)) {
    grid-column: 2;
    grid-row: 1;
  }

  .game-card--list .game-card__actions :deep(.icon-button:nth-child(2)) {
    grid-column: 1;
    grid-row: 1;
  }

  .game-card--list .game-card__actions :deep(.icon-button) {
    width: 28px;
    min-width: 28px;
    height: 28px;
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
      linear-gradient(145deg, rgba(124, 92, 255, 0.28), rgba(46, 73, 86, 0.16)),
      linear-gradient(180deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.03));
    font-size: 32px;
  }

  .game-card--actions-quick .game-card__logo-image {
    width: 48px;
    height: 48px;
  }

  .game-card--actions-quick .game-card__content {
    display: grid;
    gap: 3px;
  }

  .game-card--actions-quick .game-card__title-row h2 {
    display: -webkit-box;
    min-height: 34px;
    font-size: 13px;
    line-height: 1.3;
    white-space: normal;
    -webkit-box-orient: vertical;
    -webkit-line-clamp: 2;
  }

  .game-card--actions-quick .game-card__path {
    margin: 0;
    font-size: 11px;
  }

  .game-card--actions-quick .game-card__stats {
    display: none;
  }

  .game-card--actions-quick .game-card__actions {
    display: contents;
    margin: 0;
  }

  .game-card--actions-quick .game-card__actions :deep(.icon-button) {
    position: absolute;
    right: 8px;
    width: 28px;
    min-width: 28px;
    height: 28px;
    border-radius: 8px;
    background: rgba(13, 12, 17, 0.64);
  }

  .game-card--actions-quick .game-card__actions :deep(.icon-button:first-child) {
    top: 8px;
    margin-right: 0;
  }

  .game-card--actions-quick .game-card__actions :deep(.icon-button:last-child) {
    top: auto;
    bottom: 8px;
    min-height: 28px;
  }

  @media (max-width: 720px) {
    .game-card--list,
    .game-card--grid {
      grid-template-columns: 48px minmax(0, 1fr);
    }

    .game-card--list {
      column-gap: 12px;
    }

    .game-card--list .game-card__content {
      display: grid;
      grid-template-columns: minmax(0, 1fr);
    }

    .game-card--list .game-card__stats {
      display: flex;
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
