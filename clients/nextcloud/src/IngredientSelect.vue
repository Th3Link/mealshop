<script setup lang="ts">
import { computed } from 'vue'
import NcSelect from '@nextcloud/vue/components/NcSelect'
import { z } from 'zod'
import type { Ingredient } from './contracts'
import { locale, t } from './i18n'
const model = defineModel<string>({ required: true })
const props = defineProps<{ ingredients: Ingredient[]; label?: string }>()
const emit = defineEmits<{ change: [] }>()
const options = computed(() =>
  props.ingredients
    .filter((item) => !item.archived || item.id === model.value)
    .map((item) => ({
      id: item.id,
      label:
        (item.translations[locale.value]?.name ?? item.name) +
        (item.archived ? ` · ${t('archived')}` : ''),
      search: [
        item.name,
        ...item.aliases,
        ...Object.values(item.translations).flatMap((tr) => [tr.name, ...tr.aliases]),
      ]
        .join(' ')
        .toLocaleLowerCase(),
      favorite: item.favorite,
    }))
    .sort(
      (a, b) =>
        Number(b.favorite) - Number(a.favorite) || a.label.localeCompare(b.label, locale.value),
    ),
)
const selected = computed(() => options.value.find((option) => option.id === model.value) ?? null)
function select(value: unknown) {
  const parsed = z.object({ id: z.string() }).nullable().safeParse(value)
  if (parsed.success) {
    model.value = parsed.data?.id ?? ''
    emit('change')
  }
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
    :input-label="label ?? t('selectIngredient')"
    :placeholder="t('chooseIngredient')"
    :filter-by="filter"
    label="label"
    @update:model-value="select"
  />
</template>
