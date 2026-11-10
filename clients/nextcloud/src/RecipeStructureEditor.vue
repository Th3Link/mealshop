<script setup lang="ts">
import { computed, ref } from 'vue'
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
const openParts = ref(new Set<string>())
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
function addLine(partId: string | null = null) {
  lines.value.push({
    id: `line-${crypto.randomUUID()}`,
    part_id: partId,
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
  const id = `part-${crypto.randomUUID()}`
  recipe.value.parts.push({ id, name: '', uses_parts: [], starts_after: [] })
  openParts.value.add(id)
  emit('changed')
}
function removePart(id: string) {
  if (!window.confirm(t('removePartConfirm'))) return
  recipe.value.parts = recipe.value.parts.filter((p) => p.id !== id)
  openParts.value.delete(id)
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
  <section
    v-for="part in sections"
    :key="part.id ?? 'main'"
    class="editor-section recipe-part-content"
  >
    <component
      :is="recipe.parts.length ? 'details' : 'div'"
      :open="part.id === null || openParts.has(part.id) || recipe.parts.length === 1"
      class="part-editor"
    >
      <summary v-if="recipe.parts.length">{{ part.name || t('partName') }}</summary>
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
          <NcButton variant="tertiary" :aria-label="t('removePart')" @click="removePart(part.id)"
            ><template #icon><Icon :path="mdiTrashCanOutline" /></template
          ></NcButton>
        </div>
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
      </template>
      <h2>{{ t('lines') }}</h2>
      <div
        v-for="entry in lines
          .map((line, index) => ({ line, index }))
          .filter((e) => (e.line.part_id ?? null) === part.id)"
        :key="entry.line.id ?? entry.index"
        class="ingredient-entry"
      >
        <div class="recipe-ingredient-row">
          <RecipeQuantity
            :model-value="entry.line"
            :index="entry.index"
            :ingredients="catalogue.ingredients"
            :conversions="conversions"
            :invalid="invalid"
            @update:model-value="lines[entry.index] = $event"
            @changed="emit('changed')"
            @create="emit('create', entry.index, $event)"
          />
          <NcButton
            variant="tertiary"
            :aria-label="`${t('remove')} ${entry.index + 1}`"
            @click="removeLine(entry.index)"
            ><template #icon><Icon :path="mdiTrashCanOutline" /></template
          ></NcButton>
        </div>
        <details v-if="recipe.parts.length">
          <summary>{{ t('moveToPart') }}</summary>
          <label class="field"
            >{{ t('belongsTo')
            }}<select v-model="entry.line.part_id">
              <option :value="null">{{ t('mainRecipe') }}</option>
              <option v-for="p in recipe.parts" :key="p.id" :value="p.id">{{ p.name }}</option>
            </select></label
          >
        </details>
      </div>
      <NcButton :disabled="lines.length >= 100" @click="addLine(part.id)"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('addLine') }}</NcButton
      >
      <h2>{{ t('steps') }}</h2>
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
      <NcButton :disabled="recipe.steps.length >= 100" @click="addStep(part.id)"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('addStep') }}</NcButton
      >
    </component>
  </section>
  <NcButton :disabled="recipe.parts.length >= 30" @click="addPart"
    ><template #icon><Icon :path="mdiPlus" /></template>{{ t('addPart') }}</NcButton
  >
</template>
