<script setup lang="ts">
  const props = withDefaults(
    defineProps<{
      id: string
      modelValue: string
      placeholder?: string
      label?: string
      type?: 'text' | 'search' | 'password'
      size?: 'sm' | 'md'
      readonly?: boolean
      disabled?: boolean
      invalid?: boolean
      describedBy?: string
      autocomplete?: string
      spellcheck?: boolean
      name?: string
    }>(),
    {
      type: 'text',
      size: 'md',
      readonly: false,
      disabled: false,
      invalid: false,
      placeholder: undefined,
      label: undefined,
      describedBy: undefined,
      autocomplete: undefined,
      spellcheck: undefined,
      name: undefined
    }
  )

  const emit = defineEmits<{
    'update:modelValue': [value: string]
  }>()
</script>

<template>
  <label
    class="text-field"
    :class="[
      `text-field--${props.size}`,
      {
        'text-field--readonly': props.readonly,
        'text-field--disabled': props.disabled,
        'text-field--invalid': props.invalid
      }
    ]"
    :for="props.id"
  >
    <span v-if="$slots.icon" class="text-field__icon">
      <slot name="icon" />
    </span>
    <span v-if="props.label" class="text-field__label">{{ props.label }}</span>
    <input
      :id="props.id"
      class="text-field__input"
      :type="props.type"
      :value="props.modelValue"
      :placeholder="props.placeholder"
      :readonly="props.readonly"
      :disabled="props.disabled"
      :aria-invalid="props.invalid || undefined"
      :aria-describedby="props.describedBy"
      :autocomplete="props.autocomplete"
      :spellcheck="props.spellcheck"
      :name="props.name"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
  </label>
</template>

<style scoped>
  .text-field {
    display: flex;
    align-items: center;
    width: min(560px, 100%);
    border: 1px solid var(--border);
    border-radius: var(--control-radius);
    background: var(--surface);
    color: var(--text);
    padding: 0 12px;
    transition:
      border-color 160ms ease,
      background 160ms ease,
      box-shadow 160ms ease;
  }

  .text-field--md {
    min-height: var(--control-height-md);
  }

  .text-field--sm {
    min-height: var(--control-height-sm);
  }

  .text-field:hover:not(.text-field--disabled) {
    border-color: var(--border-strong);
    background: var(--surface-hover);
  }

  .text-field:has(.text-field__input:focus-visible) {
    border-color: var(--accent-border);
    box-shadow: var(--control-focus-shadow);
  }

  .text-field--invalid {
    border-color: var(--control-error-border);
  }

  .text-field--invalid:has(.text-field__input:focus-visible) {
    border-color: var(--danger);
    box-shadow: 0 0 0 3px var(--control-error-ring);
  }

  .text-field--disabled {
    cursor: not-allowed;
    opacity: var(--control-disabled-opacity);
  }

  .text-field--readonly {
    border-color: rgba(255, 255, 255, 0.09);
    background: rgba(255, 255, 255, 0.026);
    color: var(--text-muted);
  }

  .text-field--readonly:hover {
    border-color: rgba(255, 255, 255, 0.1);
    background: rgba(255, 255, 255, 0.026);
  }

  .text-field--readonly:has(.text-field__input:focus-visible) {
    border-color: rgba(255, 255, 255, 0.14);
    box-shadow: none;
  }

  .text-field--readonly .text-field__label {
    align-self: stretch;
    display: inline-flex;
    align-items: center;
    margin: 0 12px 0 -12px;
    border-right: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.035);
    color: var(--text-muted);
    padding: 0 12px;
  }

  .text-field__icon {
    display: inline-flex;
    color: var(--text-subtle);
  }

  .text-field__label {
    margin-right: 8px;
    color: var(--text-muted);
    font-size: var(--font-size-md);
    white-space: nowrap;
  }

  .text-field__input {
    width: 100%;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: inherit;
    padding: 0;
  }

  .text-field__input:disabled {
    cursor: not-allowed;
  }

  .text-field__icon + .text-field__input,
  .text-field__label + .text-field__input {
    padding-left: 10px;
  }

  .text-field__input::placeholder {
    color: var(--text-subtle);
  }
  .text-field__input[type='search']::-webkit-search-cancel-button {
    width: 16px;
    height: 16px;
    margin-left: 8px;
    background: var(--accent-strong);
    cursor: pointer;
    opacity: 0.9;
    -webkit-appearance: none;
    appearance: none;
    -webkit-mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='black' stroke-width='2.4' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M18 6 6 18'/%3E%3Cpath d='m6 6 12 12'/%3E%3C/svg%3E")
      center / 14px 14px no-repeat;
    mask: url("data:image/svg+xml,%3Csvg xmlns='http://www.w3.org/2000/svg' viewBox='0 0 24 24' fill='none' stroke='black' stroke-width='2.4' stroke-linecap='round' stroke-linejoin='round'%3E%3Cpath d='M18 6 6 18'/%3E%3Cpath d='m6 6 12 12'/%3E%3C/svg%3E")
      center / 14px 14px no-repeat;
  }

  .text-field__input[type='search']::-webkit-search-cancel-button:hover {
    background: var(--text);
    opacity: 1;
  }

  .text-field__input:read-only {
    color: rgba(226, 221, 232, 0.68);
    cursor: default;
  }

  .text-field__input:read-only::placeholder {
    color: rgba(172, 164, 184, 0.58);
  }
</style>
