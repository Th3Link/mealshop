<script setup lang="ts">
import { computed, nextTick, onMounted, ref, toRaw, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextField from './LabeledField.vue'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { api, domain, imageUrl } from './api'
import {
  catalogueSchema,
  nutritionSchema,
  recipeRecordSchema,
  recipesSchema,
  displayLinesSchema,
} from './contracts'
import type { Recipe } from './contracts'
import { homes, useTask, loadCatalogue } from './state'
import { servingCount, exact, locale, parseGrams, t } from './i18n'
import { mdiPlus } from '@mdi/js'
import Icon from './Icon.vue'
import { foldSearch } from './ingredient-search'
import { useUnsavedChanges } from './useUnsavedChanges'
import Nutrition from './Nutrition.vue'
import RecipeStructureEditor from './RecipeStructureEditor.vue'
import RecipeContents from './RecipeContents.vue'
import RecipeOriginEditor from './RecipeOriginEditor.vue'
import ShareControls from './ShareControls.vue'
import IngredientEditor from './IngredientEditor.vue'
import { conversionsSchema } from './quantity-api'
import type { LineDraft } from './quantity-api'
const route = useRoute(),
  router = useRouter()
const home = computed(() => String(route.params.home))
const writable = computed(() => homes.value.some((h) => h.id === home.value && h.role !== 'reader'))
const records = ref<z.infer<typeof recipesSchema>>()
const selected = ref<z.infer<typeof recipeRecordSchema>>()
const latestRevision = ref<number>()
const historical = computed(() =>
  Boolean(
    selected.value &&
    latestRevision.value !== undefined &&
    selected.value.revision !== latestRevision.value,
  ),
)
const sourceOrigin = computed(() => selected.value?.sharing_origin ?? selected.value?.recipe.origin)
const owner = computed(() => homes.value.some((h) => h.id === home.value && h.role === 'owner'))
const catalogue = ref<z.infer<typeof catalogueSchema>>()
const draft = ref<Recipe>()
const conversions = ref<z.infer<typeof conversionsSchema>>({ entries: [] })
const lineDrafts = ref<LineDraft[]>([])
const creatingIngredient = ref<{ index: number; name: string } | null>(null)
async function ingredientCreated(id: string) {
  const index = creatingIngredient.value?.index
  catalogue.value = await loadCatalogue(home.value)
  const line = index === undefined ? undefined : lineDrafts.value[index]
  if (line) {
    line.ingredient_id = id
    line.evidence = null
    line.mode = 'manual'
    line.density = ''
    line.spoon = ''
    line.invalidFields = []
  }
  creatingIngredient.value = null
  dirty.value = true
}
const editing = computed(
  () => (route.params.id === 'new' || route.query.edit === '1') && !historical.value,
)
const dirty = ref(false),
  validation = ref(''),
  previewCount = ref(1)
watch(
  [draft, lineDrafts],
  () => {
    if (editing.value && draft.value) dirty.value = true
  },
  { deep: true, flush: 'sync' },
)
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
const listPath = computed(() => `/h/${home.value}/recipes`)
const search = computed({
  get: () => (typeof route.query.q === 'string' ? route.query.q : ''),
  set: (q: string) => {
    void router.replace({ query: { ...route.query, q: q || undefined } })
  },
})
const matchingRecipes = computed(
  () =>
    records.value?.recipes.filter((record) =>
      search.value
        .trim()
        .split(/\s+/)
        .every((term) => foldSearch(record.title).includes(foldSearch(term))),
    ) ?? [],
)
const confirmDiscard = useUnsavedChanges(() => dirty.value && creatingIngredient.value === null)
async function reload() {
  if (confirmDiscard()) await load()
}
const request = (path: string) => domain(home.value, path)
let loadGeneration = 0
async function load() {
  const generation = ++loadGeneration
  dirty.value = false
  selected.value = undefined
  latestRevision.value = undefined
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
      const data = await Promise.all([
        loadCatalogue(home.value),
        api('/api/conversions', conversionsSchema),
      ])
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
        parts: [],
      }
      selected.value = undefined
    } else {
      const revision = typeof route.query.revision === 'string' ? route.query.revision : undefined
      const query = new URLSearchParams({
        display_unit: displayUnit.value,
        ...(revision ? { revision } : {}),
      })
      const record = await api(
        request(`/api/recipes/${encodeURIComponent(id)}?${query}`),
        recipeRecordSchema,
      )
      const current = revision
        ? await api(request(`/api/recipes/${encodeURIComponent(id)}`), recipeRecordSchema)
        : record
      if (generation !== loadGeneration) return
      latestRevision.value = current.revision
      selected.value = record
      previewCount.value = selected.value.recipe.servings
      draft.value = structuredClone(toRaw(selected.value.recipe))
    }
    lineDrafts.value =
      draft.value?.lines.map((l) => ({
        id: l.id ?? `line-${crypto.randomUUID()}`,
        part_id: l.part_id ?? null,
        prepared_part: l.prepared_part ?? null,
        extensions: structuredClone(toRaw(l.extensions ?? [])),
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
  if (generation === loadGeneration) {
    await nextTick()
    document.querySelector<HTMLElement>('.mealshop h1')?.focus()
  }
}
watch([() => route.params.id, () => route.query.edit, () => route.query.revision], load)
onMounted(load)
function removeImage() {
  if (!draft.value) return
  draft.value.image_id = null
  dirty.value = true
}
async function uploadImage(event: Event) {
  if (!(event.target instanceof HTMLInputElement) || !draft.value) return
  const file = event.target.files?.[0]
  event.target.value = ''
  if (!file) return
  if (
    file.size > 5 * 1024 * 1024 ||
    !['image/jpeg', 'image/png', 'image/webp'].includes(file.type)
  ) {
    validation.value = t('imageTooLarge')
    return
  }
  const current = draft.value
  await run(async () => {
    const result = await api(request('/api/household/images'), z.object({ id: z.string() }), {
      method: 'POST',
      data: file,
      contentType: file.type,
    })
    if (draft.value === current) {
      current.image_id = result.id
      dirty.value = true
      validation.value = ''
    }
  })
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
      validation.value = t(
        line.invalidFields.includes('temperature') ? 'temperatureError' : 'quantityError',
      )
      return
    }
    if (line.unit === 'g' || line.unit === 'kg') {
      lines.push({
        extensions: structuredClone(toRaw(line.extensions ?? [])),
        id: line.id,
        part_id: line.part_id,
        prepared_part: line.prepared_part,
        ingredient_id: line.ingredient_id,
        quantity: { amount, unit: line.unit },
      })
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
      extensions: structuredClone(toRaw(line.extensions ?? [])),
      id: line.id,
      part_id: line.part_id,
      prepared_part: line.prepared_part,
      ingredient_id: line.ingredient_id,
      quantity: { amount, unit: line.unit },
      conversion: evidence,
    })
  }
  if (
    !recipe.title.trim() ||
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
    await router.push({
      path: `${listPath.value}/${result.id}`,
      query: { q: search.value || undefined },
    })
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
  <header v-if="route.params.id && !selected && !draft && !busy" class="page-header">
    <h1 tabindex="-1">{{ t('recipes') }}</h1>
    <NcButton :to="listPath">{{ t('back') }}</NcButton>
  </header>
  <NcNoteCard v-if="error" type="error" role="alert"
    >{{ errorMessage(error)
    }}<NcButton :disabled="busy" @click="reload">{{
      t(dirty ? 'reloadSaved' : 'retry')
    }}</NcButton></NcNoteCard
  >
  <NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
  <template v-if="!route.params.id">
    <header class="page-header">
      <div>
        <h1 tabindex="-1">{{ t('recipes') }}</h1>
        <p>{{ t('recipeIntro') }}</p>
      </div>
      <NcButton v-if="writable" variant="primary" :to="`${listPath}/new`"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('newRecipe') }}</NcButton
      >
    </header>
    <div v-if="records?.recipes.length" class="toolbar">
      <NcTextField
        v-model="search"
        type="search"
        :label="t('searchRecipes')"
        :show-trailing-button="Boolean(search)"
        trailing-button-icon="close"
        :trailing-button-label="t('clearSearch')"
        @trailing-button-click="search = ''"
      />
      <p role="status">{{ t('recipes') }}: {{ matchingRecipes.length }}</p>
    </div>
    <NcEmptyContent
      v-if="records && !records.recipes.length"
      :name="t('noRecipes')"
      :description="t('noRecipesText')"
    />
    <NcEmptyContent
      v-if="records?.recipes.length && !matchingRecipes.length"
      :name="t('noResults')"
    >
      <template #action
        ><NcButton @click="search = ''">{{ t('clearSearch') }}</NcButton></template
      >
    </NcEmptyContent>
    <ul v-if="records" class="record-list">
      <li v-for="record in matchingRecipes" :key="record.id">
        <RouterLink :to="{ path: `${listPath}/${record.id}`, query: route.query }"
          ><img
            v-if="record.image_id"
            :src="imageUrl(home, record.image_id)"
            alt=""
            class="recipe-thumbnail"
            loading="lazy"
            width="64"
            height="64"
          />
          <div>
            <strong :lang="record.language">{{ record.title }}</strong
            ><span>{{ servingCount(record.servings) }}</span
            ><span v-if="record.language !== locale" class="muted"
              >{{ t('originalLanguage') }}:
              {{ record.language === 'de' ? 'Deutsch' : 'English' }}</span
            >
          </div>
          <span>{{ t('revision') }} {{ record.revision }}</span></RouterLink
        >
      </li>
    </ul>
  </template>
  <template v-else-if="editing && draft">
    <NcButton
      :disabled="busy"
      :to="{
        path: selected ? `${listPath}/${selected.id}` : listPath,
        query: { q: search || undefined },
      }"
      >{{ t('cancel') }}</NcButton
    >
    <header class="page-header recipe-editor-header">
      <h1 tabindex="-1">{{ selected ? t('edit') : t('newRecipe') }}</h1>
      <NcButton
        variant="primary"
        type="submit"
        form="mealshop-recipe-form"
        :disabled="busy || !writable"
        >{{ busy ? t('saving') : t('save') }}</NcButton
      >
    </header>
    <NcNoteCard v-if="validation" type="error" role="alert">{{ validation }}</NcNoteCard>
    <form
      id="mealshop-recipe-form"
      class="recipe-editor"
      @submit.prevent="save"
      @input="dirty = true"
      @change="dirty = true"
    >
      <fieldset class="editor-fields" :disabled="busy || !writable">
        <div class="recipe-title-row">
          <NcTextField v-model="draft.title" :label="t('title')" required maxlength="200" />
          <div class="form-row recipe-basics">
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
        </div>
        <RecipeStructureEditor
          v-if="catalogue"
          v-model:recipe="draft"
          v-model:lines="lineDrafts"
          :catalogue="catalogue"
          :conversions="conversions"
          :invalid="Boolean(validation)"
          @changed="dirty = true"
          @create="(index, name) => (creatingIngredient = { index, name })"
        />
        <section class="recipe-metadata editor-section">
          <h2>{{ t('optionalDetails') }}</h2>
          <div class="editor-fields">
            <NcTextField
              v-model="draft.author"
              :label="t('author')"
              :helper-text="t('optional')"
              maxlength="200"
            />
            <RecipeOriginEditor
              v-model="draft.origin"
              :locked="sourceOrigin?.kind === 'imported'"
              @update:model-value="dirty = true"
            />
            <fieldset class="recipe-image-editor">
              <legend>{{ t('image') }}</legend>
              <img
                v-if="draft.image_id"
                :src="imageUrl(home, draft.image_id)"
                :alt="draft.title"
                class="recipe-image"
              />
              <label class="field"
                >{{ t('image')
                }}<input
                  type="file"
                  accept="image/jpeg,image/png,image/webp"
                  :disabled="busy"
                  @change="uploadImage"
              /></label>
              <p class="muted">{{ t('imageHelp') }}</p>
              <NcButton v-if="draft.image_id" @click="removeImage">{{ t('removeImage') }}</NcButton>
            </fieldset>
            <label class="field"
              >{{ t('preparation')
              }}<select v-model="draft.preparation">
                <option value="as_recorded">{{ t('asRecorded') }}</option>
                <option value="cooking_unmodelled">{{ t('cooking') }}</option>
              </select></label
            >
          </div>
        </section>
      </fieldset>
    </form>
  </template>
  <template v-else-if="selected">
    <NcButton
      :to="
        typeof route.query.collection === 'string'
          ? {
              path: `/h/${home}/collections/${route.query.collection}`,
              query: { page: route.query.collectionPage },
            }
          : typeof route.query.history === 'string'
            ? { path: `/h/${home}/history/${encodeURIComponent(route.query.history)}` }
            : { path: listPath, query: { q: search || undefined } }
      "
      >{{ t('back') }}</NcButton
    >
    <NcNoteCard v-if="historical" type="info"
      >{{ t('historicalRecipeHelp') }}
      <NcButton :to="{ query: { ...route.query, revision: undefined, edit: undefined } }">{{
        t('currentRecipe')
      }}</NcButton></NcNoteCard
    >
    <header class="page-header">
      <div>
        <h1 tabindex="-1" :lang="selected.recipe.language">{{ selected.recipe.title }}</h1>
        <p v-if="selected.recipe.language !== locale" class="muted">
          {{ t('originalLanguage') }}:
          {{ selected.recipe.language === 'de' ? 'Deutsch' : 'English' }}
        </p>
        <p>
          <template v-if="selected.recipe.author">{{ selected.recipe.author }} · </template
          >{{ t('revision') }} {{ selected.revision }} ·
          {{ servingCount(selected.recipe.servings) }}
        </p>
      </div>
      <div class="toolbar">
        <ShareControls
          v-if="owner"
          :home="home"
          :title="selected.recipe.title"
          :resource="{ kind: 'recipe', recipe_id: selected.id, revision: selected.revision }"
          :source-only="sourceOrigin?.kind === 'imported'"
        /><NcButton v-if="writable && !historical" :to="{ query: { ...route.query, edit: '1' } }">{{
          t('edit')
        }}</NcButton>
      </div>
    </header>
    <p v-if="sourceOrigin?.kind === 'imported'">
      <a :href="sourceOrigin.source_url" target="_blank" rel="noopener noreferrer nofollow">{{
        t('originalRecipe')
      }}</a>
    </p>
    <img
      v-if="selected.recipe.image_id"
      :src="imageUrl(home, selected.recipe.image_id)"
      :alt="selected.recipe.title"
      class="recipe-image"
    />
    <div class="recipe-reading-toolbar">
      <form class="recipe-scale" @submit.prevent="scale">
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

      <label class="field"
        >{{ t('displayUnit')
        }}<select v-model="displayUnit" :aria-label="t('displayUnit')">
          <option value="g">g</option>
          <option value="kg">kg</option>
        </select></label
      >

      <NcButton
        v-if="writable && selected.recipe.steps.length"
        :to="{
          path: `/h/${home}/cooking/new`,
          query: { recipe: selected.id, revision: selected.revision, servings: previewCount },
        }"
        >{{ t('cookNew') }}</NcButton
      >
    </div>
    <RecipeContents :record="selected" :lines="scaledLines ?? selected.display_lines" />
    <Nutrition :value="preview ?? selected.nutrition" />
  </template>
  <IngredientEditor
    v-if="creatingIngredient !== null && catalogue"
    :home="home"
    :initial-name="creatingIngredient.name"
    :parent-dirty="dirty"
    :catalogue="catalogue"
    @close="creatingIngredient = null"
    @saved="ingredientCreated"
  />
</template>
