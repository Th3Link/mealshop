<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import LabeledField from './LabeledField.vue'
import { exact, locale, parseGrams, t } from './i18n'
const model = defineModel<string>({ required: true })
defineProps<{ disabled: boolean }>()
const raw = ref(exact(model.value)),
  touched = ref(false)
function parse(value: string, language = locale.value) {
  const amount = parseGrams(value, language)
  return amount && (amount.split('.')[1]?.length ?? 0) <= 2 ? amount : null
}
const invalid = computed(() => Boolean(raw.value) && !parse(raw.value))
function update(value: string | number) {
  raw.value = String(value)
  model.value = parse(raw.value) ?? ''
}
watch(locale, (_, previous) => {
  const amount = parse(raw.value, previous)
  if (amount) {
    raw.value = exact(amount)
    model.value = amount
  }
})
watch(model, (amount) => {
  if (amount && parse(raw.value) !== amount) raw.value = exact(amount)
})
</script>
<template>
  <LabeledField
    :model-value="raw"
    :label="t('stockPrice')"
    inputmode="decimal"
    required
    maxlength="32"
    :disabled="disabled"
    :aria-invalid="touched && invalid"
    :helper-text="t('stockPriceHelp')"
    @blur="touched = true"
    @update:model-value="update"
  />
</template>
