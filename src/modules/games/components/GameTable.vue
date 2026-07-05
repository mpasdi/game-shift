<script setup lang="ts">
  import { computed } from 'vue'
  import { Pencil, Play, Star, Trash2 } from '@lucide/vue'
  import BaseButton from '../../../shared/components/BaseButton.vue'
  import DataTable from '../../../shared/components/DataTable.vue'
  import IconButton from '../../../shared/components/IconButton.vue'
  import type { Game } from '../types/game'
  import GameArtwork from './GameArtwork.vue'

  type GameTableDensity = 'compact' | 'regular'
  type GameTableActionMode = 'full' | 'quick' | 'launch-only'
  type GameTableLaunchStyle = 'button' | 'icon'
  interface GameTableColumn {
    key: string
    label: string
    width?: string
    align?: 'left' | 'center' | 'right'
  }

  const props = withDefaults(
    defineProps<{
      games: Game[]
      density?: GameTableDensity
      actionMode?: GameTableActionMode
      launchStyle?: GameTableLaunchStyle
      launchingGameIds?: string[]
    }>(),
    {
      density: 'regular',
      actionMode: 'full',
      launchStyle: 'icon',
      launchingGameIds: () => []
    }
  )

  const emit = defineEmits<{
    edit: [game: Game]
    launch: [game: Game]
    toggleFavorite: [game: Game]
    remove: [game: Game]
  }>()

  const actionColumnWidth = computed(() => {
    if (props.launchStyle === 'button') return '92px'
    if (props.actionMode === 'full') return '132px'
    if (props.actionMode === 'quick') return '76px'
    return '48px'
  })

  const columns = computed<GameTableColumn[]>(() => [
    { key: 'game', label: '游戏', width: 'minmax(220px, 2fr)' },
    { key: 'lastPlay', label: '最近游玩', width: 'minmax(150px, 1fr)' },
    { key: 'playCount', label: '次数', width: '72px', align: 'center' },
    { key: 'actions', label: '操作', width: actionColumnWidth.value, align: 'right' }
  ])

  function getExeFileName(game: Game) {
    return game.exePath.split(/[\\/]/).pop() ?? game.exePath
  }

  function formatLastPlayFull(game: Game) {
    if (!game.lastPlayTime) return '无启动记录'
    return new Intl.DateTimeFormat('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    }).format(new Date(game.lastPlayTime))
  }

  function isLaunching(game: Game) {
    return props.launchingGameIds.includes(game.id)
  }

  function getTableRowKey(row: unknown) {
    return (row as Game).id
  }
</script>

<template>
  <DataTable
    class="game-table"
    :columns="columns"
    :rows="props.games"
    :row-key="getTableRowKey"
    :density="props.density"
    aria-label="游戏列表"
  >
    <template #cell-game="{ row }">
      <div class="game-table__game">
        <GameArtwork :game="row as Game" variant="list" />
        <div class="game-table__name">
          <h2 :title="(row as Game).name">{{ (row as Game).name }}</h2>
          <p :title="(row as Game).exePath">{{ getExeFileName(row as Game) }}</p>
        </div>
      </div>
    </template>

    <template #cell-lastPlay="{ row }">
      <span class="game-table__stat" :title="formatLastPlayFull(row as Game)">
        {{ formatLastPlayFull(row as Game) }}
      </span>
    </template>

    <template #cell-playCount="{ row }">
      <span class="game-table__stat" :title="`${(row as Game).playCount} 次`">{{ (row as Game).playCount }}</span>
    </template>

    <template #cell-actions="{ row }">
      <div class="game-table__actions">
        <BaseButton
          v-if="props.launchStyle === 'button'"
          class="game-table__launch-button"
          size="sm"
          variant="primary"
          :disabled="isLaunching(row as Game)"
          @click="emit('launch', row as Game)"
        >
          <template #icon><Play :size="14" /></template>
          启动
        </BaseButton>
        <IconButton
          v-else
          class="game-table__action game-table__action--primary"
          label="启动游戏"
          variant="active"
          :disabled="isLaunching(row as Game)"
          @click="emit('launch', row as Game)"
        >
          <Play :size="15" />
        </IconButton>
        <IconButton
          v-if="props.actionMode === 'full' || props.actionMode === 'quick'"
          class="game-table__action game-table__action--favorite"
          :label="(row as Game).favorite ? '取消收藏' : '收藏游戏'"
          variant="plain"
          @click="emit('toggleFavorite', row as Game)"
        >
          <Star :size="16" :fill="(row as Game).favorite ? 'currentColor' : 'none'" />
        </IconButton>
        <IconButton
          v-if="props.actionMode === 'full'"
          class="game-table__action"
          label="编辑游戏"
          variant="plain"
          @click="emit('edit', row as Game)"
        >
          <Pencil :size="16" />
        </IconButton>
        <IconButton
          v-if="props.actionMode === 'full'"
          class="game-table__action"
          label="移除游戏"
          variant="plain"
          @click="emit('remove', row as Game)"
        >
          <Trash2 :size="16" />
        </IconButton>
      </div>
    </template>
  </DataTable>
</template>

<style scoped>
  .game-table {
    min-width: 0;
  }

  .game-table__game {
    display: grid;
    grid-template-columns: 30px minmax(0, 1fr);
    gap: 10px;
    align-items: center;
    min-width: 0;
  }

  .game-table__name {
    display: grid;
    min-width: 0;
    gap: 3px;
  }

  .game-table__name h2 {
    overflow: hidden;
    margin: 0;
    color: var(--text);
    font-size: var(--font-size-md);
    line-height: 1.28;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .game-table__name p {
    overflow: hidden;
    margin: 0;
    color: var(--text-muted);
    font-size: var(--font-size-xs);
    line-height: 1.25;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .game-table__stat {
    overflow: hidden;
    display: block;
    color: rgba(245, 242, 255, 0.84);
    font-size: var(--font-size-sm);
    font-weight: 600;
    line-height: 1.15;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .game-table__actions {
    display: flex;
    justify-content: flex-end;
    gap: 6px;
  }

  .game-table__actions :deep(.icon-button) {
    width: 28px;
    min-width: 28px;
    height: 28px;
  }

  .game-table__launch-button {
    min-width: 70px;
    box-shadow: 0 8px 18px rgba(73, 51, 180, 0.28);
  }

  .game-table__actions :deep(.game-table__action--primary) {
    border-color: var(--accent-border);
    background: rgba(47, 34, 82, 0.72);
    color: var(--accent-strong);
  }

  .game-table :deep(.data-table__row:hover .game-table__action--primary:not(:disabled)),
  .game-table__actions :deep(.game-table__action--primary:hover:not(:disabled)) {
    border-color: transparent;
    background: linear-gradient(180deg, #8d73ff, #6d50e8);
    color: #ffffff;
    box-shadow: 0 8px 18px rgba(73, 51, 180, 0.28);
  }

  .game-table__actions :deep(.game-table__action--favorite) {
    border-color: transparent;
    background: transparent;
  }

  .game-table__actions :deep(.game-table__action--favorite:hover) {
    border-color: var(--border);
    background: var(--surface);
  }

  .game-table__actions :deep(.game-table__action--favorite svg) {
    color: var(--accent-strong);
  }
</style>
