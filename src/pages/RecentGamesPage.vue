<script setup lang="ts">
  import { computed, onMounted } from 'vue'
  import { Clock3, Search } from '@lucide/vue'
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
    gamesStore.setFilter('recent')
  })
</script>

<template>
  <GameCollectionSection
    :title="hasSearch ? '搜索结果' : '最近游玩'"
    :icon="hasSearch ? Search : Clock3"
    :meta="hasSearch ? `${visibleGames.length} 个匹配` : `${gamesStore.countByFilter('recent')} 条记录`"
    :games="visibleGames"
    :view-mode="actions.viewMode.value"
    :action-mode="actions.viewMode.value === 'grid' ? 'quick' : 'full'"
    :launching-game-ids="actions.launchingGameIds.value"
    :show-manage-actions="false"
    show-last-play-time
    empty-title="还没有最近游玩记录"
    empty-description="启动游戏后，这里会按最近游玩时间展示。"
    @update-view-mode="actions.setViewMode"
    @edit="actions.openEditGameDialog"
    @launch="actions.launchGame"
    @toggle-favorite="actions.toggleFavorite"
    @remove="actions.openRemoveGameDialog"
  />
</template>
