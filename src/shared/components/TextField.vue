<script setup lang="ts">
  defineProps<{
    id: string
    modelValue: string
    placeholder?: string
    label?: string
    type?: 'text' | 'search'
  }>()

  const emit = defineEmits<{
    'update:modelValue': [value: string]
  }>()
</script>

<template>
  <label class="text-field" :for="id">
    <span v-if="$slots.icon" class="text-field__icon">
      <slot name="icon" />
    </span>
    <span v-if="label" class="text-field__label">{{ label }}</span>
    <input
      :id="id"
      class="text-field__input"
      :type="type ?? 'text'"
      :value="modelValue"
      :placeholder="placeholder"
      @input="emit('update:modelValue', ($event.target as HTMLInputElement).value)"
    />
  </label>
</template>
