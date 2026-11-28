<script setup lang="ts">
import StarRating from './StarRating.vue'
const model = defineModel<number | null>({ required: true })
defineProps<{ label: string }>()
</script>
<template>
  <StarRating v-model="model" :label="label" />
</template>
