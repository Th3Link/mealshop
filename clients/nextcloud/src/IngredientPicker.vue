<script setup lang="ts">
import { computed } from 'vue'
import { z } from 'zod'
import NcSelect from '@nextcloud/vue/components/NcSelect'
import type { Ingredient } from './contracts'
import { locale } from './i18n'
const model = defineModel<string[]>({ required: true })
const props = defineProps<{ ingredients: Ingredient[]; label: string; disabled?: boolean }>()
const options = computed(() =>
  props.ingredients.map((i) => ({
    id: i.id,
    label: i.translations[locale.value]?.name ?? i.name,
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
  return search
    .toLocaleLowerCase()
    .trim()
    .split(/\s+/)
    .every((term) => option.search.includes(term))
}
</script>
<template>
  <NcSelect
    :model-value="selected"
    :options="options"
    :input-label="label"
    :disabled="disabled"
    :multiple="true"
    :close-on-select="false"
    :filter-by="filter"
    label="label"
    @update:model-value="update"
  />
</template>
