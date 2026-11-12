<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import LabeledField from './LabeledField.vue'
import { foldSearch } from './ingredient-search'
import { t } from './i18n'
const model = defineModel<string[]>({ required: true })
const props = defineProps<{
  label: string
  options: ReadonlyArray<{ id: string; label: string }>
  disabled: boolean
}>()
const query = ref(''),
  page = ref(0)
const options = computed(() => [
  ...props.options,
  ...model.value
    .filter((id) => !props.options.some((o) => o.id === id))
    .map((id, index) => ({ id, label: `${t('stockUnavailableEntry')} ${index + 1}` })),
])
const matching = computed(() =>
  options.value.filter((o) => foldSearch(o.label).includes(foldSearch(query.value))),
)
const visible = computed(() => matching.value.slice(page.value * 25, (page.value + 1) * 25))
watch(query, () => {
  page.value = 0
})
const all = computed(() => [...new Set([...model.value, ...matching.value.map((o) => o.id)])])
</script>
<template>
  <fieldset class="editor-fields stock-selection" :disabled="disabled">
    <legend>{{ label }}</legend>
    <LabeledField v-model="query" :label="t('stockSelectSearch')" type="search" />
    <div class="actions">
      <NcButton :disabled="disabled || all.length > 100 || !matching.length" @click="model = all">{{
        t('stockSelectAll')
      }}</NcButton
      ><NcButton v-if="model.length" :disabled="disabled" @click="model = []">{{
        t('stockClearSelection')
      }}</NcButton
      ><span role="status">{{ model.length }} / 100</span>
    </div>
    <p v-if="!matching.length" class="muted">{{ t('stockNoMatches') }}</p>
    <NcCheckboxRadioSwitch
      v-for="option in visible"
      :key="option.id"
      v-model="model"
      :value="option.id"
      :disabled="disabled || (model.length >= 100 && !model.includes(option.id))"
      >{{ option.label }}</NcCheckboxRadioSwitch
    >
    <div v-if="matching.length > 25" class="actions">
      <NcButton :disabled="page === 0" @click="page--">{{ t('stockPagesPrevious') }}</NcButton
      ><NcButton :disabled="(page + 1) * 25 >= matching.length" @click="page++">{{
        t('stockPagesNext')
      }}</NcButton>
    </div>
  </fieldset>
</template>
