<script setup lang="ts">
import { computed, ref, nextTick, watch } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextField from './LabeledField.vue'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import Icon from './Icon.vue'
import { mdiPlus, mdiTrashCanOutline } from '@mdi/js'
import RecipeQuantity from './RecipeQuantity.vue'
import RecipeReferences from './RecipeReferences.vue'
import RecipeStepEditor from './RecipeStepEditor.vue'
import type { Recipe, Catalogue } from './contracts'
import type { LineDraft } from './quantity-api'
import type { z } from 'zod'
import { conversionsSchema } from './quantity-api'
import { locale, t } from './i18n'
const recipe = defineModel<Recipe>('recipe', { required: true })
const lines = defineModel<LineDraft[]>('lines', { required: true })
const props = defineProps<{
  catalogue: Catalogue
  conversions: z.infer<typeof conversionsSchema>
  sourceLines?: Record<string, string>
  invalid: boolean
}>()
const emit = defineEmits<{ changed: []; create: [index: number, name: string] }>()
const preparationError = ref('')
watch(
  [recipe, lines],
  () => {
    preparationError.value = ''
  },
  { deep: true },
)
const sections = computed(() => [
  ...(!recipe.value.parts.length ||
  lines.value.some((line) => !line.part_id) ||
  recipe.value.steps.some((step) => !step.part_id)
    ? [{ id: null, name: t('mainRecipe') }]
    : []),
  ...recipe.value.parts,
])
const partOptions = computed(() =>
  recipe.value.parts.map((p) => ({ id: p.id, label: p.name || t('partName') })),
)
const ingredientOptions = computed(() =>
  lines.value.flatMap((line, index) =>
    line.id
      ? [
          {
            id: line.id,
            label: `${index + 1}. ${props.catalogue.ingredients.find((i) => i.id === line.ingredient_id)?.translations[locale.value]?.name ?? props.catalogue.ingredients.find((i) => i.id === line.ingredient_id)?.name ?? t('untitledLine')}`,
          },
        ]
      : [],
  ),
)
function addLine(partId: string | null = null, input = '1') {
  lines.value.push({
    id: `line-${crypto.randomUUID()}`,
    part_id: partId,
    ingredient_id: '',
    input,
    unit: 'g',
    evidence: null,
    mode: 'manual',
    density: '',
    spoon: '',
    invalidFields: [],
  })
  emit('changed')
}
function removeLine(index: number) {
  const line = lines.value[index]
  if (!line || (line.prepared_part && !discardPreparation(index))) return
  const position = lines.value.indexOf(line)
  if (position >= 0) lines.value.splice(position, 1)
  for (const step of recipe.value.steps)
    step.ingredient_line_ids = step.ingredient_line_ids.filter((id) => id !== line.id)
  emit('changed')
}
function addPart() {
  const id = `part-${crypto.randomUUID()}`
  recipe.value.parts.push({ id, name: '', uses_parts: [], starts_after: [] })
  emit('changed')
}
function removePart(id: string) {
  if (lines.value.some((line) => line.prepared_part === id)) {
    preparationError.value = t('preparationInUse')
    return
  }
  if (!window.confirm(t('removePartConfirm'))) return
  recipe.value.parts = recipe.value.parts.filter((p) => p.id !== id)
  for (const p of recipe.value.parts) {
    p.uses_parts = p.uses_parts.filter((ref) => ref !== id)
    p.starts_after = p.starts_after?.filter((ref) => ref !== id)
  }
  for (const line of lines.value) if (line.part_id === id) line.part_id = null
  for (const step of recipe.value.steps) {
    if (step.part_id === id) step.part_id = null
    step.prepared_parts = step.prepared_parts.filter((ref) => ref !== id)
  }
  emit('changed')
}
async function prepare(index: number) {
  const line = lines.value[index]
  if (
    !line?.ingredient_id ||
    line.prepared_part ||
    recipe.value.parts.length >= 30 ||
    lines.value.length >= 100 ||
    recipe.value.steps.length >= 100
  )
    return
  const item = props.catalogue.ingredients.find((item) => item.id === line.ingredient_id)
  const id = `part-${crypto.randomUUID()}`
  recipe.value.parts.push({
    id,
    name: item?.translations[locale.value]?.name ?? item?.name ?? t('homemade'),
    uses_parts: [],
    starts_after: [],
  })
  line.prepared_part = id
  addLine(id, '')
  addStep(id)
  emit('changed')
  await nextTick()
  const section = document.getElementById(`recipe-part-${id}`)
  section?.scrollIntoView({ block: 'start' })
  section?.querySelector<HTMLInputElement>('.ingredient-select input')?.focus()
}
function discardPreparation(index: number) {
  const line = lines.value[index],
    id = line?.prepared_part
  if (!line || !id) return true
  const part = recipe.value.parts.find((part) => part.id === id)
  const shared =
    recipe.value.parts.some(
      (p) => p.id !== id && (p.uses_parts.includes(id) || p.starts_after?.includes(id)),
    ) || recipe.value.steps.some((s) => s.part_id !== id && s.prepared_parts.includes(id))
  const nested =
    recipe.value.steps.some((s) => s.part_id === id && s.prepared_parts.length) ||
    part?.uses_parts.length ||
    lines.value.some((l) => l.part_id === id && l.prepared_part)
  if (shared || nested) {
    preparationError.value = t('preparationInUse')
    return false
  }
  if (!window.confirm(t('buyPreparationConfirm'))) return false
  const removed = new Set(lines.value.filter((l) => l.part_id === id).map((l) => l.id))
  lines.value = lines.value.filter((l) => l.part_id !== id)
  recipe.value.steps = recipe.value.steps.filter((s) => s.part_id !== id)
  for (const step of recipe.value.steps)
    step.ingredient_line_ids = step.ingredient_line_ids.filter((id) => !removed.has(id))
  recipe.value.parts = recipe.value.parts.filter((p) => p.id !== id)
  line.prepared_part = null
  preparationError.value = ''
  emit('changed')
  return true
}
function removeStep(index: number) {
  recipe.value.steps.splice(index, 1)
  emit('changed')
}
function addStep(partId: string | null = null) {
  recipe.value.steps.push({
    instruction: '',
    active_seconds: 0,
    passive_seconds: 0,
    part_id: partId,
    ingredient_line_ids: [],
    prepared_parts: [],
  })
  emit('changed')
}
</script>
<template>
  <NcNoteCard v-if="preparationError" type="error" role="alert">{{ preparationError }}</NcNoteCard>
  <section
    v-for="part in sections"
    :key="part.id ?? 'main'"
    tabindex="-1"
    :id="`recipe-part-${part.id ?? 'main'}`"
    class="editor-section recipe-part-content part-editor"
    :aria-label="part.name || t('partName')"
  >
    <h2 v-if="recipe.parts.length" :class="{ 'visually-hidden': part.id !== null }">
      {{ part.name || t('partName') }}
    </h2>
    <template v-if="part.id !== null">
      <div class="part-heading">
        <NcTextField
          :model-value="part.name"
          :label="t('partName')"
          required
          maxlength="200"
          @update:model-value="
            recipe.parts.forEach((p) => {
              if (p.id === part.id) p.name = String($event)
            })
          "
        />
        <NcButton variant="tertiary" :aria-label="t('removePart')" @click="removePart(part.id)">
          <template #icon><Icon :path="mdiTrashCanOutline" /></template>
        </NcButton>
      </div>
      <div v-if="recipe.parts.length > 1" class="part-dependencies">
        <RecipeReferences
          :model-value="recipe.parts.find((p) => p.id === part.id)?.uses_parts ?? []"
          :label="t('usesParts')"
          :options="partOptions.filter((p) => p.id !== part.id)"
          @update:model-value="
            recipe.parts.forEach((p) => {
              if (p.id === part.id) p.uses_parts = $event
            })
          "
        />
        <RecipeReferences
          :model-value="recipe.parts.find((p) => p.id === part.id)?.starts_after ?? []"
          :label="t('startsAfterParts')"
          :options="partOptions.filter((p) => p.id !== part.id)"
          @update:model-value="
            recipe.parts.forEach((p) => {
              if (p.id === part.id) p.starts_after = $event
            })
          "
        />
      </div>
    </template>
    <div class="recipe-composition">
      <div class="recipe-ingredients-editor">
        <component :is="recipe.parts.length ? 'h3' : 'h2'">{{ t('lines') }}</component>
        <div class="ingredient-column-head" aria-hidden="true">
          <span>{{ t('ingredientColumn') }}</span
          ><span>{{ t('amount') }}</span
          ><span>{{ t('unit') }}</span
          ><span></span><span></span>
        </div>
        <div
          v-for="entry in lines
            .map((line, index) => ({ line, index }))
            .filter((e) => (e.line.part_id ?? null) === part.id)"
          :key="entry.line.id ?? entry.index"
          class="ingredient-entry"
        >
          <p v-if="entry.line.id && sourceLines?.[entry.line.id]" class="import-source-line">
            <strong>{{ t('importSourceIngredient') }}:</strong> {{ sourceLines[entry.line.id] }}
          </p>
          <div class="recipe-ingredient-row">
            <RecipeQuantity
              :model-value="entry.line"
              :index="entry.index"
              :parts="partOptions"
              :ingredients="catalogue.ingredients"
              :conversions="conversions"
              :invalid="invalid"
              :prepared-name="recipe.parts.find((p) => p.id === entry.line.prepared_part)?.name"
              :can-prepare="
                recipe.parts.length < 30 && lines.length < 100 && recipe.steps.length < 100
              "
              @update:model-value="lines[entry.index] = $event"
              @changed="emit('changed')"
              @create="emit('create', entry.index, $event)"
              @prepare="prepare(entry.index)"
              @buy="discardPreparation(entry.index)"
            />
            <NcButton
              variant="tertiary"
              :aria-label="`${t('remove')} ${entry.index + 1}`"
              @click="removeLine(entry.index)"
            >
              <template #icon><Icon :path="mdiTrashCanOutline" /></template>
            </NcButton>
          </div>
        </div>
        <NcButton variant="tertiary" :disabled="lines.length >= 100" @click="addLine(part.id)"
          ><template #icon><Icon :path="mdiPlus" /></template>{{ t('addLine') }}</NcButton
        >
      </div>
      <div class="recipe-steps-editor">
        <component :is="recipe.parts.length ? 'h3' : 'h2'">{{ t('steps') }}</component>
        <RecipeStepEditor
          v-for="entry in recipe.steps
            .map((step, index) => ({ step, index }))
            .filter((e) => e.step.part_id === part.id)"
          :key="entry.index"
          :model-value="entry.step"
          @update:model-value="recipe.steps[entry.index] = $event"
          :index="entry.index"
          :parts="partOptions"
          :ingredients="ingredientOptions"
          @remove="removeStep(entry.index)"
        />
        <NcButton
          variant="tertiary"
          :disabled="recipe.steps.length >= 100"
          @click="addStep(part.id)"
          ><template #icon><Icon :path="mdiPlus" /></template>{{ t('addStep') }}</NcButton
        >
      </div>
    </div>
  </section>
  <NcButton variant="tertiary" :disabled="recipe.parts.length >= 30" @click="addPart"
    ><template #icon><Icon :path="mdiPlus" /></template>{{ t('addPart') }}</NcButton
  >
</template>
