<script setup lang="ts">
  import { computed, onMounted } from 'vue'
  import { Library, Search } from '@lucide/vue'
  import { storeToRefs } from 'pinia'
  import EmptyState from '../shared/components/EmptyState.vue'
  import GameCardGrid from '../modules/games/components/GameCardGrid.vue'
  import GameCollectionLayout from '../modules/games/components/GameCollectionLayout.vue'
  import GameGridCard from '../modules/games/components/GameGridCard.vue'
  import GameTable from '../modules/games/components/GameTable.vue'
  import { useGameLibraryActions } from '../modules/games/composables/useGameLibraryActions'
  import { useGamesStore } from '../modules/games/stores/games'

  const gamesStore = useGamesStore()
  const { games, searchText } = storeToRefs(gamesStore)
  const actions = useGameLibraryActions()

  const visibleGames = computed(() => gamesStore.filteredGames)
  const hasSearch = computed(() => searchText.value.trim() !== '')
  const title = computed(() => (hasSearch.value ? '搜索结果' : '全部游戏'))
  const icon = computed(() => (hasSearch.value ? Search : Library))
  const meta = computed(() =>
    hasSearch.value ? `${visibleGames.value.length} 个匹配` : `${games.value.length} 个游戏`
  )
  const emptyTitle = computed(() => (hasSearch.value ? '没有找到匹配的游戏' : '还没有添加任何游戏'))
  const emptyDescription = computed(() =>
    hasSearch.value ? '可以调整搜索关键词，或添加新的启动程序。' : '使用右上角的添加或扫描功能开始建立游戏库。'
  )

  onMounted(() => {
    gamesStore.setFilter('all')
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
      :label="emptyTitle"
      :eyebrow="title"
      :title="emptyTitle"
      :description="emptyDescription"
    >
      <template #icon><component :is="icon" :size="15" /></template>
    </EmptyState>

    <GameCardGrid v-else-if="actions.viewMode.value === 'grid'" variant="full">
      <GameGridCard
        v-for="game in visibleGames"
        :key="game.id"
        :game="game"
        :is-launching="actions.launchingGameIds.value.includes(game.id)"
        @edit="actions.openEditGameDialog"
        @launch="actions.launchGame"
        @toggle-favorite="actions.toggleFavorite"
        @remove="actions.openRemoveGameDialog"
      />
    </GameCardGrid>

    <GameTable
      v-else
      :games="visibleGames"
      :launching-game-ids="actions.launchingGameIds.value"
      @edit="actions.openEditGameDialog"
      @launch="actions.launchGame"
      @toggle-favorite="actions.toggleFavorite"
      @remove="actions.openRemoveGameDialog"
    />
  </GameCollectionLayout>
</template>
