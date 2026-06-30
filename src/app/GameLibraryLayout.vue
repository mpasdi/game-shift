<script setup lang="ts">
  import { computed, onMounted, provide, ref } from 'vue'
  import { RouterLink, RouterView, useRoute } from 'vue-router'
  import { storeToRefs } from 'pinia'
  import { open as openDialog } from '@tauri-apps/plugin-dialog'
  import { Clock3, FolderSearch, Home, Library, Plus, RefreshCw, Search, Settings, Sparkles, Star } from '@lucide/vue'
  import AppShell from './AppShell.vue'
  import AddGameDialog from '../modules/games/components/AddGameDialog.vue'
  import RemoveGameDialog from '../modules/games/components/RemoveGameDialog.vue'
  import ScanResultsDialog from '../modules/games/components/ScanResultsDialog.vue'
  import { gameLibraryActionsKey } from '../modules/games/composables/useGameLibraryActions'
  import type { GameViewMode } from '../modules/games/composables/useGameLibraryActions'
  import { useGamesStore } from '../modules/games/stores/games'
  import type {
    CreateGamePayload,
    Game,
    GameFilter,
    ScanCandidate,
    UpdateGamePayload
  } from '../modules/games/types/game'
  import { routeNames } from '../router/routeNames'
  import BaseButton from '../shared/components/BaseButton.vue'
  import EmptyState from '../shared/components/EmptyState.vue'
  import IconButton from '../shared/components/IconButton.vue'
  import TextField from '../shared/components/TextField.vue'

  interface NavItem {
    name: string
    filter?: GameFilter
    label: string
    icon: unknown
  }

  const route = useRoute()
  const gamesStore = useGamesStore()
  const { games, searchText, isLoading, launchingGameIds, errorMessage } = storeToRefs(gamesStore)
  const viewMode = ref<GameViewMode>('list')
  const isGameDialogOpen = ref(false)
  const editingGame = ref<Game | null>(null)
  const removingGame = ref<Game | null>(null)
  const isScanResultsOpen = ref(false)
  const scanCandidates = ref<ScanCandidate[]>([])
  const scanErrorMessage = ref<string | null>(null)
  const isImportingScanResults = ref(false)

  const dialogMode = computed(() => (editingGame.value ? 'edit' : 'create'))
  const shouldShowEmptyLibrary = computed(() => games.value.length === 0 && route.name !== 'settings')

  const navItems: NavItem[] = [
    { name: routeNames.home, filter: 'all', label: '首页', icon: Home },
    { name: routeNames.games, filter: 'all', label: '全部游戏', icon: Library },
    { name: routeNames.favorites, filter: 'favorite', label: '收藏游戏', icon: Star },
    { name: routeNames.recent, filter: 'recent', label: '最近游玩', icon: Clock3 }
  ]

  onMounted(() => {
    void gamesStore.loadGames()
  })

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

  async function launchGame(game: Game) {
    await gamesStore.launchGame(game.id)
  }

  async function scanDirectory() {
    const selected = await openDialog({
      multiple: false,
      directory: true
    })

    if (typeof selected !== 'string') return

    scanErrorMessage.value = null
    scanCandidates.value = []
    try {
      scanCandidates.value = await gamesStore.scanGames(selected)
      isScanResultsOpen.value = true
    } catch (error) {
      scanErrorMessage.value = error instanceof Error ? error.message : String(error)
      isScanResultsOpen.value = true
    }
  }

  function closeScanResultsDialog() {
    if (isImportingScanResults.value) return
    isScanResultsOpen.value = false
    scanErrorMessage.value = null
  }

  async function importScanCandidates(candidates: ScanCandidate[]) {
    isImportingScanResults.value = true
    scanErrorMessage.value = null

    try {
      for (const candidate of candidates) {
        await gamesStore.createGame({
          name: candidate.name,
          exePath: candidate.exePath,
          workDir: candidate.folderPath,
          args: null
        })
      }
      isScanResultsOpen.value = false
      scanErrorMessage.value = null
      await gamesStore.loadGames()
    } catch (error) {
      scanErrorMessage.value = error instanceof Error ? error.message : String(error)
    } finally {
      isImportingScanResults.value = false
    }
  }

  function setViewMode(nextViewMode: GameViewMode) {
    viewMode.value = nextViewMode
  }

  function isUpdateGamePayload(payload: CreateGamePayload | UpdateGamePayload): payload is UpdateGamePayload {
    return 'id' in payload
  }

  provide(gameLibraryActionsKey, {
    viewMode,
    launchingGameIds,
    setViewMode,
    openEditGameDialog,
    openRemoveGameDialog,
    launchGame,
    toggleFavorite
  })
