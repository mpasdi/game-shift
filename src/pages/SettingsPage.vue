<script setup lang="ts">
  import { Settings } from '@lucide/vue'
  import EmptyState from '../shared/components/EmptyState.vue'
</script>

<template>
  <EmptyState
    label="设置"
    eyebrow="Settings"
    title="设置页面待接入"
    description="后续可以在这里展示扫描目录、数据库位置、导入导出和默认排除规则。"
  >
    <template #icon><Settings :size="15" /></template>
  </EmptyState>
</template>
