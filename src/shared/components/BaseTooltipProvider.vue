<script setup lang="ts">
  import { TooltipProvider } from 'reka-ui'
</script>

<template>
  <TooltipProvider :delay-duration="150" :skip-delay-duration="80" :disable-hoverable-content="true">
    <slot />
  </TooltipProvider>
</template>