</script>

<template>
  <AppShell>
    <template #nav>
      <nav class="side-nav" aria-label="游戏库导航">
        <RouterLink
          v-for="item in navItems"
          :key="item.name"
          v-slot="{ isExactActive, href, navigate }"
          :to="{ name: item.name }"
          custom
        >
          <a class="side-nav__item" :class="{ 'side-nav__item--active': isExactActive }" :href="href" @click="navigate">
            <component :is="item.icon" :size="16" />
            <span>{{ item.label }}</span>
            <small v-if="item.name !== routeNames.home && item.filter">
              {{ gamesStore.countByFilter(item.filter) }}
            </small>
          </a>
        </RouterLink>
      </nav>
    </template>

    <template #summary>
      <div class="play-summary">
        <p>游戏库</p>
        <strong>{{ games.length }}</strong>
        <span>本地记录</span>
      </div>
    </template>

    <template #settings>
      <RouterLink v-slot="{ isActive, href, navigate }" :to="{ name: routeNames.settings }" custom>
        <a class="settings-entry" :class="{ 'settings-entry--active': isActive }" :href="href" @click="navigate">
          <Settings :size="16" />
          <span>设置</span>
        </a>
      </RouterLink>
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
        <BaseButton variant="secondary" :loading="gamesStore.isScanning" @click="scanDirectory">
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
        v-else-if="shouldShowEmptyLibrary"
        label="空游戏库"
        eyebrow="尚未导入游戏"
        title="从第一个启动程序开始"
        description="选择本地 .exe 后，Game Shift 会保存到本地数据库。也可以直接扫描目录，批量导入可启动的 .exe。"
      >
        <template #icon><Sparkles :size="15" /></template>
        <template #actions>
          <BaseButton variant="primary" @click="openCreateGameDialog">
            <template #icon><Plus :size="17" /></template>
            手动添加
          </BaseButton>
          <BaseButton variant="secondary" :loading="gamesStore.isScanning" @click="scanDirectory">
            <template #icon><FolderSearch :size="17" /></template>
            扫描目录
          </BaseButton>
        </template>
      </EmptyState>

      <RouterView v-else />
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

  <ScanResultsDialog
    :open="isScanResultsOpen"
    :candidates="scanCandidates"
    :importing="isImportingScanResults"
    :error-message="scanErrorMessage"
    @close="closeScanResultsDialog"
    @import="importScanCandidates"
  />
</template>

<style scoped>
  .side-nav {
    display: grid;
    gap: 6px;
    margin-top: 28px;
  }

  .side-nav__item,
  .settings-entry {
    text-decoration: none;
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
    font-size: var(--font-size-xs);
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
    font-size: var(--font-size-sm);
  }

  .play-summary strong {
    color: var(--text);
    font-size: 28px;
    line-height: 1;
  }

  .settings-entry {
    display: inline-flex;
    gap: 10px;
    align-items: center;
    min-height: 38px;
    margin-top: 14px;
    border: 1px solid transparent;
    border-radius: 8px;
    background: transparent;
    color: var(--text-muted);
    padding: 0 10px;
  }

  .settings-entry:hover,
  .settings-entry--active {
    border-color: var(--accent-border);
    background: var(--accent-soft);
    color: var(--text);
  }

  .top-search {
    min-width: 0;
  }

  .top-search :deep(.text-field) {
    width: min(720px, 100%);
  }

  .top-actions {
    display: flex;
    gap: 10px;
    align-items: center;
  }

  .library-shell {
    display: grid;
    width: 100%;
    gap: 24px;
  }

  @media (max-width: 960px) {
    .side-nav__item span,
    .side-nav__item small,
    .play-summary,
    .settings-entry span {
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

    .top-actions {
      width: 100%;
      align-items: stretch;
      flex-direction: column;
    }

    .top-actions :deep(.base-button) {
      width: 100%;
    }
  }
</style>
