<script setup lang="ts">
import { computed } from 'vue'
import { mdiStar, mdiStarHalfFull, mdiStarOutline } from '@mdi/js'
import Icon from './Icon.vue'
import { locale, t } from './i18n'
const props = defineProps<{ sum: number; count: number }>()
const average = computed(() => (props.count ? props.sum / props.count : 0))
const text = computed(() =>
  new Intl.NumberFormat(locale.value, { maximumFractionDigits: 1 }).format(average.value),
)
</script>
<template>
  <span v-if="count" class="star-average" role="img" :aria-label="`${text} ${t('ratingStars')}`"
    ><Icon
      v-for="star in 5"
      :key="star"
      :path="
        average >= star - 0.25 ? mdiStar : average >= star - 0.75 ? mdiStarHalfFull : mdiStarOutline
      "
    /><span aria-hidden="true">{{ text }}</span></span
  >
</template>
