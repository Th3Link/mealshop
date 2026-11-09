<script setup lang="ts">
import { computed, onMounted, ref, toRaw, watch } from 'vue'
import { onBeforeRouteLeave, onBeforeRouteUpdate, useRoute, useRouter } from 'vue-router'
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextField from '@nextcloud/vue/components/NcTextField'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { api, domain } from './api'
import {
  catalogueSchema,
  nutritionSchema,
  recipeRecordSchema,
  recipesSchema,
  displayLinesSchema,
} from './contracts'
import type { Recipe } from './contracts'
import { homes, useTask, loadCatalogue } from './state'
import { exact, locale, parseGrams, t } from './i18n'
import Nutrition from './Nutrition.vue'
import RecipeQuantity from './RecipeQuantity.vue'
import { conversionsSchema } from './quantity-api'
import type { LineDraft } from './quantity-api'
const route = useRoute(),
  router = useRouter()
const home = computed(() => String(route.params.home))
const writable = computed(() => homes.value.find((h) => h.id === home.value)?.role !== 'reader')
const records = ref<z.infer<typeof recipesSchema>>()
const selected = ref<z.infer<typeof recipeRecordSchema>>()
const catalogue = ref<z.infer<typeof catalogueSchema>>()
const draft = ref<Recipe>()
const conversions = ref<z.infer<typeof conversionsSchema>>({ entries: [] })
const lineDrafts = ref<LineDraft[]>([])
const dirty = ref(false),
  validation = ref(''),
  previewCount = ref(1)
