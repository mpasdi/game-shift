<script setup lang="ts">
  const props = defineProps<{
    id: string
    modelValue: string
    placeholder?: string
    label?: string
    type?: 'text' | 'search'
    readonly?: boolean
  }>()

  const emit = defineEmits<{
    'update:modelValue': [value: string]
  }>()
</script>

<template>
  <label class="text-field" :class="{ 'text-field--readonly': props.readonly }" :for="props.id">
    <span v-if="$slots.icon" class="text-field__icon">
      <slot name="icon" />
    </span>
    <span v-if="props.label" class="text-field__label">{{ props.label }}</span>
    <input
      :id="props.id"
      class="text-field__input"
      :type="props.type ?? 'text'"
      :value="props.modelValue"
      :placeholder="props.placeholder"
      :readonly="props.readonly"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
  </label>
</template>

<style scoped>
  .text-field {
    display: flex;
    align-items: center;
    width: min(560px, 100%);
    min-height: 38px;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: rgba(255, 255, 255, 0.07);
    color: var(--text);
    padding: 0 12px;
    box-shadow: inset 0 1px 0 rgba(255, 255, 255, 0.045);
    transition:
      border-color 160ms ease,
      background 160ms ease,
      box-shadow 160ms ease;
  }

  .text-field:hover {
    border-color: var(--border-strong);
    background: rgba(255, 255, 255, 0.09);
  }

  .text-field:focus-within {
    border-color: rgba(167, 139, 250, 0.76);
    box-shadow: 0 0 0 3px var(--focus-ring);
  }

  .text-field--readonly {
    border-color: rgba(255, 255, 255, 0.12);
    background: rgba(255, 255, 255, 0.045);
  }

  .text-field--readonly .text-field__label {
    align-self: stretch;
    display: inline-flex;
    align-items: center;
    margin: 0 12px 0 -12px;
    border-right: 1px solid var(--border);
    background: rgba(255, 255, 255, 0.04);
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
    font-size: 13px;
    white-space: nowrap;
  }

  .text-field__input {
    width: 100%;
    min-width: 0;
    border: 0;
    outline: 0;
    background: transparent;
    color: inherit;
    padding: 0 0 0 10px;
  }

  .text-field__input::placeholder {
    color: var(--text-subtle);
  }

  .text-field__input:read-only {
    color: rgba(245, 242, 255, 0.82);
    cursor: text;
  }
</style>
