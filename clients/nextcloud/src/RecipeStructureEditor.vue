<script setup lang="ts">
import { computed } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextField from './LabeledField.vue'
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
  invalid: boolean
}>()
const emit = defineEmits<{ changed: []; create: [index: number, name: string] }>()
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
function addLine() {
  lines.value.push({
    id: `line-${crypto.randomUUID()}`,
    part_id: null,
    ingredient_id: '',
    input: '1',
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
  const id = lines.value[index]?.id
  lines.value.splice(index, 1)
  for (const step of recipe.value.steps)
    step.ingredient_line_ids = step.ingredient_line_ids.filter((ref) => ref !== id)
  emit('changed')
}
function addPart() {
  recipe.value.parts.push({ id: `part-${crypto.randomUUID()}`, name: '', uses_parts: [] })
  emit('changed')
}
function removePart(id: string) {
  if (!window.confirm(t('removePartConfirm'))) return
  recipe.value.parts = recipe.value.parts.filter((p) => p.id !== id)
  for (const p of recipe.value.parts) p.uses_parts = p.uses_parts.filter((ref) => ref !== id)
  for (const line of lines.value) if (line.part_id === id) line.part_id = null
  for (const step of recipe.value.steps) {
    if (step.part_id === id) step.part_id = null
    step.prepared_parts = step.prepared_parts.filter((ref) => ref !== id)
  }
  emit('changed')
}
function removeStep(index: number) {
  recipe.value.steps.splice(index, 1)
  emit('changed')
}
function addStep() {
  recipe.value.steps.push({
    instruction: '',
    active_seconds: 0,
    passive_seconds: 0,
    part_id: null,
    ingredient_line_ids: [],
    prepared_parts: [],
  })
  emit('changed')
}
</script>
<template>
  <section class="editor-section">
    <div class="section-heading">
      <div>
        <h2>{{ t('recipeParts') }}</h2>
        <p class="muted">{{ t('partsHelp') }}</p>
      </div>
      <NcButton :disabled="recipe.parts.length >= 30" @click="addPart"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('addPart') }}</NcButton
      >
    </div>
    <fieldset v-for="(part, index) in recipe.parts" :key="part.id" class="recipe-part">
      <legend>{{ t('recipeParts') }} {{ index + 1 }}</legend>
      <div class="part-heading">
        <NcTextField v-model="part.name" :label="t('partName')" required maxlength="200" /><NcButton
          variant="tertiary"
          :aria-label="`${t('removePart')} ${index + 1}`"
          @click="removePart(part.id)"
          ><template #icon><Icon :path="mdiTrashCanOutline" /></template
        ></NcButton>
      </div>
      <RecipeReferences
        v-model="part.uses_parts"
        :label="t('usesParts')"
        :options="partOptions.filter((p) => p.id !== part.id)"
      />
    </fieldset>
  </section>
  <section class="editor-section">
    <h2>{{ t('lines') }}</h2>
    <div v-for="(line, index) in lines" :key="line.id ?? index" class="ingredient-entry">
      <div class="recipe-ingredient-row">
        <RecipeQuantity
          :model-value="line"
          :index="index"
          :ingredients="catalogue.ingredients"
          :conversions="conversions"
          :invalid="invalid"
          @update:model-value="lines[index] = $event"
          @changed="emit('changed')"
          @create="emit('create', index, $event)"
        /><NcButton
          variant="tertiary"
          :aria-label="`${t('remove')} ${index + 1}`"
          @click="removeLine(index)"
          ><template #icon><Icon :path="mdiTrashCanOutline" /></template
        ></NcButton>
      </div>
      <label v-if="recipe.parts.length" class="field part-assignment"
        >{{ t('belongsTo')
        }}<select v-model="line.part_id">
          <option :value="null">{{ t('mainRecipe') }}</option>
          <option v-for="part in recipe.parts" :key="part.id" :value="part.id">
            {{ part.name || t('partName') }}
          </option>
        </select></label
      >
    </div>
    <NcButton :disabled="lines.length >= 100" @click="addLine"
      ><template #icon><Icon :path="mdiPlus" /></template>{{ t('addLine') }}</NcButton
    >
  </section>
  <section class="editor-section">
    <h2>{{ t('steps') }}</h2>
    <RecipeStepEditor
      v-for="(step, index) in recipe.steps"
      :key="index"
      :model-value="step"
      @update:model-value="recipe.steps[index] = $event"
      :index="index"
      :parts="partOptions"
      :ingredients="ingredientOptions"
      @remove="removeStep(index)"
    />
    <NcButton :disabled="recipe.steps.length >= 100" @click="addStep"
      ><template #icon><Icon :path="mdiPlus" /></template>{{ t('addStep') }}</NcButton
    >
  </section>
</template>
