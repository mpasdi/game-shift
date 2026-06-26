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
