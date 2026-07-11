<script setup lang="ts">
  import { computed, onMounted } from 'vue'
  import { Library, Search } from '@lucide/vue'
  import { storeToRefs } from 'pinia'
  import GameCollectionSection from '../modules/games/components/GameCollectionSection.vue'
  import { useGameLibraryActions } from '../modules/games/composables/useGameLibraryActions'
  import { useGamesStore } from '../modules/games/stores/games'

  const gamesStore = useGamesStore()
  const { games, searchText } = storeToRefs(gamesStore)
  const actions = useGameLibraryActions()

  const visibleGames = computed(() => gamesStore.filteredGames)
  const hasSearch = computed(() => searchText.value.trim() !== '')

  onMounted(() => {
    gamesStore.setFilter('all')
  })
</script>

<template>
  <GameCollectionSection
    :title="hasSearch ? '搜索结果' : '全部游戏'"
    :icon="hasSearch ? Search : Library"
    :meta="hasSearch ? `${visibleGames.length} 个匹配` : `${games.length} 个游戏`"
    :games="visibleGames"
    :view-mode="actions.viewMode.value"
    :launching-game-ids="actions.launchingGameIds.value"
    :empty-title="hasSearch ? '没有找到匹配的游戏' : '还没有添加任何游戏'"
    :empty-description="
      hasSearch ? '可以调整搜索关键词，或添加新的启动程序。' : '使用右上角的添加或扫描功能开始建立游戏库。'
    "
    @update-view-mode="actions.setViewMode"
    @edit="actions.openEditGameDialog"
    @launch="actions.launchGame"
    @toggle-favorite="actions.toggleFavorite"
    @remove="actions.openRemoveGameDialog"
  />
</template>
