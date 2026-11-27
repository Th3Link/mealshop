<script setup lang="ts">
import { useId } from 'vue'
import { mdiStar, mdiStarOutline, mdiClose } from '@mdi/js'
import Icon from './Icon.vue'
import { t } from './i18n'
const model = defineModel<number | null>({ required: true })
defineProps<{ label: string; disabled?: boolean }>()
const name = useId()
</script>
<template>
  <fieldset class="star-rating" :disabled="disabled">
    <legend>{{ label }}</legend>
    <div class="stars">
      <label v-for="rating in 5" :key="rating" :class="{ filled: (model ?? 0) >= rating }"
        ><input
          type="radio"
          class="visually-hidden"
          :name="name"
          :value="rating"
          :checked="model === rating"
          :aria-label="`${rating} / 5`"
          @change="model = rating" /><Icon
          :path="(model ?? 0) >= rating ? mdiStar : mdiStarOutline"
      /></label>
      <button
        v-if="model !== null"
        type="button"
        class="star-clear"
        :aria-label="`${t('ratingClear')} · ${label}`"
        :title="t('ratingClear')"
        @click="model = null"
      >
        <Icon :path="mdiClose" />
      </button>
      <span v-else class="muted">{{ t('historyNoRating') }}</span>
    </div>
  </fieldset>
</template>
