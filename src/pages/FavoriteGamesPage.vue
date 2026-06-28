<script setup lang="ts">
  import { computed, onMounted } from 'vue'
  import { Star, Search } from '@lucide/vue'
  import { storeToRefs } from 'pinia'
  import GameCollectionSection from '../modules/games/components/GameCollectionSection.vue'
  import { useGameLibraryActions } from '../modules/games/composables/useGameLibraryActions'
  import { useGamesStore } from '../modules/games/stores/games'

  const gamesStore = useGamesStore()
  const { searchText } = storeToRefs(gamesStore)
  const actions = useGameLibraryActions()

  const visibleGames = computed(() => gamesStore.filteredGames)
  const hasSearch = computed(() => searchText.value.trim() !== '')

  onMounted(() => {
    gamesStore.setFilter('favorite')
  })
</script>

<template>
  <GameCollectionSection
    :title="hasSearch ? '搜索结果' : '收藏游戏'"
    :icon="hasSearch ? Search : Star"
    :meta="hasSearch ? `${visibleGames.length} 个匹配` : `${gamesStore.countByFilter('favorite')} 个收藏`"
    :games="visibleGames"
    :view-mode="actions.viewMode.value"
    :action-mode="actions.viewMode.value === 'grid' ? 'quick' : 'full'"
    :show-manage-actions="false"
    :launching-game-ids="actions.launchingGameIds.value"
    :list-class="{ 'favorite-grid': actions.viewMode.value === 'grid' }"
    empty-title="还没有收藏游戏"
    empty-description="点击游戏卡片上的星标后，收藏内容会展示在这里。"
    @update-view-mode="actions.setViewMode"
    @launch="actions.launchGame"
    @toggle-favorite="actions.toggleFavorite"
  />
</template>
