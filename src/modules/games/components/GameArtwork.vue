<script setup lang="ts">
  import { computed } from 'vue'
  import type { StyleValue } from 'vue'
  import { convertFileSrc } from '@tauri-apps/api/core'
  import type { Game } from '../types/game'

  type ArtworkGame = Pick<Game, 'name' | 'cover' | 'icon'>

  const props = defineProps<{
    game: ArtworkGame
    variant: 'grid' | 'quick' | 'list' | 'preview'
  }>()

  const initial = computed(() => props.game.name.trim().slice(0, 1).toUpperCase() || 'G')
  const coverSrc = computed(() => toLocalAssetSrc(props.game.cover))
  const iconSrc = computed(() => toLocalAssetSrc(props.game.icon))
  const artworkStyle = computed<StyleValue>(() => {
    const hash = Array.from(props.game.name).reduce(
      (value, character) => (value * 31 + character.charCodeAt(0)) % 100_000,
      0
    )
    return { '--artwork-hue': String(205 + Math.abs(hash % 105)) }
  })

  function toLocalAssetSrc(path?: string | null) {
    return path ? convertFileSrc(path) : null
  }
</script>

<template>
  <div
    class="game-artwork"
    :class="[
      `game-artwork--${props.variant}`,
      {
        'game-artwork--cover': props.variant !== 'list' && coverSrc,
        'game-artwork--icon': !coverSrc && iconSrc,
        'game-artwork--fallback': !coverSrc && !iconSrc
      }
    ]"
    :style="artworkStyle"
    aria-hidden="true"
  >
    <img v-if="props.variant !== 'list' && coverSrc" class="game-artwork__cover" :src="coverSrc" alt="" />
    <template v-else-if="iconSrc">
      <img v-if="props.variant !== 'list'" class="game-artwork__icon-backdrop" :src="iconSrc" alt="" />
      <span v-if="props.variant !== 'list'" class="game-artwork__logo-frame">
        <img class="game-artwork__logo" :src="iconSrc" alt="" />
      </span>
      <img v-else class="game-artwork__logo" :src="iconSrc" alt="" />
    </template>
    <span v-else class="game-artwork__monogram">{{ initial }}</span>
  </div>
</template>

<style scoped>
  .game-artwork {
    --artwork-hue: 258;
    position: relative;
    isolation: isolate;
    display: grid;
    overflow: hidden;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.1);
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
    background: linear-gradient(160deg, hsl(var(--artwork-hue) 45% 24%), rgba(20, 18, 27, 0.96));
    font-size: 38px;
  }

  .game-artwork--quick {
    width: 100%;
    height: 100%;
    background: linear-gradient(145deg, hsl(var(--artwork-hue) 42% 22%), rgba(24, 22, 30, 0.96));
    font-size: 32px;
  }

  .game-artwork--preview {
    width: 100%;
    height: 100%;
    border: 0;
    border-radius: 0;
    background: linear-gradient(160deg, hsl(var(--artwork-hue) 45% 24%), rgba(20, 18, 27, 0.96));
  }

  .game-artwork--list {
    width: 36px;
    height: 36px;
    font-size: 15px;
  }

  .game-artwork__cover {
    width: 100%;
    height: 100%;
    object-fit: cover;
    transition: transform 180ms ease;
  }

  .game-artwork--icon::before,
  .game-artwork--fallback::before {
    position: absolute;
    z-index: -1;
    inset: 0;
    background:
      radial-gradient(circle at 18% 8%, hsl(var(--artwork-hue) 88% 72% / 0.28), transparent 38%),
      radial-gradient(circle at 82% 64%, hsl(calc(var(--artwork-hue) + 38) 76% 62% / 0.2), transparent 46%),
      linear-gradient(160deg, hsl(var(--artwork-hue) 42% 22%), rgba(18, 16, 24, 0.98));
    content: '';
  }

  .game-artwork--icon::after,
  .game-artwork--fallback::after {
    position: absolute;
    z-index: 0;
    inset: 0;
    background-image: linear-gradient(rgba(255, 255, 255, 0.028) 1px, transparent 1px);
    background-size: 100% 7px;
    content: '';
    mask-image: linear-gradient(to bottom, rgba(0, 0, 0, 0.6), transparent 82%);
    pointer-events: none;
  }

  .game-artwork__logo {
    position: relative;
    z-index: 1;
    width: 62%;
    height: 62%;
    object-fit: contain;
  }

  .game-artwork__logo-frame {
    position: relative;
    z-index: 1;
    display: grid;
    width: 84px;
    height: 84px;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.16);
    border-radius: 20px;
    background: rgba(12, 10, 18, 0.42);
    box-shadow:
      0 18px 46px rgba(0, 0, 0, 0.34),
      inset 0 1px 0 rgba(255, 255, 255, 0.08);
    transform: translateY(-36px);
    backdrop-filter: blur(14px);
  }

  .game-artwork__logo-frame .game-artwork__logo {
    width: 58px;
    height: 58px;
  }

  .game-artwork__icon-backdrop {
    position: absolute;
    inset: 0;
    width: 100%;
    height: 100%;
    z-index: -1;
    filter: blur(28px) saturate(1.45);
    opacity: 0.2;
    object-fit: cover;
    transform: scale(1.45);
  }

  .game-artwork__monogram {
    position: relative;
    z-index: 1;
    display: grid;
    width: 92px;
    height: 92px;
    place-items: center;
    border: 1px solid rgba(255, 255, 255, 0.14);
    border-radius: 26px;
    background: rgba(12, 10, 18, 0.32);
    color: rgba(255, 255, 255, 0.9);
    font-size: 42px;
    font-weight: 750;
    box-shadow:
      0 18px 48px rgba(0, 0, 0, 0.28),
      inset 0 1px 0 rgba(255, 255, 255, 0.08);
    text-shadow: 0 6px 24px hsl(var(--artwork-hue) 90% 65% / 0.34);
    transform: translateY(-36px);
    backdrop-filter: blur(14px);
  }

  .game-artwork--quick .game-artwork__logo-frame,
  .game-artwork--quick .game-artwork__monogram {
    width: 58px;
    height: 58px;
    border-radius: 15px;
    font-size: 27px;
    transform: none;
  }

  .game-artwork--quick .game-artwork__logo-frame .game-artwork__logo {
    width: 42px;
    height: 42px;
  }

  .game-artwork--preview .game-artwork__logo-frame,
  .game-artwork--preview .game-artwork__monogram {
    width: 72px;
    height: 72px;
    border-radius: 18px;
    font-size: 32px;
    transform: none;
  }

  .game-artwork--preview .game-artwork__logo-frame .game-artwork__logo {
    width: 50px;
    height: 50px;
  }

  .game-artwork--quick .game-artwork__cover {
    object-fit: cover;
  }

  .game-artwork--list .game-artwork__logo {
    width: 28px;
    height: 28px;
  }

  .game-artwork--list .game-artwork__monogram {
    width: auto;
    height: auto;
    border: 0;
    border-radius: 0;
    background: transparent;
    box-shadow: none;
    font-size: 15px;
    text-shadow: none;
    transform: none;
    backdrop-filter: none;
  }
</style>
