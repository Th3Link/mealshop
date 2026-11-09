<script setup lang="ts">
import { computed, ref, useId } from 'vue'
import NcSelect from '@nextcloud/vue/components/NcSelect'
import { z } from 'zod'
import type { Ingredient } from './contracts'
import { locale, t } from './i18n'
import { foldSearch as fold } from './ingredient-search'
const model = defineModel<string>({ required: true })
const props = defineProps<{ ingredients: Ingredient[]; label?: string; allowCreate?: boolean }>()
const emit = defineEmits<{ change: []; create: [name: string] }>()
const id = useId(),
  search = ref('')
const options = computed(() => {
  const byId = new Map(props.ingredients.map((i) => [i.id, i]))
  const items = props.ingredients
    .filter((i) => !i.archived || i.id === model.value)
    .map((i) => {
      const parent = byId.get(i.organization.group_id ?? '')
      const name = i.translations[locale.value]?.name ?? i.name
      const context = parent ? (parent.translations[locale.value]?.name ?? parent.name) : ''
      return {
        kind: 'ingredient',
        id: i.id,
        label: name,
        context,
        form: i.translations[locale.value]?.form ?? i.form,
        search: fold(
          [
            name,
            context,
            i.name,
            ...i.aliases,
            ...Object.values(i.translations).flatMap((tr) => [tr.name, ...tr.aliases]),
          ].join(' '),
        ),
        favorite: i.favorite,
      }
    })
    .sort(
      (a, b) =>
        Number(b.favorite) - Number(a.favorite) || a.label.localeCompare(b.label, locale.value),
    )
  return props.allowCreate && search.value.trim()
    ? [
        ...items,
        {
          kind: 'create',
          id: '',
          label: `${t('createNamed')} „${search.value.trim()}“`,
          context: '',
          form: '',
          search: '',
          favorite: false,
        },
      ]
    : items
})
const selected = computed(
  () => options.value.find((o) => o.kind === 'ingredient' && o.id === model.value) ?? null,
)
function select(value: unknown) {
  const result = z.object({ id: z.string(), kind: z.string() }).nullable().safeParse(value)
  if (!result.success) return
  if (result.data?.kind === 'create') {
    emit('create', search.value.trim())
    return
  }
  model.value = result.data?.id ?? ''
  emit('change')
}
function filter(option: { kind: string; search: string }, _label: string, query: string) {
  return (
    option.kind === 'create' ||
    query
      .trim()
      .split(/\s+/)
      .every((term) => option.search.includes(fold(term)))
  )
}
</script>
<template>
  <div class="ingredient-select labeled-field">
    <label :for="id">{{ label ?? t('selectIngredient') }}</label>
    <NcSelect
      :input-id="id"
      :model-value="selected"
      :options="options"
      :input-label="label ?? t('selectIngredient')"
      label-outside
      :placeholder="t('chooseIngredient')"
      :filter-by="filter"
      label="label"
      @search="search = $event"
      @update:model-value="select"
    >
      <template #option="option"
        ><div
          class="mealshop-selection-option"
          :class="{ 'mealshop-create-option': option.kind === 'create' }"
        >
          <strong>{{ option.label }}</strong
          ><small aria-hidden="true" v-if="option.context">{{ option.context }}</small
          ><small aria-hidden="true" v-else-if="option.kind === 'ingredient'">{{
            option.form
          }}</small>
        </div></template
      >
      <template #selected-option="option"
        ><span class="mealshop-selection-name" :title="option.label">{{
          option.label
        }}</span></template
      >
      <template #no-options>{{ t('noSearchResults') }}</template>
    </NcSelect>
  </div>
</template>
