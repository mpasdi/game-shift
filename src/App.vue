<script setup lang="ts">
  import { computed, ref } from 'vue'
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

  const gamesStore = useGamesStore()
  const { games, searchText, activeFilter } = storeToRefs(gamesStore)
  const viewMode = ref<'grid' | 'list'>('grid')

  const visibleGames = computed(() => gamesStore.filteredGames)
  const filterItems = [
    { key: 'all', label: '全部', icon: Library },
    { key: 'favorite', label: '收藏', icon: Heart },
    { key: 'recent', label: '最近', icon: Clock3 }
  ] as const
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
        <label class="search-box" for="game-search">
          <Search :size="17" />
          <input id="game-search" v-model="searchText" type="search" placeholder="输入游戏名称或 exe 文件名" />
        </label>

        <div class="toolbar-actions">
          <div class="segmented" aria-label="视图切换">
            <button :class="{ active: viewMode === 'grid' }" type="button" title="网格视图" @click="viewMode = 'grid'">
              <Grid2X2 :size="17" />
            </button>
            <button :class="{ active: viewMode === 'list' }" type="button" title="列表视图" @click="viewMode = 'list'">
              <LayoutList :size="17" />
            </button>
          </div>
          <button class="secondary-action" type="button">
            <FolderSearch :size="17" />
            扫描目录
          </button>
          <button class="primary-action" type="button">
            <Plus :size="17" />
            手动添加
          </button>
        </div>
      </header>

      <section v-if="games.length === 0" class="empty-state" aria-label="空游戏库">
        <div class="empty-copy">
          <p class="eyebrow">
            <Sparkles :size="15" />
            尚未导入游戏
          </p>
          <h2>从扫描本地目录开始建立你的游戏库</h2>
          <p>扫描结果会先进入候选列表，确认后才会写入本地数据库。</p>
        </div>
        <div class="empty-actions">
          <button class="primary-action" type="button">
            <FolderSearch :size="17" />
            扫描目录
          </button>
          <button class="secondary-action" type="button">
            <Plus :size="17" />
            手动添加
          </button>
        </div>
      </section>

      <section v-else class="game-area" :class="viewMode">
        <article v-for="game in visibleGames" :key="game.id" class="game-item">
          <div class="game-icon">{{ game.name.slice(0, 1).toUpperCase() }}</div>
          <div class="game-meta">
            <h2>{{ game.name }}</h2>
            <p>{{ game.exePath }}</p>
          </div>
          <button class="primary-action compact" type="button">启动</button>
        </article>
      </section>
    </section>
  </main>
</template>
