<script setup lang="ts">
import { computed, ref, watch, watchEffect } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import type { Reading } from './contracts'
import { exact, locale, parseNonnegative, t } from './i18n'
const model = defineModel<Reading | undefined>()
defineProps<{ label: string }>()
const ranged = ref(Boolean(model.value?.range))
const point = ref(exact(model.value?.amount ?? ''))
const lower = ref(exact(model.value?.range?.lower ?? ''))
const upper = ref(exact(model.value?.range?.upper ?? ''))
const pointField = ref<HTMLInputElement>(),
  lowerField = ref<HTMLInputElement>(),
  upperField = ref<HTMLInputElement>()
const canonical = computed(() => ({
  point: parseNonnegative(point.value),
  lower: parseNonnegative(lower.value),
  upper: parseNonnegative(upper.value),
}))
function atoms(value: string) {
  const [whole = '0', fraction = ''] = value.split('.')
  return BigInt(whole) * 1000000n + BigInt(fraction.padEnd(6, '0'))
}
const validRange = computed(
  () =>
    canonical.value.lower !== null &&
    canonical.value.upper !== null &&
    atoms(canonical.value.lower) <= atoms(canonical.value.upper),
)
watchEffect(() => {
  pointField.value?.setCustomValidity(
    point.value && canonical.value.point === null ? t('nutrientQuantityError') : '',
  )
  for (const field of [lowerField.value, upperField.value])
    field?.setCustomValidity(validRange.value ? '' : t('rangeInputError'))
})
function update() {
  const uncertainty =
    model.value?.uncertainty ?? 'Household declared value or range; verify the supplied evidence'
  if (ranged.value) {
    if (validRange.value && canonical.value.lower !== null && canonical.value.upper !== null)
      model.value = {
        amount: null,
        range: { lower: canonical.value.lower, upper: canonical.value.upper },
        uncertainty,
      }
  } else if (!point.value || canonical.value.point !== null)
    model.value = { amount: canonical.value.point, uncertainty }
}
function toggle() {
  ranged.value = !ranged.value
  if (ranged.value) {
    lower.value = point.value
    upper.value = point.value
  } else {
    point.value = ''
    model.value = { amount: null, uncertainty: model.value?.uncertainty ?? 'Household declaration' }
  }
  update()
}
watch(model, (value) => {
  ranged.value = Boolean(value?.range) || (ranged.value && value?.amount === null)
  if (value?.range) {
    if (parseNonnegative(lower.value) !== value.range.lower) lower.value = exact(value.range.lower)
    if (parseNonnegative(upper.value) !== value.range.upper) upper.value = exact(value.range.upper)
  } else if (parseNonnegative(point.value) !== value?.amount)
    point.value = exact(value?.amount ?? '')
})
watch(locale, (_next, previous) => {
  const oldLower = parseNonnegative(lower.value, previous)
  const oldUpper = parseNonnegative(upper.value, previous)
  if (
    ranged.value &&
    oldLower !== null &&
    oldUpper !== null &&
    atoms(oldLower) <= atoms(oldUpper)
  ) {
    lower.value = exact(oldLower)
    upper.value = exact(oldUpper)
  } else if (!ranged.value) {
    const oldPoint = parseNonnegative(point.value, previous)
    if (!point.value || oldPoint !== null) point.value = exact(oldPoint ?? '')
  }
})
</script>
<template>
  <div class="nutrient-input">
    <div class="nutrient-input-heading">
      <span>{{ label }}</span
      ><NcButton
        variant="tertiary"
        :aria-label="`${t(ranged ? 'pointMode' : 'rangeMode')} · ${label}`"
        :aria-pressed="ranged"
        :title="t(ranged ? 'pointMode' : 'rangeMode')"
        @click="toggle"
        ><span aria-hidden="true">{{ ranged ? '=' : '↔' }}</span></NcButton
      >
    </div>
    <div v-if="ranged" class="nutrient-bounds">
      <label class="field"
        ><span class="visually-hidden">{{ label }} · {{ t('lowerValue') }}</span
        ><input
          ref="lowerField"
          v-model="lower"
          :placeholder="t('lowerValue')"
          required
          inputmode="decimal"
          maxlength="32"
          @input="update" /></label
      ><span aria-hidden="true">–</span
      ><label class="field"
        ><span class="visually-hidden">{{ label }} · {{ t('upperValue') }}</span
        ><input
          ref="upperField"
          v-model="upper"
          :placeholder="t('upperValue')"
          required
          inputmode="decimal"
          maxlength="32"
          @input="update"
      /></label>
    </div>
    <label v-else class="field"
      ><span class="visually-hidden">{{ label }}</span
      ><input ref="pointField" v-model="point" inputmode="decimal" maxlength="32" @input="update"
    /></label>
  </div>
</template>
