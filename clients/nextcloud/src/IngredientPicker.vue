<script setup lang="ts">
import { computed, useId } from 'vue'
import { z } from 'zod'
import NcSelect from '@nextcloud/vue/components/NcSelect'
import type { Ingredient } from './contracts'
import { locale, t } from './i18n'
import { matchesSearch } from './ingredient-search'
const id = useId()
const model = defineModel<string[]>({ required: true })
const props = defineProps<{ ingredients: Ingredient[]; label: string; disabled?: boolean }>()
const options = computed(() =>
  props.ingredients.map((i) => ({
    id: i.id,
    label:
      (i.translations[locale.value]?.name ?? i.name) + (i.archived ? ` · ${t('archived')}` : ''),
    search: [
      i.name,
      ...i.aliases,
      ...Object.values(i.translations).flatMap((tr) => [tr.name, ...tr.aliases]),
    ]
      .join(' ')
      .toLocaleLowerCase(),
  })),
)
const selected = computed(() => options.value.filter((i) => model.value.includes(i.id)))
function update(value: unknown) {
  const result = z.array(z.object({ id: z.string() })).safeParse(value)
  if (result.success) model.value = result.data.map((i) => i.id)
}
function filter(option: { search: string }, _label: string, search: string) {
  return matchesSearch(option.search, search)
}
</script>
<template>
  <div class="labeled-field">
    <label :for="id">{{ label }}</label
    ><NcSelect
      :input-id="id"
      label-outside
      :model-value="selected"
      :options="options"
      :input-label="label"
      :disabled="disabled"
      :multiple="true"
      :close-on-select="false"
      :filter-by="filter"
      label="label"
      @update:model-value="update"
      ><template #option="option"
        ><span class="mealshop-selection-name">{{ option.label }}</span></template
      ><template #selected-option="option"
        ><span class="mealshop-selection-name">{{ option.label }}</span></template
      ><template #no-options>{{ t('noSearchResults') }}</template></NcSelect
    >
  </div>
</template>
