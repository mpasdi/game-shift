<script setup lang="ts">
  import { Settings } from '@lucide/vue'
  import brandIconUrl from '../assets/brand-icon.png'

  withDefaults(
    defineProps<{
      showToolbar?: boolean
    }>(),
    {
      showToolbar: true
    }
  )
</script>

<template>
  <main class="app-shell">
    <aside class="sidebar" aria-label="主导航">
      <div class="brand-block">
        <img class="brand-mark" :src="brandIconUrl" alt="" aria-hidden="true" />
        <span>Game Shift</span>
      </div>

      <slot name="nav" />

      <div class="sidebar-spacer" />

      <slot name="summary" />

      <slot name="settings">
        <button class="settings-entry" type="button">
          <Settings :size="16" />
          <span>设置</span>
        </button>
      </slot>
    </aside>

    <section class="workspace">
      <header v-if="showToolbar" class="top-bar">
        <div class="top-bar__inner">
          <slot name="toolbar" />
        </div>
      </header>

      <div class="workspace-body">
        <slot />
      </div>
    </section>
  </main>
</template>

<style scoped>
  .app-shell {
    --sidebar-expanded-width: 210px;
    --sidebar-collapsed-width: 74px;
    --workspace-max-width: 1760px;
    --workspace-padding-x: clamp(18px, 2.4vw, 42px);
    display: grid;
    grid-template-columns: var(--sidebar-expanded-width) minmax(0, 1fr);
    height: 100vh;
    overflow: hidden;
    background:
      linear-gradient(rgba(255, 255, 255, 0.012) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255, 255, 255, 0.01) 1px, transparent 1px),
      radial-gradient(circle at 8% 8%, rgba(124, 92, 255, 0.07), transparent 360px);
    background-size:
      56px 56px,
      56px 56px,
      auto,
      auto;
    transition: grid-template-columns 220ms ease;
  }

  .sidebar {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    height: 100vh;
    border-right: 1px solid rgba(255, 255, 255, 0.08);
    background: transparent;
    padding: 26px 18px 22px;
    transition: padding 220ms ease;
  }

  .brand-block {
    display: flex;
    gap: 10px;
    align-items: center;
    min-height: 34px;
    color: var(--text);
    font-size: var(--font-size-md);
    font-weight: 700;
    transition: gap 180ms ease;
  }

  .brand-block span,
  .settings-entry span {
    overflow: hidden;
    max-width: 120px;
    white-space: nowrap;
    transition:
      max-width 180ms ease,
      opacity 140ms ease,
      transform 140ms ease;
  }

  .brand-mark {
    display: grid;
    width: 30px;
    height: 30px;
    place-items: center;
    border: 1px solid var(--accent-border);
    border-radius: 8px;
    background: linear-gradient(135deg, var(--accent), #5b44d8);
    color: #ffffff;
    box-shadow: 0 10px 22px rgba(31, 24, 86, 0.34);
  }

  .sidebar-spacer {
    flex: 1;
  }

  .settings-entry {
    display: inline-flex;
    gap: 10px;
    align-items: center;
    min-height: 38px;
    margin-top: 14px;
    border: 0;
    border-radius: 8px;
    background: transparent;
    color: var(--text-muted);
    padding: 0 10px;
  }

  .settings-entry:hover {
    background: var(--surface);
    color: var(--text);
  }

  .workspace {
    display: grid;
    grid-template-rows: auto minmax(0, 1fr);
    min-width: 0;
    min-height: 0;
  }

  .top-bar {
    z-index: 20;
    width: 100%;
  }

  .top-bar__inner {
    display: grid;
    width: 100%;
    grid-template-columns: minmax(260px, 1fr) auto;
    gap: 14px;
    align-items: center;
    padding: 22px var(--workspace-padding-x) 16px;
  }

  .workspace-body {
    width: min(100%, var(--workspace-max-width));
    height: 100%;
    min-height: 0;
    margin: 0 auto;
    overflow: auto;
    padding: 0 var(--workspace-padding-x) 34px;
    scrollbar-width: none;
  }

  .workspace-body::-webkit-scrollbar {
    display: none;
    width: 0;
    height: 0;
  }

  @media (max-width: 960px) {
    .app-shell {
      grid-template-columns: var(--sidebar-collapsed-width) minmax(0, 1fr);
    }

    .sidebar {
      padding: 20px 12px;
    }

    .brand-block,
    .settings-entry {
      justify-content: center;
      padding-right: 0;
      padding-left: 0;
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
      align-self: center;
    }

    .brand-block span,
    .settings-entry span {
      position: absolute;
      max-width: 0;
      opacity: 0;
      transform: translateX(-4px);
      pointer-events: none;
    }
  }

  @media (max-width: 720px) {
    .app-shell {
      grid-template-columns: 1fr;
      height: auto;
      min-height: 100vh;
      overflow: visible;
    }

    .sidebar {
      position: static;
      height: auto;
      border-right: 0;
      border-bottom: 1px solid var(--border);
    }

    .workspace {
      min-height: 0;
    }

    .top-bar__inner {
      grid-template-columns: 1fr;
      padding: 16px 14px;
    }

    .workspace-body {
      overflow: visible;
      padding: 0 14px 26px;
    }
  }
</style>
