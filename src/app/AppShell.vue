<script setup lang="ts">
  import { Settings } from '@lucide/vue'
  import brandIconUrl from '../assets/brand-icon.png'
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
      <header class="top-bar">
        <slot name="toolbar" />
      </header>

      <div class="workspace-body">
        <slot />
      </div>
    </section>
  </main>
</template>

<style scoped>
  .app-shell {
    display: grid;
    grid-template-columns: 210px minmax(0, 1fr);
    height: 100vh;
    overflow: hidden;
    background:
      linear-gradient(rgba(255, 255, 255, 0.015) 1px, transparent 1px),
      linear-gradient(90deg, rgba(255, 255, 255, 0.012) 1px, transparent 1px),
      radial-gradient(circle at 0 0, rgba(124, 92, 255, 0.055), transparent 320px);
    background-size:
      56px 56px,
      56px 56px,
      auto,
      auto;
  }

  .sidebar {
    position: sticky;
    top: 0;
    display: flex;
    flex-direction: column;
    height: 100vh;
    border-right: 1px solid rgba(255, 255, 255, 0.08);
    background: var(--sidebar);
    padding: 26px 18px 22px;
  }

  .brand-block {
    display: flex;
    gap: 10px;
    align-items: center;
    min-height: 34px;
    color: var(--text);
    font-size: 13px;
    font-weight: 750;
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
    display: grid;
    grid-template-columns: minmax(260px, 1fr) auto;
    gap: 14px;
    align-items: center;
    padding: 22px clamp(18px, 3vw, 34px) 16px;
    background: linear-gradient(180deg, rgba(17, 16, 21, 0.66) 0%, rgba(17, 16, 21, 0.42) 64%, transparent 100%);
  }

  .workspace-body {
    min-height: 0;
    overflow: auto;
    padding: 0 clamp(18px, 3vw, 34px) 34px;
    scrollbar-width: none;
  }

  .workspace-body::-webkit-scrollbar {
    display: none;
    width: 0;
    height: 0;
  }

  @media (max-width: 960px) {
    .app-shell {
      grid-template-columns: 74px minmax(0, 1fr);
    }

    .sidebar {
      padding: 20px 12px;
    }

    .brand-block span,
    .settings-entry span {
      display: none;
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

    .top-bar {
      grid-template-columns: 1fr;
      padding: 16px 14px;
    }

    .workspace-body {
      overflow: visible;
      padding: 0 14px 26px;
    }
  }
</style>
