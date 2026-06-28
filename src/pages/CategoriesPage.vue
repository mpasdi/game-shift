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
    title="分类功能待接入"
    description="后续可以在这里维护游戏分类、标签或平台维度。"
  >
    <template #icon><Folder :size="15" /></template>
  </EmptyState>
</template>
