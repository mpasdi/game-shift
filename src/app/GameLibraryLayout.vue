<script setup lang="ts">
  import { computed, onMounted, provide, ref } from 'vue'
  import { RouterLink, RouterView, useRoute } from 'vue-router'
  import { storeToRefs } from 'pinia'
  import { open as openDialog } from '@tauri-apps/plugin-dialog'
  import { Clock3, FolderSearch, Home, Library, Plus, Search, Settings, Sparkles, Star } from '@lucide/vue'
  import AppShell from './AppShell.vue'
  import AddGameDialog from '../modules/games/components/AddGameDialog.vue'
  import EmptyLibraryState from '../modules/games/components/EmptyLibraryState.vue'
  import RemoveGameDialog from '../modules/games/components/RemoveGameDialog.vue'
  import ScanResultsDialog from '../modules/games/components/ScanResultsDialog.vue'
  import { gameLibraryActionsKey } from '../modules/games/composables/useGameLibraryActions'
  import type { GameViewMode } from '../modules/games/composables/useGameLibraryActions'
  import { useGamesStore } from '../modules/games/stores/games'
  import type { CreateGamePayload, Game, ScanCandidate, UpdateGamePayload } from '../modules/games/types/game'
  import { routeNames } from '../router/routeNames'
  import BaseButton from '../shared/components/BaseButton.vue'
  import EmptyState from '../shared/components/EmptyState.vue'
  import TextField from '../shared/components/TextField.vue'
  import { getErrorMessage, useToast } from '../shared/composables/useToast'

  interface NavItem {
    name: string
    label: string
    icon: unknown
  }

  const route = useRoute()
  const gamesStore = useGamesStore()
  const toast = useToast()
  const { games, searchText, isLoading, launchingGameIds, libraryErrorMessage } = storeToRefs(gamesStore)
  type ViewModeScope = 'home' | 'games' | 'favorites' | 'recent'

  const viewModes = ref<Record<ViewModeScope, GameViewMode>>({
    home: 'list',
    games: 'list',
    favorites: 'grid',
    recent: 'list'
  })
  const isGameDialogOpen = ref(false)
  const editingGame = ref<Game | null>(null)
  const removingGame = ref<Game | null>(null)
  const isScanResultsOpen = ref(false)
  const scanCandidates = ref<ScanCandidate[]>([])
  const scanErrorMessage = ref<string | null>(null)
  const isImportingScanResults = ref(false)
  const dialogMode = computed(() => (editingGame.value ? 'edit' : 'create'))
  const shouldShowEmptyLibrary = computed(() => games.value.length === 0 && route.name === routeNames.home)
  const shouldShowLibraryToolbar = computed(() => route.name !== routeNames.settings)
  const shouldShowLibraryActions = computed(() =>
    [routeNames.home, routeNames.games].includes(route.name as typeof routeNames.home | typeof routeNames.games)
  )
  const viewModeScope = computed<ViewModeScope>(() => {
    switch (route.name) {
      case routeNames.home:
        return 'home'
      case routeNames.favorites:
        return 'favorites'
      case routeNames.recent:
        return 'recent'
      default:
        return 'games'
    }
  })
  const viewMode = computed(() => viewModes.value[viewModeScope.value])

  const navItems: NavItem[] = [
    { name: routeNames.home, label: '首页', icon: Home },
    { name: routeNames.games, label: '全部游戏', icon: Library },
    { name: routeNames.favorites, label: '收藏游戏', icon: Star },
    { name: routeNames.recent, label: '最近游玩', icon: Clock3 }
  ]

  onMounted(() => {
    void gamesStore.loadGames()
  })

  function openCreateGameDialog() {
    gamesStore.clearErrorMessage()
    editingGame.value = null
    isGameDialogOpen.value = true
  }

  function openEditGameDialog(game: Game) {
    gamesStore.clearErrorMessage()
    editingGame.value = game
    isGameDialogOpen.value = true
  }

  function closeGameDialog() {
    gamesStore.clearErrorMessage()
    isGameDialogOpen.value = false
    editingGame.value = null
  }

  async function submitGame(payload: CreateGamePayload | UpdateGamePayload) {
    const isEditing = isUpdateGamePayload(payload)

    try {
      if (isEditing) {
        await gamesStore.updateGame(payload)
        toast.success({ title: '游戏信息已保存' })
      } else {
        await gamesStore.createGame(payload)
        toast.success({ title: '游戏已添加到库中' })
      }
      closeGameDialog()
    } catch {
      // The dialog keeps the operation error visible next to the form.
    }
  }
  function openRemoveGameDialog(game: Game) {
    gamesStore.clearErrorMessage()
    removingGame.value = game
  }

  function closeRemoveGameDialog() {
    gamesStore.clearErrorMessage()
    removingGame.value = null
  }

  async function confirmRemoveGame() {
    if (!removingGame.value) return

    const gameName = removingGame.value.name

    try {
      await gamesStore.deleteGame(removingGame.value.id)
      closeRemoveGameDialog()
      toast.success({ title: '游戏已从库中移除', description: gameName })
    } catch {
      // The confirmation dialog keeps the operation error visible.
    }
  }
  async function toggleFavorite(game: Game) {
    try {
      await gamesStore.updateGame({
        id: game.id,
        name: game.name,
        exePath: game.exePath,
        workDir: game.workDir,
        args: game.args,
        favorite: !game.favorite
      })
    } catch (error) {
      toast.error({ title: '更新收藏状态失败', description: getErrorMessage(error) })
    }
  }
  async function launchGame(game: Game) {
    try {
      const launchedGame = await gamesStore.launchGame(game.id)
      if (launchedGame) {
        toast.success({ title: '游戏已启动', description: game.name })
      }
    } catch (error) {
      toast.error({ title: '启动游戏失败', description: getErrorMessage(error) })
    }
  }
  async function scanDirectory() {
    try {
      const selected = await openDialog({
        multiple: false,
        directory: true
      })

      if (typeof selected !== 'string') return

      scanErrorMessage.value = null
      scanCandidates.value = []
      scanCandidates.value = await gamesStore.scanGames(selected)
      isScanResultsOpen.value = true
      toast.info({ title: '目录扫描完成', description: `发现 ${scanCandidates.value.length} 个候选程序` })
    } catch (error) {
      scanErrorMessage.value = getErrorMessage(error)
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
      toast.success({ title: '扫描结果已导入', description: `已导入 ${candidates.length} 个游戏` })
    } catch (error) {
      scanErrorMessage.value = getErrorMessage(error)
    } finally {
      isImportingScanResults.value = false
    }
  }
  function setViewMode(nextViewMode: GameViewMode) {
    viewModes.value[viewModeScope.value] = nextViewMode
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
  <AppShell :show-toolbar="shouldShowLibraryToolbar">
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
          </a>
        </RouterLink>
      </nav>
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
        <TextField
          id="game-search"
          v-model="searchText"
          type="search"
          name="game-library-search"
          autocomplete="off"
          placeholder="搜索游戏 / 启动程序"
        >
          <template #icon><Search :size="17" /></template>
        </TextField>
      </div>

      <div v-if="shouldShowLibraryActions" class="top-actions">
        <BaseButton variant="primary" @click="openCreateGameDialog">
          <template #icon><Plus :size="16" /></template>
          添加游戏
        </BaseButton>
        <BaseButton variant="secondary" :loading="gamesStore.isScanning" @click="scanDirectory">
          <template #icon><FolderSearch :size="16" /></template>
          扫描目录
        </BaseButton>
      </div>
    </template>

    <section class="library-shell" :class="{ 'library-shell--empty': shouldShowEmptyLibrary }" aria-label="游戏库">
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
        v-else-if="libraryErrorMessage"
        label="游戏库加载失败"
        eyebrow="加载失败"
        title="无法读取本地游戏库"
        :description="libraryErrorMessage"
      >
        <template #icon><Sparkles :size="15" /></template>
        <template #actions>
          <BaseButton variant="secondary" @click="gamesStore.loadGames()">重试</BaseButton>
        </template>
      </EmptyState>

      <EmptyLibraryState
        v-else-if="shouldShowEmptyLibrary"
        :scanning="gamesStore.isScanning"
        @add="openCreateGameDialog"
        @scan="scanDirectory"
      />

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
      width 220ms ease,
      min-height 220ms ease,
      padding 220ms ease,
      background 160ms ease,
      border-color 160ms ease,
      color 160ms ease;
  }

  .side-nav__item span {
    overflow: hidden;
    max-width: 120px;
    white-space: nowrap;
    transition:
      max-width 180ms ease,
      opacity 140ms ease,
      transform 140ms ease;
  }

  .side-nav__item:hover {
    border-color: transparent;
    background: var(--accent-soft);
    color: var(--text);
  }

  .side-nav__item--active {
    border-color: transparent;
    background: rgba(124, 92, 255, 0.24);
    color: #ffffff;
  }

  .side-nav__item--active svg {
    color: #c4b5fd;
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

  .settings-entry span {
    overflow: hidden;
    max-width: 120px;
    white-space: nowrap;
    transition:
      max-width 180ms ease,
      opacity 140ms ease,
      transform 140ms ease;
  }

  .settings-entry:hover {
    border-color: transparent;
    background: var(--accent-soft);
    color: var(--text);
  }

  .settings-entry--active {
    border-color: transparent;
    background: rgba(124, 92, 255, 0.24);
    color: #ffffff;
  }

  .settings-entry--active svg {
    color: #c4b5fd;
  }

  .top-search {
    width: min(100%, 380px);
    min-width: 0;
  }

  .top-search :deep(.text-field) {
    width: 100%;
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

  .library-shell--empty {
    min-height: 100%;
  }

  @media (max-width: 960px) {
    .side-nav__item span,
    .settings-entry span {
      position: absolute;
      max-width: 0;
      opacity: 0;
      transform: translateX(-4px);
      pointer-events: none;
    }

    .side-nav__item {
      grid-template-columns: 1fr;
      justify-items: center;
    }

    .side-nav__item svg {
      grid-column: 1;
      grid-row: 1;
    }

    .side-nav__item {
      justify-self: center;
      width: 38px;
      min-height: 38px;
      padding: 0;
    }

    .settings-entry {
      display: grid;
      width: 38px;
      min-width: 38px;
      height: 38px;
      min-height: 38px;
      margin-right: auto;
      margin-left: auto;
      padding: 0;
      gap: 0;
      place-items: center;
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
