<script setup lang="ts">
  import { onMounted } from 'vue'
  import { Folder } from '@lucide/vue'
  import { useGamesStore } from '../modules/games/stores/games'
  import EmptyState from '../shared/components/EmptyState.vue'

  const gamesStore = useGamesStore()

  onMounted(() => {
    gamesStore.setFilter('all')
  })
</script>

<template>
  <EmptyState
    label="分类"
    eyebrow="Categories"
    title="分类管理将在后续版本开放"
    description="当前版本优先提供本地游戏库、收藏、最近游玩和启动能力。分类与标签管理会在游戏库主流程稳定后补充。"
  >
    <template #icon><Folder :size="15" /></template>
  </EmptyState>
</template>
