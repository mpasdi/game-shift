<script setup lang="ts">
  import { Star } from '@lucide/vue'

  const props = defineProps<{
    active: boolean
  }>()

  const emit = defineEmits<{
    toggle: []
  }>()
</script>

<template>
  <button
    class="game-favorite-toggle"
    :class="{ 'game-favorite-toggle--active': props.active }"
    type="button"
    :title="props.active ? '取消收藏' : '收藏游戏'"
    :aria-label="props.active ? '取消收藏' : '收藏游戏'"
    @click="emit('toggle')"
  >
    <Star :size="14" :fill="props.active ? 'currentColor' : 'none'" />
  </button>
</template>

<style scoped>
  .game-favorite-toggle {
    display: grid;
    position: absolute;
    z-index: 2;
    top: 12px;
    right: 12px;
    width: 26px;
    height: 26px;
    border: 1px solid rgba(168, 139, 250, 0.72);
    border-radius: 999px;
    background: rgba(40, 30, 66, 0.86);
    color: #ddd6fe;
    padding: 0;
    place-items: center;
    box-shadow: 0 8px 18px rgba(0, 0, 0, 0.26);
    backdrop-filter: blur(10px);
    transition:
      background 160ms ease,
      border-color 160ms ease;
  }

  .game-favorite-toggle:hover {
    border-color: rgba(221, 214, 254, 0.86);
    background: rgba(109, 67, 214, 0.94);
    color: #fff;
  }

  .game-favorite-toggle--active {
    border-color: rgba(139, 92, 246, 0.34);
    background: linear-gradient(135deg, #8b5cf6, #6d5dfc);
    color: #fff;
  }
</style>
