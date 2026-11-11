<script setup lang="ts">
import { t } from './i18n'
defineProps<{ unknown?: boolean }>()
</script>
<template>
  <span class="estimate-badge" :title="t(unknown ? 'nutritionMissingHelp' : 'estimateHelp')"
    ><span aria-hidden="true">{{ unknown ? '?' : '≈' }}</span>
    <span class="estimate-label">{{ t(unknown ? 'nutritionMissing' : 'estimated') }}</span></span
  >
</template>
