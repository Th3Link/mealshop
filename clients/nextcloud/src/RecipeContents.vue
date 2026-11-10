<script setup lang="ts">
import { computed } from 'vue'
import type { z } from 'zod'
import { recipeRecordSchema, displayLinesSchema } from './contracts'
import { t, locale, compactExact, exact } from './i18n'
const props = defineProps<{
  record: z.infer<typeof recipeRecordSchema>
  lines: z.infer<typeof displayLinesSchema>
}>()
const parts = computed(() => [
  { id: null, name: t('mainRecipe'), uses_parts: [] },
  ...props.record.recipe.parts,
])
function ingredient(index: number) {
  const line = props.record.recipe.lines[index]
  const item = props.record.nutrition.ingredients.find(
    (i) => i.ingredient_id === line?.ingredient_id,
  )
  return item?.translations[locale.value]?.name ?? item?.name ?? line?.ingredient_id ?? ''
}
const partName = (id: string) => props.record.recipe.parts.find((p) => p.id === id)?.name ?? id
function lineName(id: string) {
  return ingredient(props.record.recipe.lines.findIndex((l) => l.id === id))
}
</script>
<template>
  <section
    v-for="part in parts.filter(
      (p) =>
        record.recipe.lines.some((l) => (l.part_id ?? null) === p.id) ||
        record.recipe.steps.some((s) => s.part_id === p.id),
    )"
    :key="part.id ?? 'main'"
    class="recipe-part-content"
  >
    <h2 v-if="record.recipe.parts.length">{{ part.name }}</h2>
    <p v-if="part.uses_parts.length" class="muted">
      {{ t('usesParts') }}: {{ part.uses_parts.map(partName).join(', ') }}
    </p>
    <component :is="record.recipe.parts.length ? 'h3' : 'h2'">{{ t('lines') }}</component>
    <ul class="plain-list">
      <li
        v-for="entry in lines
          .map((line, index) => ({ line, index }))
          .filter((e) => (record.recipe.lines[e.index]?.part_id ?? null) === part.id)"
        :key="entry.index"
      >
        {{ compactExact(entry.line.amount) }} {{ entry.line.unit }} · {{ ingredient(entry.index)
        }}<span
          v-for="extension in record.recipe.lines[entry.index]?.extensions ?? []"
          :key="extension.type"
        >
          · {{ t('ingredientTemperature') }}: {{ exact(extension.value) }} °C</span
        ><small v-if="record.recipe.lines[entry.index]?.quantity">
          · {{ t('originalInput') }}:
          {{ exact(record.recipe.lines[entry.index]?.quantity?.amount ?? null) }}
          {{ record.recipe.lines[entry.index]?.quantity?.unit }}</small
        >
      </li>
    </ul>
    <component :is="record.recipe.parts.length ? 'h3' : 'h2'">{{ t('steps') }}</component>
    <ol class="plain-list">
      <li
        v-for="(step, index) in record.recipe.steps.filter((s) => s.part_id === part.id)"
        :key="index"
      >
        <p :lang="record.recipe.language">{{ step.instruction }}</p>
        <p v-if="step.ingredient_line_ids.length" class="step-references">
          {{ t('stepIngredients') }}: {{ step.ingredient_line_ids.map(lineName).join(', ') }}
        </p>
        <p v-if="step.prepared_parts.length" class="step-references">
          {{ t('stepParts') }}: {{ step.prepared_parts.map(partName).join(', ') }}
        </p>
        <small
          >{{ t('activeSeconds') }}: {{ step.active_seconds }} · {{ t('passiveSeconds') }}:
          {{ step.passive_seconds }}</small
        >
      </li>
    </ol>
  </section>
</template>
