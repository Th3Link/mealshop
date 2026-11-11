<script setup lang="ts">
import { useId } from 'vue'
import { t } from './i18n'
const model = defineModel<number | null>({ required: true })
defineProps<{ label: string }>()
const id = useId()
</script>
<template>
  <label class="field" :for="id"
    >{{ label
    }}<select :id="id" v-model="model">
      <option :value="null">{{ t('historyNoRating') }}</option>
      <option v-for="rating in 5" :key="rating" :value="rating">{{ rating }} / 5</option>
    </select></label
  >
</template>