const preview = ref<z.infer<typeof nutritionSchema>>()
const scaledLines = ref<z.infer<typeof displayLinesSchema>>()
const displayUnit = ref<'g' | 'kg'>('g')
try {
  if (localStorage.getItem('mealshop.recipeDisplayUnit') === 'kg') displayUnit.value = 'kg'
} catch {
  /* Preference remains available in memory. */
}
watch(displayUnit, () => {
  try {
    localStorage.setItem('mealshop.recipeDisplayUnit', displayUnit.value)
  } catch {
    /* In-memory fallback. */
  }
  if (selected.value && !editing.value) {
    if (preview.value) void scale()
    else void load()
  }
})
const { busy, error, run, errorMessage } = useTask()
const editing = computed(() => route.params.id === 'new' || route.query.edit === '1')
const listPath = computed(() => `/h/${home.value}/recipes`)
const request = (path: string) => domain(home.value, path)
let loadGeneration = 0
async function load() {
  const generation = ++loadGeneration
  selected.value = undefined
  draft.value = undefined
  records.value = undefined
  const isEditing = editing.value
  await run(async () => {
    validation.value = ''
    preview.value = undefined
    scaledLines.value = undefined
    const id = typeof route.params.id === 'string' ? route.params.id : ''
    if (!id) {
      const list = await api(request('/api/recipes'), recipesSchema)
      if (generation !== loadGeneration) return
      records.value = list
      selected.value = undefined
      draft.value = undefined
      return
    }
    if (isEditing) {
      const data = await Promise.all([loadCatalogue(), api('/api/conversions', conversionsSchema)])
      if (generation !== loadGeneration) return
      catalogue.value = data[0]
      conversions.value = data[1]
    }
    if (id === 'new') {
      draft.value = {
        title: '',
        author: '',
        language: locale.value,
        servings: 1,
        preparation: 'as_recorded',
        lines: [],
        steps: [],
      }
      selected.value = undefined
    } else {
      const record = await api(
        request(`/api/recipes/${encodeURIComponent(id)}?display_unit=${displayUnit.value}`),
        recipeRecordSchema,
      )
      if (generation !== loadGeneration) return
      selected.value = record
      previewCount.value = selected.value.recipe.servings
      draft.value = structuredClone(toRaw(selected.value.recipe))
    }
    lineDrafts.value =
      draft.value?.lines.map((l) => ({
        ingredient_id: l.ingredient_id,
        input: exact(l.quantity?.amount ?? l.grams),
        unit: l.quantity?.unit ?? 'g',
        evidence: l.conversion ? structuredClone(toRaw(l.conversion)) : null,
        mode: l.conversion ? 'saved' : 'manual',
        density: l.conversion ? exact(l.conversion.grams_per_base_unit) : '',
        spoon: l.conversion?.millilitres_per_spoon ? exact(l.conversion.millilitres_per_spoon) : '',
        invalidFields: [],
      })) ?? []
    dirty.value = false
  })
}
watch(() => [route.params.id, route.query.edit], load)
onMounted(load)
onBeforeRouteLeave(() => !dirty.value || window.confirm(t('unsaved')))
onBeforeRouteUpdate(() => !dirty.value || window.confirm(t('unsaved')))
function addLine() {
  lineDrafts.value.push({
    ingredient_id: '',
    input: '1',
    unit: 'g',
    evidence: null,
    mode: 'manual',
    density: '',
    spoon: '',
    invalidFields: [],
  })
  dirty.value = true
}
function removeLine(index: number) {
  lineDrafts.value.splice(index, 1)
  dirty.value = true
}
function removeStep(index: number) {
  draft.value?.steps.splice(index, 1)
  dirty.value = true
}
function addStep() {
  draft.value?.steps.push({ instruction: '', active_seconds: 0, passive_seconds: 0 })
  dirty.value = true
}
async function save() {
  if (busy.value) return
  const recipe = draft.value
  if (!recipe) return
  validation.value = ''
  const lines: Array<Omit<Recipe['lines'][number], 'grams'> & { grams?: string }> = []
  for (const line of lineDrafts.value) {
    const amount = parseGrams(line.input)
    if (!amount || line.invalidFields.length) {
      validation.value = t('quantityError')
      return
    }
    if (line.unit === 'g' || line.unit === 'kg') {
      lines.push({ ingredient_id: line.ingredient_id, quantity: { amount, unit: line.unit } })
      continue
    }
    const evidence = line.evidence ? structuredClone(toRaw(line.evidence)) : null
    if (!evidence) {
      validation.value = t('conversionHelp')
      return
    }
    if (line.mode === 'manual') {
      const density = parseGrams(line.density)
      const spoon = line.spoon ? parseGrams(line.spoon) : null
      if (!density || (['tsp', 'tbsp'].includes(line.unit) && !spoon)) {
        validation.value = t('quantityError')
        return
      }
      evidence.ingredient_id = line.ingredient_id
      evidence.form =
        catalogue.value?.ingredients.find((i) => i.id === line.ingredient_id)?.form ?? ''
      evidence.grams_per_base_unit = density
      evidence.millilitres_per_spoon = spoon
    }
    lines.push({
      ingredient_id: line.ingredient_id,
      quantity: { amount, unit: line.unit },
      conversion: evidence,
    })
  }
  if (
    !recipe.title.trim() ||
    !recipe.author.trim() ||
    !lines.length ||
    lines.some((l) => !l.ingredient_id) ||
    !Number.isInteger(Number(recipe.servings)) ||
    Number(recipe.servings) < 1
  ) {
    validation.value = t('required')
    return
  }
  await run(async () => {
    const body = {
      ...recipe,
      servings: Number(recipe.servings),
      lines,
      steps: recipe.steps.map((s) => ({
        ...s,
        active_seconds: Number(s.active_seconds),
        passive_seconds: Number(s.passive_seconds),
      })),
    }
    const result = await api(
      request(selected.value ? `/api/recipes/${selected.value.id}` : '/api/recipes'),
      recipeRecordSchema,
      {
        method: selected.value ? 'PUT' : 'POST',
        data: selected.value ? { expected_revision: selected.value.revision, recipe: body } : body,
      },
    )
    dirty.value = false
    selected.value = result
    await router.push(`${listPath.value}/${result.id}`)
  })
}
async function scale() {
  if (!selected.value) return
  await run(async () => {
    const result = await api(
      request(
        `/api/recipes/${selected.value?.id}/scale?servings=${previewCount.value}&revision=${selected.value?.revision}&display_unit=${displayUnit.value}`,
      ),
      z.object({
        nutrition: nutritionSchema,
        display_lines: displayLinesSchema,
        lines: z.array(z.object({ ingredient_id: z.string(), grams: z.string() })),
      }),
    )
    preview.value = result.nutrition
    scaledLines.value = result.display_lines
  })
}
</script>
<template>
  <NcNoteCard v-if="error" type="error" role="alert"
    >{{ errorMessage(error)
    }}<NcButton :disabled="busy" @click="load">{{ t('retry') }}</NcButton></NcNoteCard
  >
  <NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
  <template v-if="!route.params.id">
    <header class="page-header">
      <div>
        <h1 tabindex="-1">{{ t('recipes') }}</h1>
        <p>{{ t('recipeIntro') }}</p>
      </div>
      <NcButton v-if="writable" variant="primary" :to="`${listPath}/new`">{{
        t('newRecipe')
      }}</NcButton>
    </header>
    <NcEmptyContent
      v-if="records && !records.recipes.length"
      :name="t('noRecipes')"
      :description="t('noRecipesText')"
    />
    <ul v-if="records" class="record-list">
      <li v-for="record in records.recipes" :key="record.id">
        <RouterLink :to="`${listPath}/${record.id}`"
          ><div>
            <strong :lang="record.language">{{ record.title }}</strong
            ><span>{{ record.servings }} {{ t('servings') }}</span>
          </div>
          <span>{{ t('revision') }} {{ record.revision }}</span></RouterLink
        >
      </li>
    </ul>
  </template>
  <template v-else-if="editing && draft">
    <NcButton :to="selected ? `${listPath}/${selected.id}` : listPath">{{ t('cancel') }}</NcButton>
    <header class="page-header">
      <h1 tabindex="-1">{{ selected ? t('edit') : t('newRecipe') }}</h1>
    </header>
    <NcNoteCard v-if="validation" type="error" role="alert">{{ validation }}</NcNoteCard>
    <form class="narrow" @submit.prevent="save" @input="dirty = true" @change="dirty = true">
      <NcTextField v-model="draft.title" :label="t('title')" required maxlength="200" />
      <NcTextField v-model="draft.author" :label="t('author')" required maxlength="200" />
      <div class="form-row">
        <NcTextField
          v-model="draft.servings"
          type="number"
          :label="t('servings')"
          min="1"
          max="1000000"
          step="1"
          required
        /><label class="field"
          >{{ t('language')
          }}<select v-model="draft.language">
            <option value="en">English</option>
            <option value="de">Deutsch</option>
          </select></label
        >
      </div>
      <label class="field"
        >{{ t('preparation')
        }}<select v-model="draft.preparation">
          <option value="as_recorded">{{ t('asRecorded') }}</option>
          <option value="cooking_unmodelled">{{ t('cooking') }}</option>
        </select></label
      >
      <h2>{{ t('lines') }}</h2>
      <div v-for="(line, index) in lineDrafts" :key="index">
        <RecipeQuantity
          :model-value="line"
          @update:model-value="lineDrafts[index] = $event"
          :index="index"
          :ingredients="catalogue?.ingredients ?? []"
          :conversions="conversions"
          :invalid="Boolean(validation)"
        /><NcButton :aria-label="`${t('remove')} ${index + 1}`" @click="removeLine(index)">{{
          t('remove')
        }}</NcButton>
      </div>
      <NcButton :disabled="lineDrafts.length >= 100" @click="addLine">{{ t('addLine') }}</NcButton>
      <h2>{{ t('steps') }}</h2>
      <div v-for="(step, index) in draft.steps" :key="index">
        <label class="field"
          >{{ index + 1 }}. {{ t('instruction')
          }}<textarea v-model="step.instruction" required maxlength="4000" />
        </label>
        <div class="form-row">
          <NcTextField
            v-model="step.active_seconds"
            type="number"
            :label="t('activeSeconds')"
            min="0"
            max="604800"
            step="1"
          /><NcTextField
            v-model="step.passive_seconds"
            type="number"
            :label="t('passiveSeconds')"
            min="0"
            max="604800"
            step="1"
          />
        </div>
        <NcButton
          :aria-label="`${t('remove')} ${t('instruction')} ${index + 1}`"
          @click="removeStep(index)"
          >{{ t('remove') }}</NcButton
        >
      </div>
      <NcButton :disabled="draft.steps.length >= 100" @click="addStep">{{ t('addStep') }}</NcButton>
      <NcButton variant="primary" type="submit" :disabled="busy || !writable">{{
        t('save')
      }}</NcButton>
    </form>
  </template>
  <template v-else-if="selected">
    <NcButton :to="listPath">{{ t('back') }}</NcButton>
    <header class="page-header">
      <div>
        <h1 tabindex="-1" :lang="selected.recipe.language">{{ selected.recipe.title }}</h1>
        <p>
          {{ selected.recipe.author }} · {{ t('revision') }} {{ selected.revision }} ·
          {{ selected.recipe.servings }} {{ t('servings') }}
        </p>
      </div>
      <NcButton v-if="writable" :to="{ query: { edit: '1' } }">{{ t('edit') }}</NcButton>
    </header>
    <section>
      <h2>{{ t('lines') }}</h2>
      <label class="field"
        >{{ t('displayUnit')
        }}<select v-model="displayUnit" :aria-label="t('displayUnit')">
          <option value="g">g</option>
          <option value="kg">kg</option>
        </select></label
      >
      <ul class="plain-list">
        <li v-for="(line, index) in scaledLines ?? selected.display_lines" :key="index">
          {{ exact(line.amount) }} {{ line.unit }} ·
          {{
            selected.nutrition.ingredients.find((i) => i.ingredient_id === line.ingredient_id)
              ?.name ?? line.ingredient_id
          }}<small v-if="selected.recipe.lines[index]?.quantity">
            · {{ t('originalInput') }}:
            {{ exact(selected.recipe.lines[index]?.quantity?.amount ?? null) }}
            {{ selected.recipe.lines[index]?.quantity?.unit }}</small
          >
        </li>
      </ul>
    </section>
    <section>
      <h2>{{ t('steps') }}</h2>
      <ol class="plain-list">
        <li v-for="(step, index) in selected.recipe.steps" :key="index">
          <p :lang="selected.recipe.language">{{ step.instruction }}</p>
          <small
            >{{ t('activeSeconds') }}: {{ step.active_seconds }} · {{ t('passiveSeconds') }}:
            {{ step.passive_seconds }}</small
          >
        </li>
      </ol>
    </section>
    <form class="inline-row" @submit.prevent="scale">
      <NcTextField
        v-model="previewCount"
        type="number"
        :label="t('previewServings')"
        min="1"
        max="1000000"
        step="1"
        required
      /><NcButton type="submit" :disabled="busy">{{ t('preview') }}</NcButton>
    </form>
    <Nutrition :value="preview ?? selected.nutrition" />
  </template>
</template>
