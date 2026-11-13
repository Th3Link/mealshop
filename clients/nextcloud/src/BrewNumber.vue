<script setup lang="ts">
import { computed } from 'vue'
import LabeledField from './LabeledField.vue'
import { exact } from './i18n'
const value = defineModel<string>({ required: true })
defineProps<{ label: string; required?: boolean; signed?: boolean }>()
const displayed = computed({
  get: () => exact(value.value),
  set: (v: string | number) => {
    value.value = String(v).replace(',', '.')
  },
})
</script>
<template>
  <LabeledField
    v-model="displayed"
    :label="label"
    :required="required"
    inputmode="decimal"
    maxlength="32"
    :pattern="signed ? '[-]?[0-9]+([.,][0-9]{1,6})?' : '[0-9]+([.,][0-9]{1,6})?'"
  />
</template>
