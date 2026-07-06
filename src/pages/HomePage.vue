<script setup lang="ts">
  import { computed, nextTick, onBeforeUnmount, onMounted, ref, watch } from 'vue'
  import { RouterLink } from 'vue-router'
  import { Clock3, Library, Play, Search, Star } from '@lucide/vue'
  import { storeToRefs } from 'pinia'
  import GameCollectionSection from '../modules/games/components/GameCollectionSection.vue'
  import GameArtwork from '../modules/games/components/GameArtwork.vue'
  import GameList from '../modules/games/components/GameList.vue'
  import { useGameLibraryActions } from '../modules/games/composables/useGameLibraryActions'
  import { useGamesStore } from '../modules/games/stores/games'
  import type { Game } from '../modules/games/types/game'
  import { routeNames } from '../router/routeNames'
  import BaseButton from '../shared/components/BaseButton.vue'
  import IconButton from '../shared/components/IconButton.vue'

  const gamesStore = useGamesStore()
  const { games, searchText } = storeToRefs(gamesStore)
  const actions = useGameLibraryActions()

  const visibleGames = computed(() => gamesStore.filteredGames)
  const favoriteGames = computed(() => games.value.filter((game) => game.favorite))
  const recentGames = computed(() =>
    games.value
      .filter((game) => game.lastPlayTime)
      .slice()
      .sort((left, right) => (right.lastPlayTime ?? 0) - (left.lastPlayTime ?? 0))
  )
  const hasSearch = computed(() => searchText.value.trim() !== '')
  const pageContent = ref<HTMLElement | null>(null)
  const favoriteColumns = ref(4)
  const recentPreviewGames = computed(() => recentGames.value.slice(0, 5))
  const favoritePreviewGames = computed(() => favoriteGames.value.slice(0, favoriteColumns.value))
  const libraryPreviewGames = computed(() => visibleGames.value.slice(0, 5))
  let pageContentObserver: ResizeObserver | null = null

  onMounted(() => {
    gamesStore.setFilter('all')
    void nextTick(() => {
      updateFavoriteColumns()
      if (!pageContent.value) return
      pageContentObserver = new ResizeObserver(updateFavoriteColumns)
      pageContentObserver.observe(pageContent.value)
    })
  })

  watch(favoriteGames, () => {
    void nextTick(updateFavoriteColumns)
  })

  onBeforeUnmount(() => {
    pageContentObserver?.disconnect()
    pageContentObserver = null
  })

  function getFavoriteColumnCount(width: number) {
    const minimumCardWidth = 150
    const gap = 12
    const columns = Math.floor((width + gap) / (minimumCardWidth + gap))
    return Math.min(10, Math.max(2, columns))
  }

  function updateFavoriteColumns() {
    const width = pageContent.value?.clientWidth ?? 0
    if (!width) return
    favoriteColumns.value = getFavoriteColumnCount(width)
  }

  function getExeFileName(game: Game) {
    return game.exePath.split(/[\\/]/).pop() ?? game.exePath
  }

  function formatLastPlayFull(game: Game) {
    if (!game.lastPlayTime) return '无启动记录'
    return new Intl.DateTimeFormat('zh-CN', {
      year: 'numeric',
      month: '2-digit',
      day: '2-digit',
      hour: '2-digit',
      minute: '2-digit'
    }).format(new Date(game.lastPlayTime))
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

  <div v-else ref="pageContent" class="home-page">
    <section v-if="recentPreviewGames.length > 0" class="library-section library-section--recent">
      <div class="section-heading">
        <h2 class="section-title">
          <Clock3 :size="14" />
          最近游玩
        </h2>
        <RouterLink class="link-button" :to="{ name: routeNames.recent }">查看更多</RouterLink>
      </div>

      <div class="home-list">
        <article v-for="game in recentPreviewGames" :key="game.id" class="home-row">
          <GameArtwork :game="game" variant="list" />
          <div class="home-row__content">
            <h3>{{ game.name }}</h3>
            <p>{{ getExeFileName(game) }}</p>
          </div>
          <span class="home-row__time">{{ formatLastPlayFull(game) }}</span>
          <span class="home-row__meta">{{ game.playCount }}</span>
          <BaseButton
            class="home-row__launch"
            size="sm"
            variant="primary"
            :disabled="actions.launchingGameIds.value.includes(game.id)"
            @click="actions.launchGame(game)"
          >
            <template #icon><Play :size="14" /></template>
            启动
          </BaseButton>
        </article>
      </div>
    </section>

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
        class="home-favorite-grid"
        :style="{ '--favorite-columns': favoriteColumns }"
        :games="favoritePreviewGames"
        view-mode="grid"
        action-mode="quick"
        :show-manage-actions="false"
        :launching-game-ids="actions.launchingGameIds.value"
        @launch="actions.launchGame"
        @toggle-favorite="actions.toggleFavorite"
      />
    </section>

    <section class="library-section library-section--library">
      <div class="section-heading">
        <h2 class="section-title">
          <Library :size="14" />
          游戏库
        </h2>
        <RouterLink class="link-button" :to="{ name: routeNames.games }">查看全部</RouterLink>
      </div>

      <div v-if="libraryPreviewGames.length === 0" class="section-empty">
        <Library :size="18" />
        <span>添加或扫描目录后，游戏会显示在这里。</span>
      </div>
      <div v-else class="home-list">
        <article v-for="game in libraryPreviewGames" :key="game.id" class="home-row">
          <GameArtwork :game="game" variant="list" />
          <div class="home-row__content">
            <h3>{{ game.name }}</h3>
            <p>{{ getExeFileName(game) }}</p>
          </div>
          <span class="home-row__time">{{ formatLastPlayFull(game) }}</span>
          <span class="home-row__meta">{{ game.playCount }}</span>
          <IconButton
            label="启动游戏"
            variant="active"
            :disabled="actions.launchingGameIds.value.includes(game.id)"
            @click="actions.launchGame(game)"
          >
            <Play :size="15" />
          </IconButton>
        </article>
      </div>
    </section>
  </div>
</template>

<style scoped>
  .home-page {
    display: grid;
    gap: 24px;
    min-width: 0;
  }

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

  .home-list {
    overflow: hidden;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.028);
  }

  .home-row {
    display: grid;
    grid-template-columns: 30px minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
    min-height: 50px;
    padding: 9px 10px;
  }

  .home-row + .home-row {
    border-top: 1px solid rgba(255, 255, 255, 0.065);
  }

  .home-row:hover {
    background: rgba(255, 255, 255, 0.035);
  }

  .home-row__content {
    display: grid;
    min-width: 0;
    gap: 3px;
  }

  .home-row__content h3 {
    overflow: hidden;
    margin: 0;
    color: var(--text);
    font-size: var(--font-size-sm);
    line-height: 1.25;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .home-row__content p,
  .home-row__time,
  .home-row__meta {
    overflow: hidden;
    margin: 0;
    color: var(--text-muted);
    font-size: var(--font-size-xs);
    line-height: 1.25;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .home-row__time {
    min-width: 150px;
    font-weight: 600;
  }

  .home-row__meta {
    min-width: 54px;
    color: rgba(246, 243, 248, 0.72);
    font-weight: 600;
  }

  .home-row__launch {
    min-width: 70px;
    box-shadow: 0 8px 18px rgba(73, 51, 180, 0.28);
  }

  .library-section--recent .home-row,
  .library-section--library .home-row {
    grid-template-columns: 30px minmax(180px, 2fr) minmax(150px, 1fr) minmax(54px, 0.45fr) auto;
    column-gap: 18px;
  }

  .home-row :deep(.icon-button) {
    width: 28px;
    min-width: 28px;
    height: 28px;
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

  @media (max-width: 720px) {
    .section-heading {
      align-items: flex-start;
      flex-direction: column;
    }
  }
</style>
