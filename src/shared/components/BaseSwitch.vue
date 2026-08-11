<script setup lang="ts">
  import { SwitchRoot, SwitchThumb } from 'reka-ui'

  const props = withDefaults(
    defineProps<{
      modelValue: boolean
      accessibleLabel: string
      disabled?: boolean
    }>(),
    {
      disabled: false
    }
  )

  const emit = defineEmits<{
    'update:modelValue': [value: boolean]
  }>()
</script>

<template>
  <SwitchRoot
    class="base-switch"
    :model-value="props.modelValue"
    :aria-label="props.accessibleLabel"
    :disabled="props.disabled"
    @update:model-value="emit('update:modelValue', $event)"
  >
    <span class="base-switch__track" aria-hidden="true">
      <SwitchThumb class="base-switch__thumb" />
    </span>
    <span class="base-switch__label"><slot /></span>
  </SwitchRoot>
</template>

<style scoped>
  .base-switch {
    display: inline-flex;
    gap: 8px;
    align-items: center;
    border: 0;
    background: transparent;
    color: var(--text-muted);
    padding: 4px;
    font-size: var(--font-size-sm);
  }

  .base-switch__track {
    display: flex;
    width: 34px;
    height: 19px;
    align-items: center;
    border: 1px solid var(--border-strong);
    border-radius: 999px;
    background: var(--surface);
    padding: 2px;
    transition: background 160ms ease;
  }

  .base-switch__thumb {
    width: 13px;
    height: 13px;
    border-radius: 999px;
    background: var(--text-muted);
    transition:
      background 160ms ease,
      transform 160ms ease;
  }

  .base-switch[data-state='checked'] {
    color: var(--text);
  }

  .base-switch[data-state='checked'] .base-switch__track {
    border-color: var(--accent-border);
    background: var(--accent);
  }

  .base-switch[data-state='checked'] .base-switch__thumb {
    background: #ffffff;
    transform: translateX(15px);
  }

  .base-switch:focus-visible {
    border-radius: 7px;
    outline: 3px solid var(--focus-ring);
  }

  .base-switch:disabled {
    cursor: not-allowed;
    opacity: 0.5;
  }
</style>
