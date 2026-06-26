<script setup lang="ts">
  import { computed, onMounted, ref } from 'vue'
  import { storeToRefs } from 'pinia'
  import {
    Clock3,
    Folder,
    FolderSearch,
    Grid2X2,
    Heart,
    Home,
    LayoutList,
    Library,
    Plus,
    RefreshCw,
    Search,
    Sparkles
  } from '@lucide/vue'
  import AppShell from '../app/AppShell.vue'
  import AddGameDialog from '../modules/games/components/AddGameDialog.vue'
  import GameList from '../modules/games/components/GameList.vue'
  import RemoveGameDialog from '../modules/games/components/RemoveGameDialog.vue'
  import { useGamesStore } from '../modules/games/stores/games'
  import type { CreateGamePayload, Game, GameFilter, UpdateGamePayload } from '../modules/games/types/game'
  import BaseButton from '../shared/components/BaseButton.vue'
  import EmptyState from '../shared/components/EmptyState.vue'
  import IconButton from '../shared/components/IconButton.vue'
  import TextField from '../shared/components/TextField.vue'

  type NavId = 'home' | 'all' | 'favorite' | 'recent'

  interface NavItem {
    id: NavId
    filter: GameFilter
    label: string
    icon: unknown
  }

  const gamesStore = useGamesStore()
  const { games, searchText, activeFilter, isLoading, errorMessage } = storeToRefs(gamesStore)
  const viewMode = ref<'grid' | 'list'>('list')
  const activeNav = ref<NavId>('home')
  const isGameDialogOpen = ref(false)
  const editingGame = ref<Game | null>(null)
  const removingGame = ref<Game | null>(null)

  const visibleGames = computed(() => gamesStore.filteredGames)
  const favoriteGames = computed(() => games.value.filter((game) => game.favorite))
  const recentGamesCount = computed(() => games.value.filter((game) => game.lastPlayTime).length)
  const dialogMode = computed(() => (editingGame.value ? 'edit' : 'create'))
  const hasSearch = computed(() => searchText.value.trim() !== '')
  const isHome = computed(() => activeNav.value === 'home' && !hasSearch.value)
  const sectionTitle = computed(() => {
    if (hasSearch.value) return '搜索结果'
    if (activeFilter.value === 'favorite') return '收藏游戏'
    if (activeFilter.value === 'recent') return '最近游玩'
    return '全部游戏'
  })
  const sectionMeta = computed(() => {
    if (hasSearch.value) return `${visibleGames.value.length} 个匹配`
    if (activeFilter.value === 'favorite') return `${favoriteGames.value.length} 个收藏`
    if (activeFilter.value === 'recent') return `${recentGamesCount.value} 条记录`
    return `${games.value.length} 个游戏`
  })

  const navItems: NavItem[] = [
    { id: 'home', filter: 'all', label: '首页', icon: Home },
    { id: 'all', filter: 'all', label: '全部游戏', icon: Library },
    { id: 'favorite', filter: 'favorite', label: '收藏游戏', icon: Heart },
    { id: 'recent', filter: 'recent', label: '最近游玩', icon: Clock3 }
  ]

  onMounted(() => {
    void gamesStore.loadGames()
  })

  function setNav(item: NavItem) {
    activeNav.value = item.id
    gamesStore.setFilter(item.filter)
  }

  function openCreateGameDialog() {
    editingGame.value = null
    isGameDialogOpen.value = true
  }

  function openEditGameDialog(game: Game) {
    editingGame.value = game
    isGameDialogOpen.value = true
  }

  function closeGameDialog() {
    isGameDialogOpen.value = false
    editingGame.value = null
  }

  async function submitGame(payload: CreateGamePayload | UpdateGamePayload) {
    if (isUpdateGamePayload(payload)) {
      await gamesStore.updateGame(payload)
    } else {
      await gamesStore.createGame(payload)
    }
    closeGameDialog()
  }

  function openRemoveGameDialog(game: Game) {
    removingGame.value = game
  }

  function closeRemoveGameDialog() {
    removingGame.value = null
  }

  async function confirmRemoveGame() {
    if (!removingGame.value) return

    await gamesStore.deleteGame(removingGame.value.id)
    closeRemoveGameDialog()
  }

  async function toggleFavorite(game: Game) {
    await gamesStore.updateGame({
      id: game.id,
      name: game.name,
      exePath: game.exePath,
      workDir: game.workDir,
      args: game.args,
      favorite: !game.favorite
    })
  }

  function isUpdateGamePayload(payload: CreateGamePayload | UpdateGamePayload): payload is UpdateGamePayload {
    return 'id' in payload
  }
</script>

