<script setup lang="ts">
  import { computed, nextTick, ref, useId, watch } from 'vue'
  import { onClickOutside } from '@vueuse/core'
  import { Check, ChevronDown } from '@lucide/vue'

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
      disabled?: boolean
      loading?: boolean
    }>(),
    {
      placeholder: '请选择',
      disabled: false,
      loading: false
    }
  )

  const emit = defineEmits<{
    'update:modelValue': [value: string]
  }>()

  const root = ref<HTMLElement | null>(null)
  const trigger = ref<HTMLButtonElement | null>(null)
  const isOpen = ref(false)
  const activeIndex = ref(-1)
  const listboxId = `base-select-${useId()}`

  const selectedOption = computed(() => props.options.find((option) => option.value === props.modelValue) ?? null)
  const activeOptionId = computed(() =>
    isOpen.value && activeIndex.value >= 0 ? `${listboxId}-option-${activeIndex.value}` : undefined
  )

  watch(
    () => [props.modelValue, props.options] as const,
    () => {
      activeIndex.value = selectedIndex()
      if (!props.options.length) isOpen.value = false
    },
    { deep: true }
  )

  onClickOutside(root, close)

  function toggle() {
    if (props.disabled || props.loading || !props.options.length) return
    if (isOpen.value) {
      close()
      return
    }
    open()
  }

  function open() {
    isOpen.value = true
    activeIndex.value = Math.max(selectedIndex(), 0)
  }

  function close() {
    isOpen.value = false
  }

  function choose(option: BaseSelectOption) {
    emit('update:modelValue', option.value)
    close()
    void nextTick(() => trigger.value?.focus())
  }

  function handleKeydown(event: KeyboardEvent) {
    if (props.disabled || props.loading || !props.options.length) return

    if (event.key === 'Escape') {
      if (!isOpen.value) return
      event.preventDefault()
      close()
      return
    }

    if (event.key === 'ArrowDown' || event.key === 'ArrowUp') {
      event.preventDefault()
      if (!isOpen.value) open()
      const direction = event.key === 'ArrowDown' ? 1 : -1
      activeIndex.value = wrapIndex(activeIndex.value + direction)
      return
    }

    if (event.key === 'Enter' || event.key === ' ') {
      event.preventDefault()
      if (!isOpen.value) {
        open()
        return
      }
      const option = props.options[activeIndex.value]
      if (option) choose(option)
    }
  }

  function handleFocusOut() {
    requestAnimationFrame(() => {
      if (root.value?.contains(document.activeElement)) return
      close()
    })
  }

  function selectedIndex() {
    const index = props.options.findIndex((option) => option.value === props.modelValue)
    return index >= 0 ? index : 0
  }

  function wrapIndex(index: number) {
    const length = props.options.length
    return ((index % length) + length) % length
  }
</script>

<template>
  <div ref="root" class="base-select" @focusout="handleFocusOut">
    <button
      ref="trigger"
      class="base-select__trigger"
      type="button"
      role="combobox"
      aria-haspopup="listbox"
      :aria-label="props.accessibleLabel"
      :aria-expanded="isOpen"
      :aria-controls="listboxId"
      :aria-activedescendant="activeOptionId"
      :disabled="props.disabled || props.loading || !props.options.length"
      @click="toggle"
      @keydown="handleKeydown"
    >
      <span class="base-select__value" :class="{ 'base-select__value--placeholder': !selectedOption }">
        {{ selectedOption?.label || props.placeholder }}
      </span>
      <span v-if="props.loading" class="base-select__spinner" aria-hidden="true" />
      <ChevronDown
        v-else
        class="base-select__chevron"
        :class="{ 'base-select__chevron--open': isOpen }"
        :size="16"
        aria-hidden="true"
      />
    </button>

    <div v-if="isOpen" :id="listboxId" class="base-select__content" role="listbox" :aria-label="props.accessibleLabel">
      <button
        v-for="(option, index) in props.options"
        :id="`${listboxId}-option-${index}`"
        :key="option.value"
        class="base-select__option"
        :class="{
          'base-select__option--active': index === activeIndex,
          'base-select__option--selected': option.value === props.modelValue
        }"
        type="button"
        role="option"
        :aria-selected="option.value === props.modelValue"
        :title="option.label"
        @mouseenter="activeIndex = index"
        @click="choose(option)"
      >
        <span class="base-select__option-copy">
          <span class="base-select__option-label">{{ option.label }}</span>
          <small v-if="option.description">{{ option.description }}</small>
        </span>
        <Check v-if="option.value === props.modelValue" :size="16" aria-hidden="true" />
      </button>
    </div>
  </div>
</template>

<style scoped>
  .base-select {
    position: relative;
    min-width: 0;
  }

  .base-select__trigger {
    display: grid;
    width: 100%;
    min-height: 38px;
    grid-template-columns: minmax(0, 1fr) auto;
    gap: 10px;
    align-items: center;
    border: 1px solid var(--border);
    border-radius: 8px;
    padding: 0 12px;
    background: var(--surface);
    color: var(--text);
    text-align: left;
    transition:
      border-color 160ms ease,
      background 160ms ease,
      box-shadow 160ms ease;
  }

  .base-select__trigger:hover:not(:disabled) {
    border-color: var(--border-strong);
    background: var(--surface-hover);
  }

  .base-select__trigger:focus-visible {
    border-color: var(--accent-border);
    outline: 0;
    box-shadow: 0 0 0 3px var(--focus-ring);
  }

  .base-select__trigger:disabled {
    cursor: not-allowed;
    opacity: 0.58;
  }

  .base-select__value,
  .base-select__option-label {
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
  }

  .base-select__value--placeholder {
    color: var(--text-subtle);
  }

  .base-select__chevron {
    color: var(--text-subtle);
    transition: transform 160ms ease;
  }

  .base-select__chevron--open {
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

  .base-select__content {
    position: absolute;
    z-index: 20;
    top: calc(100% + 6px);
    right: 0;
    left: 0;
    display: grid;
    max-height: 230px;
    overflow-y: auto;
    border: 1px solid var(--border-strong);
    border-radius: 8px;
    padding: 6px;
    background: #2a2631;
    box-shadow: var(--shadow);
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

  .base-select__option--active {
    background: var(--surface-hover);
    color: var(--text);
  }

  .base-select__option--selected {
    color: var(--text);
  }

  .base-select__option-copy {
    display: grid;
    min-width: 0;
    gap: 2px;
  }

  .base-select__option-copy small {
    color: var(--text-subtle);
    font-size: var(--font-size-xs);
  }

  .base-select__option > svg {
    color: var(--accent-strong);
  }

  @keyframes select-spin {
    to {
      transform: rotate(360deg);
    }
  }
</style>
