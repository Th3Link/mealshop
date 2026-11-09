<script setup lang="ts">
defineProps<{ path: string }>()
</script>
<template>
  <svg width="20" height="20" viewBox="0 0 24 24" aria-hidden="true" focusable="false">
    <path :d="path" fill="currentColor" />
  </svg>
</template>
