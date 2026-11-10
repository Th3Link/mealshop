<script setup lang="ts">
import { ref, watch } from 'vue'
import NcTextField from './LabeledField.vue'
import { exact, locale, parseGrams, t } from './i18n'
import type { ShoppingQuantity } from './shopping-api'
const model = defineModel<ShoppingQuantity>({ required: true })
defineProps<{ disabled?: boolean }>()
const raw = ref(exact(model.value.amount)),
  invalid = ref(false)
function input(value: string | number) {
  raw.value = String(value)
  const amount = parseGrams(raw.value)
  invalid.value = Boolean(raw.value) && !amount
  model.value = { ...model.value, amount: amount ?? '' }
}
watch(locale, (_, previous) => {
  const amount = parseGrams(raw.value, previous)
  if (amount) {
    raw.value = exact(amount)
    model.value = { ...model.value, amount }
    invalid.value = false
  } else {
    invalid.value = Boolean(raw.value)
    model.value = { ...model.value, amount: '' }
  }
})
watch(
  () => model.value.amount,
  (amount) => {
    if (amount && parseGrams(raw.value) !== amount) {
      raw.value = exact(amount)
      invalid.value = false
    }
  },
)
</script>
<template>
  <div class="form-row">
    <NcTextField
      :model-value="raw"
      :label="t('shoppingQuantity')"
      inputmode="decimal"
      required
      :disabled="disabled"
      :aria-invalid="invalid"
      :helper-text="invalid ? t('quantityError') : undefined"
      @update:model-value="input"
    />
    <label class="field"
      >{{ t('unit')
      }}<select :aria-label="t('unit')" v-model="model.unit" :disabled="disabled">
        <option v-for="unit in ['g', 'kg', 'ml', 'l', 'piece']" :key="unit" :value="unit">
          {{ unit === 'piece' ? t('shoppingPieces') : unit }}
        </option>
      </select></label
    >
  </div>
</template>
