<script setup lang="ts">
  import { computed } from 'vue'
  import { convertFileSrc } from '@tauri-apps/api/core'
  import type { Game } from '../types/game'

  const props = defineProps<{
    game: Game
    variant: 'grid' | 'quick' | 'list'
  }>()

  const initial = computed(() => props.game.name.trim().slice(0, 1).toUpperCase() || 'G')
  const coverSrc = computed(() => toLocalAssetSrc(props.game.cover))
  const iconSrc = computed(() => toLocalAssetSrc(props.game.icon))

  function toLocalAssetSrc(path?: string | null) {
    return path ? convertFileSrc(path) : null
  }
</script>

<template>
  <div class="game-artwork" :class="`game-artwork--${props.variant}`" aria-hidden="true">
    <img v-if="props.variant !== 'list' && coverSrc" class="game-artwork__cover" :src="coverSrc" alt="" />
    <img v-else-if="iconSrc" class="game-artwork__logo" :src="iconSrc" alt="" />
    <span v-else>{{ initial }}</span>
  </div>
</template>

<style scoped>
  .game-artwork {
    display: grid;
    overflow: hidden;
    place-items: center;
    border: 1px solid var(--accent-border);
    border-radius: 8px;
    background: linear-gradient(135deg, rgba(124, 92, 255, 0.22), rgba(255, 255, 255, 0.06));
    color: #f5f3ff;
    font-size: 20px;
    font-weight: 800;
  }

  .game-artwork--grid {
    width: 100%;
    height: 100%;
    border: 0;
    border-radius: 0;
    background:
      radial-gradient(circle at 50% 18%, rgba(255, 255, 255, 0.13), transparent 28%),
      linear-gradient(160deg, rgba(124, 92, 255, 0.34), rgba(20, 18, 27, 0.94));
    font-size: 38px;
  }

  .game-artwork--quick {
    width: 100%;
    height: 104px;
    background:
      linear-gradient(145deg, rgba(124, 92, 255, 0.28), rgba(46, 73, 86, 0.16)),
      linear-gradient(180deg, rgba(255, 255, 255, 0.1), rgba(255, 255, 255, 0.03));
    font-size: 32px;
  }

  .game-artwork--list {
    width: 30px;
    height: 30px;
    font-size: 13px;
  }

  .game-artwork__cover {
    width: 100%;
    height: 100%;
    object-fit: cover;
  }

  .game-artwork__logo {
    width: 62%;
    height: 62%;
    object-fit: contain;
  }

  .game-artwork--grid .game-artwork__logo {
    width: 60px;
    height: 60px;
  }

  .game-artwork--quick .game-artwork__logo {
    width: 48px;
    height: 48px;
  }

  .game-artwork--list .game-artwork__logo {
    width: 22px;
    height: 22px;
  }
</style>
