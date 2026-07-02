<script setup lang="ts">
  import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
  import { RouterLink } from 'vue-router'
  import { Search, Star, Library } from '@lucide/vue'
  import { storeToRefs } from 'pinia'
  import GameCollectionSection from '../modules/games/components/GameCollectionSection.vue'
  import GameList from '../modules/games/components/GameList.vue'
  import { useGameLibraryActions } from '../modules/games/composables/useGameLibraryActions'
  import { useGamesStore } from '../modules/games/stores/games'
  import { routeNames } from '../router/routeNames'

  const gamesStore = useGamesStore()
  const { games, searchText } = storeToRefs(gamesStore)
  const actions = useGameLibraryActions()

  const visibleGames = computed(() => gamesStore.filteredGames)
  const favoriteGames = computed(() => games.value.filter((game) => game.favorite))
  const hasSearch = computed(() => searchText.value.trim() !== '')
  const favoritePreview = ref<HTMLElement | null>(null)
  const favoritePreviewColumns = ref(4)
  const favoritePreviewGames = computed(() => favoriteGames.value.slice(0, favoritePreviewColumns.value))
  let favoritePreviewObserver: ResizeObserver | null = null

  onMounted(() => {
    gamesStore.setFilter('all')
    void nextTick(() => {
      updateFavoritePreviewColumns()
      if (!favoritePreview.value) return
      favoritePreviewObserver = new ResizeObserver(updateFavoritePreviewColumns)
      favoritePreviewObserver.observe(favoritePreview.value)
    })
  })

  watch(favoriteGames, () => {
    void nextTick(updateFavoritePreviewColumns)
  })

  onBeforeUnmount(() => {
    favoritePreviewObserver?.disconnect()
    favoritePreviewObserver = null
  })

  function getFavoriteColumnCount(width: number) {
    const minimumCardWidth = 150
    const gap = 12
    const columns = Math.floor((width + gap) / (minimumCardWidth + gap))
    return Math.min(10, Math.max(2, columns))
  }

  function updateFavoritePreviewColumns() {
    const width = favoritePreview.value?.clientWidth ?? 0
    if (!width) return
    favoritePreviewColumns.value = getFavoriteColumnCount(width)
  }
</script>

<template>
  <GameCollectionSection
    v-if="hasSearch"
    title="搜索结果"
    :icon="Search"
    :meta="`${visibleGames.length} 个匹配`"
    :games="visibleGames"
    :view-mode="actions.viewMode.value"
    :launching-game-ids="actions.launchingGameIds.value"
    empty-title="没有找到匹配的游戏"
    empty-description="可以调整搜索关键词，或手动添加新的启动程序。"
    @update-view-mode="actions.setViewMode"
    @edit="actions.openEditGameDialog"
    @launch="actions.launchGame"
    @toggle-favorite="actions.toggleFavorite"
    @remove="actions.openRemoveGameDialog"
  />

  <template v-else>
    <section class="library-section library-section--favorites">
      <div class="section-heading">
        <h2 class="section-title">
          <Star :size="14" />
          收藏游戏
          <span class="section-title__meta">{{ favoriteGames.length }} 个收藏</span>
        </h2>
        <div class="section-heading__actions">
          <RouterLink class="link-button" :to="{ name: routeNames.favorites }">查看更多</RouterLink>
        </div>
      </div>

      <div v-if="favoriteGames.length === 0" class="section-empty">
        <Star :size="18" />
        <span>点击游戏卡片上的星标，把常玩的游戏放到这里。</span>
      </div>
      <div v-else ref="favoritePreview" class="favorite-preview">
        <GameList
          class="home-favorite-grid"
          :style="{ '--favorite-columns': favoritePreviewColumns }"
          :games="favoritePreviewGames"
          view-mode="grid"
          action-mode="quick"
          :show-manage-actions="false"
          :launching-game-ids="actions.launchingGameIds.value"
          @launch="actions.launchGame"
          @toggle-favorite="actions.toggleFavorite"
        />
      </div>
    </section>

    <GameCollectionSection
      title="全部游戏"
      :icon="Library"
      :meta="`${games.length} 个游戏`"
      :games="visibleGames"
      :view-mode="actions.viewMode.value"
      :launching-game-ids="actions.launchingGameIds.value"
      empty-title="没有找到匹配的游戏"
      empty-description="可以调整搜索关键词，或手动添加新的启动程序。"
      @update-view-mode="actions.setViewMode"
      @edit="actions.openEditGameDialog"
      @launch="actions.launchGame"
      @toggle-favorite="actions.toggleFavorite"
      @remove="actions.openRemoveGameDialog"
    />
  </template>
</template>

<style scoped>
  .library-section {
    display: grid;
    gap: 14px;
    animation: section-in 180ms ease-out both;
  }

  .section-heading {
    display: flex;
    gap: 14px;
    align-items: center;
    justify-content: space-between;
  }

  .section-title {
    display: inline-flex;
    gap: 7px;
    align-items: center;
    margin: 0;
    color: var(--text);
    font-size: var(--font-size-md);
    font-weight: 700;
    line-height: 1.2;
  }

  .section-title svg {
    width: 15px;
    height: 15px;
    color: var(--accent-strong);
  }

  .section-title__meta {
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    font-weight: 600;
  }

  .section-heading__actions {
    display: inline-flex;
    gap: 10px;
    align-items: center;
  }

  .link-button {
    border: 0;
    background: transparent;
    color: var(--text-muted);
    font-size: var(--font-size-sm);
    padding: 5px 0;
    text-decoration: none;
  }

  .link-button:hover {
    color: var(--accent-strong);
  }

  .section-empty {
    display: flex;
    gap: 10px;
    align-items: center;
    min-height: 58px;
    border: 1px solid rgba(255, 255, 255, 0.075);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.025);
    color: var(--text-muted);
    padding: 14px 16px;
  }

  .section-empty svg {
    flex: 0 0 auto;
    color: var(--accent-strong);
  }

  .favorite-preview {
    min-width: 0;
  }

  @keyframes section-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }

    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 720px) {
    .section-heading__actions {
      width: 100%;
      justify-content: space-between;
    }
  }
</style>