<template>
  <AppShell>
    <template #nav>
      <nav class="side-nav" aria-label="游戏库筛选">
        <button
          v-for="item in navItems"
          :key="item.id"
          class="side-nav__item"
          :class="{ 'side-nav__item--active': activeNav === item.id && !hasSearch }"
          type="button"
          @click="setNav(item)"
        >
          <component :is="item.icon" :size="16" />
          <span>{{ item.label }}</span>
          <small v-if="item.id !== 'home'">{{ gamesStore.countByFilter(item.filter) }}</small>
        </button>

        <button class="side-nav__item" type="button">
          <Folder :size="16" />
          <span>分类</span>
        </button>
      </nav>
    </template>

    <template #summary>
      <div class="play-summary">
        <p>游戏库</p>
        <strong>{{ games.length }}</strong>
        <span>本地记录</span>
      </div>
    </template>

    <template #toolbar>
      <div class="top-search">
        <TextField id="game-search" v-model="searchText" type="search" placeholder="搜索游戏 / 启动程序 / 路径">
          <template #icon><Search :size="17" /></template>
        </TextField>
      </div>

      <div class="top-actions">
        <BaseButton variant="primary" @click="openCreateGameDialog">
          <template #icon><Plus :size="16" /></template>
          添加游戏
        </BaseButton>
        <BaseButton variant="secondary">
          <template #icon><FolderSearch :size="16" /></template>
          扫描目录
        </BaseButton>
        <IconButton label="刷新游戏库" :variant="isLoading ? 'active' : 'plain'" @click="gamesStore.refreshGames()">
          <RefreshCw :size="17" />
        </IconButton>
      </div>
    </template>

    <section class="library-shell" aria-label="游戏库">
      <EmptyState
        v-if="isLoading"
        label="正在加载游戏库"
        eyebrow="正在加载"
        title="正在读取本地游戏库"
        description="Game Shift 正在初始化本地数据库并加载游戏列表。"
      >
        <template #icon><Sparkles :size="15" /></template>
      </EmptyState>

      <EmptyState
        v-else-if="errorMessage"
        label="游戏库加载失败"
        eyebrow="加载失败"
        title="无法读取本地游戏库"
        :description="errorMessage"
      >
        <template #icon><Sparkles :size="15" /></template>
        <template #actions>
          <BaseButton variant="secondary" @click="gamesStore.loadGames()">重试</BaseButton>
        </template>
      </EmptyState>

      <EmptyState
        v-else-if="games.length === 0"
        label="空游戏库"
        eyebrow="尚未导入游戏"
        title="从第一个启动程序开始"
        description="选择本地 .exe 后，Game Shift 会保存到本地数据库。目录扫描将在后续阶段接入。"
      >
        <template #icon><Sparkles :size="15" /></template>
        <template #actions>
          <BaseButton variant="primary" @click="openCreateGameDialog">
            <template #icon><Plus :size="17" /></template>
            手动添加
          </BaseButton>
          <BaseButton variant="secondary">
            <template #icon><FolderSearch :size="17" /></template>
            扫描目录
          </BaseButton>
        </template>
      </EmptyState>

      <template v-else>
        <section v-if="isHome" class="library-section library-section--favorites">
          <div class="section-heading">
            <div>
              <h2 class="section-title">
                <Heart :size="14" />
                收藏游戏
              </h2>
            </div>
            <button class="link-button" type="button" @click="setNav(navItems[2])">查看更多</button>
          </div>

          <div v-if="favoriteGames.length === 0" class="section-empty">
            <Heart :size="18" />
            <span>点击游戏卡片上的爱心，把常玩的游戏放到这里。</span>
          </div>
          <GameList
            v-else
            class="favorite-strip"
            :games="favoriteGames"
            view-mode="grid"
            action-mode="quick"
            :show-manage-actions="false"
            @toggle-favorite="toggleFavorite"
          />
        </section>

        <section class="library-section library-section--all">
          <div class="section-heading">
            <div>
              <h2 class="section-title">
                <Library :size="14" />
                <span>{{ sectionTitle }}</span>
              </h2>
            </div>
            <div class="section-actions">
              <span>{{ sectionMeta }}</span>
              <div class="segmented" aria-label="视图切换">
                <IconButton
                  label="网格视图"
                  :variant="viewMode === 'grid' ? 'active' : 'plain'"
                  @click="viewMode = 'grid'"
                >
                  <Grid2X2 :size="17" />
                </IconButton>
                <IconButton
                  label="列表视图"
                  :variant="viewMode === 'list' ? 'active' : 'plain'"
                  @click="viewMode = 'list'"
                >
                  <LayoutList :size="17" />
                </IconButton>
              </div>
            </div>
          </div>

          <EmptyState
            v-if="visibleGames.length === 0"
            label="没有匹配的游戏"
            eyebrow="无结果"
            :title="activeFilter === 'recent' ? '还没有最近游玩记录' : '没有找到匹配的游戏'"
            :description="
              activeFilter === 'recent'
                ? '启动游戏功能完成后，这里会按最近游玩时间展示。'
                : '可以调整搜索关键词，或手动添加新的启动程序。'
            "
          >
            <template #icon><Search :size="15" /></template>
          </EmptyState>

          <GameList
            v-else
            :class="{ 'favorite-grid': activeFilter === 'favorite' && viewMode === 'grid' }"
            :games="visibleGames"
            :view-mode="viewMode"
            :action-mode="activeFilter === 'favorite' && viewMode === 'grid' ? 'quick' : 'full'"
            :show-manage-actions="activeFilter !== 'favorite'"
            @edit="openEditGameDialog"
            @toggle-favorite="toggleFavorite"
            @remove="openRemoveGameDialog"
          />
        </section>
      </template>
    </section>
  </AppShell>

  <AddGameDialog
    :open="isGameDialogOpen"
    :mode="dialogMode"
    :game="editingGame"
    :saving="gamesStore.isSaving"
    :error-message="gamesStore.errorMessage"
    @close="closeGameDialog"
    @submit="submitGame"
  />

  <RemoveGameDialog
    :open="Boolean(removingGame)"
    :game="removingGame"
    :deleting="gamesStore.isSaving"
    :error-message="gamesStore.errorMessage"
    @close="closeRemoveGameDialog"
    @confirm="confirmRemoveGame"
  />
