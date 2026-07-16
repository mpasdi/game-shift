<script setup lang="ts">
  import { computed, onMounted } from 'vue'
  import { Clock3, Search } from '@lucide/vue'
  import { storeToRefs } from 'pinia'
  import EmptyState from '../shared/components/EmptyState.vue'
  import GameCardGrid from '../modules/games/components/GameCardGrid.vue'
  import GameCollectionLayout from '../modules/games/components/GameCollectionLayout.vue'
  import GameQuickCard from '../modules/games/components/GameQuickCard.vue'
  import GameTable from '../modules/games/components/GameTable.vue'
  import { useGameLibraryActions } from '../modules/games/composables/useGameLibraryActions'
  import { useGamesStore } from '../modules/games/stores/games'

  const gamesStore = useGamesStore()
  const { searchText } = storeToRefs(gamesStore)
  const actions = useGameLibraryActions()

  const visibleGames = computed(() => gamesStore.filteredGames)
  const hasSearch = computed(() => searchText.value.trim() !== '')
  const title = computed(() => (hasSearch.value ? '搜索结果' : '最近游玩'))
  const icon = computed(() => (hasSearch.value ? Search : Clock3))
  const meta = computed(() =>
    hasSearch.value ? `${visibleGames.value.length} 个匹配` : `${gamesStore.countByFilter('recent')} 条记录`
  )

  onMounted(() => {
    gamesStore.setFilter('recent')
  })
</script>

<template>
  <GameCollectionLayout
    :title="title"
    :icon="icon"
    :meta="meta"
    :view-mode="actions.viewMode.value"
    @update-view-mode="actions.setViewMode"
  >
    <EmptyState
      v-if="visibleGames.length === 0"
      variant="plain"
      label="还没有最近游玩记录"
      :eyebrow="title"
      title="还没有最近游玩记录"
      description="启动游戏后，这里会按最近游玩时间展示。"
    >
      <template #icon><component :is="icon" :size="15" /></template>
    </EmptyState>

    <GameCardGrid v-else-if="actions.viewMode.value === 'grid'" variant="quick">
      <GameQuickCard
        v-for="game in visibleGames"
        :key="game.id"
        :game="game"
        :is-launching="actions.launchingGameIds.value.includes(game.id)"
        show-last-play-time
        @launch="actions.launchGame"
        @toggle-favorite="actions.toggleFavorite"
      />
    </GameCardGrid>

    <GameTable
      v-else
      :games="visibleGames"
      action-mode="quick"
      :launching-game-ids="actions.launchingGameIds.value"
      @launch="actions.launchGame"
      @toggle-favorite="actions.toggleFavorite"
    />
  </GameCollectionLayout>
</template>
