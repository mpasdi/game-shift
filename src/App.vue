<script setup lang="ts">
  import AppUpdateDialog from './modules/updates/components/AppUpdateDialog.vue'
  import BaseTooltipProvider from './shared/components/BaseTooltipProvider.vue'
  import ToastContainer from './shared/components/ToastContainer.vue'
</script>

<template>
  <BaseTooltipProvider>
    <RouterView />
    <AppUpdateDialog />
    <ToastContainer />
  </BaseTooltipProvider>
</template>
