<script setup lang="ts">
  import { computed, onMounted } from 'vue'
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

  onMounted(() => {
    gamesStore.setFilter('all')
  })
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
        </h2>
        <RouterLink class="link-button" :to="{ name: routeNames.favorites }">查看更多</RouterLink>
      </div>

      <div v-if="favoriteGames.length === 0" class="section-empty">
        <Star :size="18" />
        <span>点击游戏卡片上的星标，把常玩的游戏放到这里。</span>
      </div>
      <GameList
        v-else
        class="favorite-strip"
        :games="favoriteGames"
        view-mode="grid"
        action-mode="quick"
        :show-manage-actions="false"
        :launching-game-ids="actions.launchingGameIds.value"
        @launch="actions.launchGame"
        @toggle-favorite="actions.toggleFavorite"
      />
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
    font-size: 14px;
    font-weight: 700;
    line-height: 1.2;
  }

  .section-title svg {
    width: 15px;
    height: 15px;
    color: var(--accent-strong);
  }

  .link-button {
    border: 0;
    background: transparent;
    color: var(--text-muted);
    font-size: 12px;
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
</style>
