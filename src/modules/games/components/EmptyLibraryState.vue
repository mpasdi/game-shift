<script setup lang="ts">
  import { FileCode2, FolderOpen, Plus } from '@lucide/vue'
  import brandIconUrl from '../../../assets/brand-icon-large.png'
  import BaseButton from '../../../shared/components/BaseButton.vue'

  withDefaults(defineProps<{ scanning?: boolean }>(), { scanning: false })

  defineEmits<{
    add: []
    scan: []
  }>()
</script>

<template>
  <section class="empty-library" aria-labelledby="empty-library-title">
    <div class="empty-library__visual">
      <img class="empty-library__logo" :src="brandIconUrl" alt="" aria-hidden="true" />

      <div class="empty-library__copy">
        <h1 id="empty-library-title">还没有录入任何游戏</h1>
        <p>添加本地启动程序，或扫描游戏文件夹，开始建立你的个人游戏库。</p>
      </div>
    </div>

    <div class="empty-library__actions">
      <BaseButton class="empty-library__primary" variant="primary" @click="$emit('add')">
        <template #icon><Plus :size="20" /></template>
        添加游戏
      </BaseButton>
      <BaseButton variant="secondary" :loading="scanning" @click="$emit('scan')">
        <template #icon><FolderOpen :size="19" /></template>
        扫描文件夹
      </BaseButton>
    </div>

    <div class="empty-library__support" aria-label="支持的游戏导入方式">
      <span class="empty-library__rule" />
      <p>支持两种方式添加游戏</p>
      <div class="empty-library__types">
        <span>
          <FileCode2 :size="17" />
          .exe 可执行文件
        </span>
        <span>
          <FolderOpen :size="17" />
          游戏目录
        </span>
      </div>
    </div>
  </section>
</template>

<style scoped>
  .empty-library {
    position: relative;
    display: flex;
    min-height: 100%;
    align-items: center;
    justify-content: center;
    flex-direction: column;
    overflow: hidden;
    border-radius: 14px;
    padding: clamp(16px, 2.2vh, 28px) 24px;
    text-align: center;
    animation: empty-library-in 320ms ease-out both;
  }

  .empty-library__visual {
    display: grid;
    width: min(620px, 100%);
    gap: clamp(22px, 3vh, 32px);
    justify-items: center;
  }

  .empty-library__logo {
    width: clamp(112px, 12vh, 136px);
    height: clamp(112px, 12vh, 136px);
    object-fit: contain;
    user-select: none;
    -webkit-user-drag: none;
  }

  .empty-library__visual,
  .empty-library__actions,
  .empty-library__support {
    transform: translateY(-3vh);
  }

  .empty-library__copy {
    display: grid;
    width: 100%;
    gap: 10px;
  }

  .empty-library__copy h1,
  .empty-library__copy p,
  .empty-library__support p {
    margin: 0;
  }

  .empty-library__copy h1 {
    color: #ffffff;
    font-size: clamp(24px, 2.2vw, 34px);
    font-weight: 750;
    letter-spacing: 0.01em;
    line-height: 1.2;
  }

  .empty-library__copy p {
    color: rgba(230, 226, 241, 0.65);
    font-size: clamp(13px, 1.1vw, 16px);
    line-height: 1.6;
  }

  .empty-library__actions {
    display: grid;
    width: min(560px, 100%);
    grid-template-columns: repeat(2, minmax(0, 1fr));
    gap: 14px;
    margin-top: clamp(18px, 2.8vh, 30px);
  }

  .empty-library__actions :deep(.base-button) {
    min-height: 44px;
    border-radius: 10px;
    font-size: 14px;
  }

  .empty-library__actions :deep(.base-button--secondary) {
    border-color: rgba(255, 255, 255, 0.11);
    background: rgba(255, 255, 255, 0.045);
  }

  .empty-library__actions :deep(.empty-library__primary) {
    box-shadow: 0 14px 36px rgba(80, 52, 220, 0.3);
  }

  .empty-library__support {
    display: grid;
    width: min(560px, 100%);
    gap: 13px;
    justify-items: center;
    margin-top: clamp(22px, 4vh, 44px);
    color: var(--text-subtle);
    font-size: 12px;
  }

  .empty-library__rule {
    width: 100%;
    height: 1px;
    background: linear-gradient(
      90deg,
      transparent,
      rgba(255, 255, 255, 0.12) 16%,
      rgba(255, 255, 255, 0.12) 84%,
      transparent
    );
  }

  .empty-library__types {
    display: flex;
    flex-wrap: wrap;
    gap: 12px;
    justify-content: center;
  }

  .empty-library__types span {
    display: inline-flex;
    gap: 8px;
    align-items: center;
    min-height: 38px;
    border: 1px solid rgba(255, 255, 255, 0.07);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.028);
    color: rgba(230, 226, 241, 0.52);
    padding: 0 14px;
  }

  .empty-library__types svg {
    color: #8f7cff;
  }

  @keyframes empty-library-in {
    from {
      opacity: 0;
      transform: translateY(8px);
    }

    to {
      opacity: 1;
      transform: translateY(0);
    }
  }

  @media (max-width: 720px) {
    .empty-library {
      min-height: auto;
      padding: 16px 14px 24px;
    }

    .empty-library__visual {
      width: 100%;
      gap: 18px;
    }

    .empty-library__copy {
      gap: 6px;
    }

    .empty-library__actions {
      grid-template-columns: 1fr;
      gap: 10px;
    }

    .empty-library__support {
      margin-top: 28px;
    }
  }

  @media (max-height: 650px) and (min-width: 721px) {
    .empty-library__support {
      margin-top: 18px;
    }

    .empty-library__support p {
      display: none;
    }

    .empty-library__types span {
      min-height: 32px;
    }
  }
</style>
