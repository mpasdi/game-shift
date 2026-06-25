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
    Search,
    Sparkles
  } from '@lucide/vue'
  import { useGamesStore } from './modules/games/stores/games'
  import BaseButton from './shared/components/BaseButton.vue'
  import EmptyState from './shared/components/EmptyState.vue'
  import IconButton from './shared/components/IconButton.vue'
  import TextField from './shared/components/TextField.vue'

  const gamesStore = useGamesStore()
  const { games, searchText, activeFilter, isLoading, errorMessage } = storeToRefs(gamesStore)
  const viewMode = ref<'grid' | 'list'>('grid')

  const visibleGames = computed(() => gamesStore.filteredGames)
  const filterItems = [
    { key: 'all', label: '全部', icon: Library },
    { key: 'favorite', label: '收藏', icon: Heart },
    { key: 'recent', label: '最近', icon: Clock3 }
  ] as const

  onMounted(() => {
    void gamesStore.loadGames()
  })
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
          <BaseButton variant="secondary">
            <template #icon><FolderSearch :size="17" /></template>
            扫描目录
          </BaseButton>
          <BaseButton variant="primary">
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
          <BaseButton variant="secondary">
            <template #icon><Plus :size="17" /></template>
            手动添加
          </BaseButton>
        </template>
      </EmptyState>

      <section v-else class="game-area" :class="viewMode">
        <article v-for="game in visibleGames" :key="game.id" class="game-item">
          <div class="game-icon">{{ game.name.slice(0, 1).toUpperCase() }}</div>
          <div class="game-meta">
            <h2>{{ game.name }}</h2>
            <p>{{ game.exePath }}</p>
          </div>
          <BaseButton variant="primary" size="sm">启动</BaseButton>
        </article>
      </section>
    </section>
  </main>
</template>
