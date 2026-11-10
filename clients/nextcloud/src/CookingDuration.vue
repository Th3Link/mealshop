<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import NcTextField from './LabeledField.vue'
import { t } from './i18n'
const model = defineModel<number>({ required: true })
const props = withDefaults(defineProps<{ label: string; max?: number }>(), { max: 604800 })
const unit = ref(60),
  raw = ref('')
function refresh() {
  unit.value = model.value % 3600 === 0 && model.value > 0 ? 3600 : model.value % 60 === 0 ? 60 : 1
  raw.value = String(model.value / unit.value)
}
refresh()
watch(model, () => {
  if (
    model.value >= 0 &&
    (!/^\d+$/.test(raw.value) || Number(raw.value) * unit.value !== model.value)
  )
    refresh()
})
const valid = computed(
  () =>
    /^\d+$/.test(raw.value) &&
    Number.isSafeInteger(Number(raw.value)) &&
    Number(raw.value) * unit.value <= props.max,
)
function update(value: string | number) {
  raw.value = String(value)
  model.value = valid.value ? Number(raw.value) * unit.value : -1
}
function change(event: Event) {
  if (event.target instanceof HTMLSelectElement) {
    unit.value = Number(event.target.value)
    update(raw.value)
  }
}
</script>
<template>
  <div class="cooking-duration">
    <NcTextField
      :model-value="raw"
      :label="label"
      inputmode="numeric"
      :aria-invalid="!valid"
      :helper-text="!valid ? t('cookDurationError') : undefined"
      @update:model-value="update"
    /><label class="field"
      ><span class="visually-hidden">{{ label }} · {{ t('unit') }}</span
      ><select :value="unit" :aria-label="`${label} · ${t('unit')}`" @change="change">
        <option :value="1">{{ t('cookSeconds') }}</option>
        <option :value="60">{{ t('cookMinutes') }}</option>
        <option :value="3600">{{ t('cookHours') }}</option>
        <option :value="86400">{{ t('cookDays') }}</option>
      </select></label
    >
  </div>
</template>
