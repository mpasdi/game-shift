<script setup lang="ts">
  import { computed } from 'vue'
  import { Check, ChevronDown } from '@lucide/vue'
  import {
    SelectContent,
    SelectIcon,
    SelectItem,
    SelectItemIndicator,
    SelectItemText,
    SelectPortal,
    SelectRoot,
    SelectTrigger,
    SelectValue,
    SelectViewport,
    type AcceptableValue
  } from 'reka-ui'

  interface BaseSelectOption {
    value: string
    label: string
    description?: string
  }

  const props = withDefaults(
    defineProps<{
      modelValue: string
      options: BaseSelectOption[]
      accessibleLabel: string
      placeholder?: string
      size?: 'sm' | 'md'
      disabled?: boolean
      loading?: boolean
      invalid?: boolean
      describedBy?: string
    }>(),
    {
      placeholder: '请选择',
      size: 'md',
      disabled: false,
      loading: false,
      invalid: false,
      describedBy: undefined
    }
  )

  const emit = defineEmits<{
    'update:modelValue': [value: string]
  }>()

  const selectedOption = computed(() => props.options.find((option) => option.value === props.modelValue) ?? null)

  function updateValue(value: AcceptableValue) {
    if (typeof value === 'string') emit('update:modelValue', value)
  }
</script>

<template>
  <SelectRoot
    :model-value="props.modelValue"
    :disabled="props.disabled || props.loading || !props.options.length"
    @update:model-value="updateValue"
  >
    <SelectTrigger
      class="base-select__trigger"
      :class="`base-select__trigger--${props.size}`"
      :aria-label="props.accessibleLabel"
      :aria-busy="props.loading || undefined"
      :aria-invalid="props.invalid || undefined"
      :aria-describedby="props.describedBy"
    >
      <SelectValue class="base-select__value" :placeholder="props.placeholder">
        {{ selectedOption?.label }}
      </SelectValue>
      <span v-if="props.loading" class="base-select__spinner" aria-hidden="true" />
      <SelectIcon v-else as-child>
        <ChevronDown class="base-select__chevron" :size="16" aria-hidden="true" />
      </SelectIcon>
    </SelectTrigger>

    <SelectPortal>
      <SelectContent
        class="base-select__content"
        position="popper"
        align="start"
        :side-offset="6"
        :collision-padding="12"
      >
        <SelectViewport class="base-select__viewport">
          <SelectItem
            v-for="option in props.options"
            :key="option.value"
            class="base-select__option"
            :value="option.value"
            :title="option.label"
          >
            <SelectItemText as-child>
              <span class="base-select__option-copy">
                <span class="base-select__option-label">{{ option.label }}</span>
                <small v-if="option.description">{{ option.description }}</small>
              </span>
            </SelectItemText>
            <SelectItemIndicator class="base-select__indicator">
              <Check :size="16" aria-hidden="true" />
            </SelectItemIndicator>
          </SelectItem>
        </SelectViewport>
      </SelectContent>
    </SelectPortal>
  </SelectRoot>
</template>

<style scoped>
  .base-select__trigger {
    display: grid;
    width: 100%;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
    border: 1px solid var(--border);
    border-radius: var(--control-radius);
    padding: 0 12px;
    background: var(--surface);
    color: var(--text);
    text-align: left;
    transition:
      border-color 160ms ease,
      background 160ms ease,
      box-shadow 160ms ease;
  }

  .base-select__trigger--md {
    min-height: var(--control-height-md);
  }

  .base-select__trigger--sm {
    min-height: var(--control-height-sm);
    padding-right: 10px;
    padding-left: 10px;
    font-size: var(--font-size-sm);
  }

  .base-select__trigger:hover:not(:disabled) {
    border-color: var(--border-strong);
    background: var(--surface-hover);
  }

  .base-select__trigger:focus-visible {
    border-color: var(--accent-border);
    outline: 0;
    box-shadow: var(--control-focus-shadow);
  }

  .base-select__trigger[aria-invalid='true'] {
    border-color: var(--control-error-border);
  }

  .base-select__trigger[aria-invalid='true']:focus-visible {
    border-color: var(--danger);
    box-shadow: 0 0 0 3px var(--control-error-ring);
  }

  .base-select__trigger:disabled {
    cursor: not-allowed;
    opacity: var(--control-disabled-opacity);
  }

  .base-select__value {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .base-select__value[data-placeholder] {
    color: var(--text-subtle);
  }

  .base-select__chevron {
    color: var(--text-subtle);
    transition: transform 160ms ease;
  }

  .base-select__trigger[data-state='open'] .base-select__chevron {
    transform: rotate(180deg);
  }

  .base-select__spinner {
    width: 14px;
    height: 14px;
    border: 2px solid var(--text-muted);
    border-right-color: transparent;
    border-radius: 999px;
    animation: select-spin 680ms linear infinite;
  }

  @keyframes select-spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>

<!-- SelectPortal 挂载到 body，以下样式需保持非 scoped。 -->
<style>
  .base-select__content {
    z-index: 200;
    width: var(--reka-select-trigger-width);
    max-height: min(230px, var(--reka-select-content-available-height));
    overflow: hidden;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    background: #2a2631;
    box-shadow: var(--shadow);
  }

  .base-select__viewport {
    padding: 6px;
  }

  .base-select__option {
    display: grid;
    min-height: 38px;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 12px;
    align-items: center;
    border: 0;
    border-radius: 6px;
    padding: 7px 9px;
    background: transparent;
    color: var(--text-muted);
    text-align: left;
  }

  .base-select__option[data-highlighted] {
    outline: 0;
    background: var(--surface-hover);
    color: var(--text);
  }

  .base-select__option[data-state='checked'] {
    color: var(--text);
  }

  .base-select__option-copy {
    display: grid;
    min-width: 0;
    gap: 2px;
  }

  .base-select__option-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .base-select__option-copy small {
    color: var(--text-subtle);
    font-size: var(--font-size-xs);
  }

  .base-select__indicator {
    display: grid;
    color: var(--accent-strong);
    place-items: center;
  }
</style>
