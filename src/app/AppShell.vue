<script setup lang="ts">
  import { Gamepad2, Settings } from '@lucide/vue'
</script>

<template>
  <main class="app-shell">
    <aside class="sidebar" aria-label="主导航">
      <div class="brand-block">
        <div class="brand-mark"><Gamepad2 :size="20" /></div>
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

      <slot />
    </section>
  </main>
</template>

<style scoped>
  .app-shell {
    display: grid;
    grid-template-columns: 210px minmax(0, 1fr);
    min-height: 100vh;
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
    min-width: 0;
    padding: 22px clamp(18px, 3vw, 34px) 34px;
  }

  .top-bar {
    position: sticky;
    top: 0;
    z-index: 20;
    display: grid;
    grid-template-columns: minmax(260px, 1fr) auto;
    gap: 14px;
    align-items: center;
    padding-bottom: 16px;
    background: linear-gradient(180deg, rgba(17, 16, 21, 0.66) 0%, rgba(17, 16, 21, 0.42) 64%, transparent 100%);
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
    }

    .sidebar {
      position: static;
      height: auto;
      border-right: 0;
      border-bottom: 1px solid var(--border);
    }

    .workspace {
      padding: 16px 14px 26px;
    }

    .top-bar {
      grid-template-columns: 1fr;
    }
  }
</style>
