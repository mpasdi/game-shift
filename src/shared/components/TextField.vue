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
