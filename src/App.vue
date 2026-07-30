<script setup lang="ts">
  import AppUpdateDialog from './modules/updates/components/AppUpdateDialog.vue'
  import ToastContainer from './shared/components/ToastContainer.vue'
</script>

<template>
  <RouterView />
  <AppUpdateDialog />
  <ToastContainer />
</template>
