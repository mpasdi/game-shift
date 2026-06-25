<script setup lang="ts">
  import { computed, onMounted, ref } from 'vue'
  import { storeToRefs } from 'pinia'
  import {
    Clock3,
    FolderSearch,
    Gamepad2,
    Grid2X2,
    Heart,
    LayoutList,
    Library,
    Plus,
    RefreshCw,
    Search,
    Sparkles
  } from '@lucide/vue'
  import AddGameDialog from './modules/games/components/AddGameDialog.vue'
  import GameList from './modules/games/components/GameList.vue'
  import { useGamesStore } from './modules/games/stores/games'
  import type { CreateGamePayload, Game, UpdateGamePayload } from './modules/games/types/game'
  import BaseButton from './shared/components/BaseButton.vue'
  import EmptyState from './shared/components/EmptyState.vue'
  import IconButton from './shared/components/IconButton.vue'
  import TextField from './shared/components/TextField.vue'

  const gamesStore = useGamesStore()
  const { games, searchText, activeFilter, isLoading, errorMessage } = storeToRefs(gamesStore)
  const viewMode = ref<'grid' | 'list'>('grid')
  const isGameDialogOpen = ref(false)
  const editingGame = ref<Game | null>(null)

  const visibleGames = computed(() => gamesStore.filteredGames)
  const dialogMode = computed(() => (editingGame.value ? 'edit' : 'create'))
  const hasActiveSearchOrFilter = computed(() => searchText.value.trim() !== '' || activeFilter.value !== 'all')
  const filterItems = [
    { key: 'all', label: '全部', icon: Library },
    { key: 'favorite', label: '收藏', icon: Heart },
    { key: 'recent', label: '最近', icon: Clock3 }
  ] as const

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

  function isUpdateGamePayload(payload: CreateGamePayload | UpdateGamePayload): payload is UpdateGamePayload {
    return 'id' in payload
  }
</script>

<template>
  <main class="app-shell">
    <aside class="sidebar" aria-label="游戏筛选">
      <div class="brand-block">
        <div class="brand-mark"><Gamepad2 :size="22" /></div>
        <div>
          <h1>Game Shift</h1>
          <p>本地游戏启动器</p>
        </div>
      </div>

      <nav class="filter-list">
        <button
          v-for="item in filterItems"
          :key="item.key"
          class="filter-button"
          :class="{ active: activeFilter === item.key }"
          type="button"
          @click="gamesStore.setFilter(item.key)"
        >
          <span class="button-label">
            <component :is="item.icon" :size="17" />
            {{ item.label }}
          </span>
          <span class="filter-count">{{ gamesStore.countByFilter(item.key) }}</span>
        </button>
      </nav>
    </aside>

    <section class="workspace" aria-label="游戏库">
      <header class="toolbar">
        <TextField id="game-search" v-model="searchText" type="search" placeholder="输入游戏名称或 exe 文件名">
          <template #icon><Search :size="17" /></template>
        </TextField>

        <div class="toolbar-actions">
          <div class="segmented" aria-label="视图切换">
            <IconButton label="网格视图" :variant="viewMode === 'grid' ? 'active' : 'plain'" @click="viewMode = 'grid'">
              <Grid2X2 :size="17" />
            </IconButton>
            <IconButton label="列表视图" :variant="viewMode === 'list' ? 'active' : 'plain'" @click="viewMode = 'list'">
              <LayoutList :size="17" />
            </IconButton>
          </div>
          <BaseButton variant="secondary" :loading="isLoading" @click="gamesStore.refreshGames()">
            <template #icon><RefreshCw :size="17" /></template>
            刷新
          </BaseButton>
          <BaseButton variant="secondary">
            <template #icon><FolderSearch :size="17" /></template>
            扫描目录
          </BaseButton>
          <BaseButton variant="primary" @click="openCreateGameDialog">
            <template #icon><Plus :size="17" /></template>
            手动添加
          </BaseButton>
        </div>
      </header>

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
        title="从扫描本地目录开始建立你的游戏库"
        description="扫描结果会先进入候选列表，确认后才会写入本地数据库。"
      >
        <template #icon><Sparkles :size="15" /></template>
        <template #actions>
          <BaseButton variant="primary">
            <template #icon><FolderSearch :size="17" /></template>
            扫描目录
          </BaseButton>
          <BaseButton variant="secondary" @click="openCreateGameDialog">
            <template #icon><Plus :size="17" /></template>
            手动添加
          </BaseButton>
        </template>
      </EmptyState>

      <EmptyState
        v-else-if="visibleGames.length === 0"
        label="没有匹配的游戏"
        eyebrow="无结果"
        title="没有找到匹配的游戏"
        :description="hasActiveSearchOrFilter ? '可以调整搜索关键词或切换左侧筛选条件。' : '当前游戏库暂无可展示内容。'"
      >
        <template #icon><Sparkles :size="15" /></template>
      </EmptyState>

      <GameList v-else :games="visibleGames" :view-mode="viewMode" @edit="openEditGameDialog" />
    </section>

    <AddGameDialog
      :open="isGameDialogOpen"
      :mode="dialogMode"
      :game="editingGame"
      :saving="gamesStore.isSaving"
      :error-message="gamesStore.errorMessage"
      @close="closeGameDialog"
      @submit="submitGame"
    />
  </main>
</template>
