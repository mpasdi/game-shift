<script setup lang="ts">
  import { TooltipContent, TooltipPortal, TooltipRoot, TooltipTrigger } from 'reka-ui'

  const props = withDefaults(
    defineProps<{
      side?: 'top' | 'right' | 'bottom' | 'left'
      align?: 'start' | 'center' | 'end'
      sideOffset?: number
      disabled?: boolean
    }>(),
    {
      side: 'top',
      align: 'center',
      sideOffset: 8,
      disabled: false
    }
  )
</script>

<template>
  <TooltipRoot :disabled="props.disabled">
    <TooltipTrigger as-child>
      <slot name="trigger" />
    </TooltipTrigger>

    <TooltipPortal>
      <TooltipContent
        class="base-tooltip__content"
        :side="props.side"
        :align="props.align"
        :side-offset="props.sideOffset"
        :collision-padding="12"
      >
        <slot />
      </TooltipContent>
    </TooltipPortal>
  </TooltipRoot>
</template>

<!-- TooltipPortal 挂载到 body，浮层样式需保持非 scoped。 -->
<style>
  .base-tooltip__content {
    z-index: 300;
    width: min(280px, calc(100vw - 24px));
    max-height: calc(100vh - 24px);
    overflow: auto;
    border: 1px solid rgba(139, 92, 246, 0.32);
    border-radius: 9px;
    background: rgba(24, 21, 32, 0.98);
    box-shadow: 0 14px 36px rgba(0, 0, 0, 0.38);
    color: var(--text);
    padding: 10px 12px;
    user-select: none;
  }
</style>