</template>

<style scoped>
  .side-nav {
    display: grid;
    gap: 6px;
    margin-top: 28px;
  }

  .side-nav__item,
  .link-button {
    border: 0;
    background: transparent;
    color: inherit;
  }

  .side-nav__item {
    display: grid;
    grid-template-columns: 18px minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
    min-height: 38px;
    border: 1px solid transparent;
    border-radius: 8px;
    color: var(--text-muted);
    padding: 0 10px;
    text-align: left;
    transition:
      background 160ms ease,
      border-color 160ms ease,
      color 160ms ease;
  }

  .side-nav__item:hover,
  .side-nav__item--active {
    border-color: var(--accent-border);
    background: var(--accent-soft);
    color: var(--text);
  }

  .side-nav__item small {
    color: var(--text-subtle);
    font-size: 11px;
  }

  .play-summary {
    display: grid;
    gap: 4px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    padding: 14px;
  }

  .play-summary p,
  .play-summary span {
    margin: 0;
    color: var(--text-muted);
    font-size: 12px;
  }

  .play-summary strong {
    color: var(--text);
    font-size: 28px;
    line-height: 1;
  }

  .top-search {
    min-width: 0;
  }

  .top-search :deep(.text-field) {
    width: min(720px, 100%);
  }

  .top-actions,
  .empty-actions,
  .section-actions {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .library-shell {
    display: grid;
    width: 100%;
    gap: 24px;
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

  .section-heading h2 {
    margin: 0;
    color: var(--text);
    font-size: 14px;
    font-weight: 700;
    line-height: 1.2;
  }

  .section-title {
    display: inline-flex;
    gap: 7px;
    align-items: center;
  }

  .section-title svg {
    width: 15px;
    height: 15px;
    color: var(--accent-strong);
  }

  .section-actions > span,
  .section-heading > span,
  .link-button {
    color: var(--text-muted);
    font-size: 12px;
  }

  .link-button {
    padding: 5px 0;
  }

  .link-button:hover {
    color: var(--accent-strong);
  }

  .section-empty {
    display: flex;
    gap: 10px;
    align-items: center;
    min-height: 70px;
    border: 1px dashed var(--border-strong);
    border-radius: 8px;
    background: var(--surface);
    color: var(--text-muted);
    padding: 18px;
  }

  .section-empty svg {
    color: var(--text-subtle);
  }

  .segmented {
    display: inline-flex;
    gap: 6px;
    height: 36px;
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

  @media (max-width: 960px) {
    .side-nav__item span,
    .side-nav__item small,
    .play-summary {
      display: none;
    }

    .side-nav__item {
      grid-template-columns: 1fr;
      justify-items: center;
    }
  }

  @media (max-width: 720px) {
    .side-nav {
      grid-template-columns: repeat(5, minmax(0, 1fr));
      margin-top: 16px;
    }

    .side-nav__item {
      min-height: 42px;
    }

    .top-actions,
    .section-heading {
      align-items: stretch;
      flex-direction: column;
    }

    .top-actions,
    .empty-actions,
    .section-actions {
      width: 100%;
    }

    .top-actions :deep(.base-button),
    .empty-actions :deep(.base-button) {
      width: 100%;
    }
  }
</style>
