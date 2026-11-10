<script setup lang="ts">
import { computed, nextTick, onMounted, ref, toRaw, watch } from 'vue'
import { useRoute } from 'vue-router'
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import CookingDuration from './CookingDuration.vue'
import NcAppSidebar from '@nextcloud/vue/components/NcAppSidebar'
import NcAppSidebarTab from '@nextcloud/vue/components/NcAppSidebarTab'
import NcTextField from './LabeledField.vue'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { api, domain } from './api'
import { catalogueSchema, emptySchema, profilesSchema, recipesSchema } from './contracts'
import { plansSchema, previewSchema } from './planning-api'
import type { Plan, PlanReview as ReviewValue } from './planning-api'
import { homes, useTask, loadCatalogue } from './state'
import { cookingListSchema } from './cooking-api'
import { compactExact, locale, t, explanation } from './i18n'
import PlanReview from './PlanReview.vue'
import MealCalendar from './MealCalendar.vue'
import { useUnsavedChanges } from './useUnsavedChanges'
const route = useRoute(),
  home = computed(() => String(route.params.home))
const writable = computed(() => homes.value.some((h) => h.id === home.value && h.role !== 'reader'))
const plans = ref<z.infer<typeof plansSchema>>(),
  profiles = ref<z.infer<typeof profilesSchema>>(),
  recipes = ref<z.infer<typeof recipesSchema>>(),
  catalogue = ref<z.infer<typeof catalogueSchema>>()
const cooking = ref<z.infer<typeof cookingListSchema>>()
const calendarDate = ref(new Date().toLocaleDateString('en-CA'))
const draft = ref<Plan>(),
  reviewed = ref(false)
const initialDraft = ref('')
const editingExisting = ref(false)
const confirmDiscard = useUnsavedChanges(
  () => Boolean(draft.value) && JSON.stringify(draft.value) !== initialDraft.value,
)
async function reload() {
  if (confirmDiscard()) {
    draft.value = undefined
    result.value = undefined
    await load()
  }
}
const result = ref<ReviewValue>()
const suggestionSchema = previewSchema.extend({ plan: planSchema })
const suggestions = ref<z.infer<typeof suggestionSchema>[]>([])
const rejected = ref<string[]>([])
const { busy, error, run, errorMessage } = useTask()
const path = (suffix: string) => domain(home.value, `/api/household/mealplan${suffix}`)
const names = (id: string) => {
  const ingredient = catalogue.value?.ingredients.find((i) => i.id === id)
  return ingredient?.translations[locale.value]?.name ?? ingredient?.name ?? id
}
const recipeOptions = computed(() => {
  const options = [...(recipes.value?.recipes ?? [])]
  const pinned = draft.value?.recipe
  if (pinned && !options.some((r) => r.id === pinned.id && r.revision === pinned.revision)) {
    options.push({
      id: pinned.id,
      revision: pinned.revision,
      title:
        plans.value?.recipe_revisions.find(
          (r) => r.id === pinned.id && r.revision === pinned.revision,
        )?.title ?? pinned.id,
      language: 'en',
      servings: 1,
    })
  }
  return options
})
const title = (plan: Plan) =>
  plans.value?.recipe_revisions.find(
    (r) => r.id === plan.recipe.id && r.revision === plan.recipe.revision,
  )?.title ??
  recipes.value?.recipes.find((r) => r.id === plan.recipe.id && r.revision === plan.recipe.revision)
    ?.title ??
  plan.recipe.id
const mealLabel = (meal: string) =>
  meal === 'preparation'
    ? t('prepareSlot')
    : meal === 'breakfast' || meal === 'lunch' || meal === 'dinner' || meal === 'snack'
      ? t(meal)
      : meal
async function load() {
  await run(async () => {
    const data = await Promise.all([
      api(path(''), plansSchema),
      api(domain(home.value, '/api/household/profiles'), profilesSchema),
      api(domain(home.value, '/api/recipes'), recipesSchema),
      loadCatalogue(home.value),
      api(domain(home.value, '/api/household/cooking'), cookingListSchema),
    ])
    plans.value = data[0]
    profiles.value = data[1]
    recipes.value = data[2]
    catalogue.value = data[3]
    cooking.value = data[4]
  })
}
function start(plan?: Plan) {
  if (!confirmDiscard()) return false
  editingExisting.value = Boolean(plan)
  draft.value = plan
    ? structuredClone(toRaw(plan))
    : {
        date: calendarDate.value,
        meal: profiles.value?.members.length ? 'dinner' : 'preparation',
        preparation: null,
        recipe: recipes.value?.recipes[0]
          ? { id: recipes.value.recipes[0].id, revision: recipes.value.recipes[0].revision }
          : { id: '', revision: 1 },
        attendees: profiles.value?.members.map((m) => m.id) ?? [],
        calendar: null,
        servings: profiles.value?.members.length || 1,
        active_seconds: 86400,
        elapsed_seconds: 86400,
        advance_seconds: 2592000,
      }
  if (!plan && draft.value.meal === 'preparation') draft.value.allocations = []
  if (plan && !draft.value.allocations && plan.meal !== 'preparation')
    draft.value.allocations = [{ date: plan.date, meal: plan.meal, servings: plan.servings }]
  result.value = undefined
  suggestions.value = []
  rejected.value = []
  initialDraft.value = JSON.stringify(draft.value)
  void nextTick().then(() =>
    document.querySelector<HTMLElement>('.mealplan-sidebar .app-sidebar-header__mainname')?.focus(),
  )
  return true
}
function selectSlot(date: string, meal: string, plan?: Plan) {
  if (!start(plan)) return
  calendarDate.value = date
  if (draft.value && !plan) {
    draft.value.date = date
    draft.value.meal = meal
    if (meal === 'preparation') draft.value.allocations = []
    initialDraft.value = JSON.stringify(draft.value)
  }
}
function cancelPlan() {
  if (!confirmDiscard()) return
  draft.value = undefined
  result.value = undefined
  suggestions.value = []
  rejected.value = []
  void nextTick().then(() => document.getElementById('mealshop-calendar-title')?.focus())
}
function selectedRecipe(event: Event) {
  if (event.target instanceof HTMLSelectElement && draft.value) {
    const value = event.target.value
    const r = recipeOptions.value.find((r) => `${r.id}@${r.revision}` === value)
    if (r) draft.value.recipe = { id: r.id, revision: r.revision }
  }
}
function setCalendar(value: unknown) {
  if (!draft.value) return
  if (value === true) {
    draft.value.calendar = {
      consent: true,
      source: '',
      event_id: '',
      date: draft.value.date,
      member_ids: [...(draft.value.attendees ?? [])],
    }
  } else {
    if (draft.value.attendees === null)
      draft.value.attendees = [...(draft.value.calendar?.member_ids ?? [])]
    draft.value.calendar = null
  }
}
function payload(): Plan | null {
  if (!draft.value) return null
  const p = structuredClone(toRaw(draft.value))
  p.servings = Number(p.servings)
  if (p.allocations)
    p.allocations = p.allocations.map((a) => ({ ...a, servings: Number(a.servings) }))
  p.active_seconds = Number(p.active_seconds)
  p.elapsed_seconds = Number(p.elapsed_seconds)
  p.advance_seconds = Number(p.advance_seconds)
  if (p.calendar) p.calendar.date = p.date
  return p
}
function addEatingDay() {
  if (!draft.value) return
  if (!draft.value.allocations)
    draft.value.allocations =
      draft.value.meal === 'preparation'
        ? []
        : [{ date: draft.value.date, meal: draft.value.meal, servings: draft.value.servings }]
  draft.value.allocations.push({ date: draft.value.date, meal: 'lunch', servings: 1 })
}
function addPreparation() {
  if (draft.value) draft.value.preparation = { date: draft.value.date, time: '' }
}
async function preview(remove = false, plan = payload()) {
  if (!plan) return
  await run(async () => {
    const preview = await api(path(remove ? '/remove-preview' : '/preview'), previewSchema, {
      method: 'POST',
      data: plan,
    })
    result.value = { plan, preview, remove }
    reviewed.value = false
  })
}
async function apply() {
  const snapshot = result.value
  if (!snapshot || !reviewed.value || !snapshot.preview.feasible) return
  await run(async () => {
    await api(path('/apply'), emptySchema, {
      method: 'POST',
      data: {
        plan: snapshot.plan,
        expected_revision: snapshot.preview.plan_revision,
        profile_revision: snapshot.preview.profile_revision,
        reviewed: true,
        remove: snapshot.remove,
      },
    })
    calendarDate.value = snapshot.plan.date
    draft.value = undefined
    result.value = undefined
    await load()
    await nextTick()
    document.getElementById('mealshop-calendar-title')?.focus()
  })
}
async function suggest() {
  const template = payload()
  if (!template) return
  await run(async () => {
    suggestions.value = []
    rejected.value = []
    const candidates = recipes.value?.recipes ?? []
    for (let offset = 0; offset < candidates.length; offset += 50) {
      const data = await api(
        path('/suggestions'),
        z.object({
          suggestions: z.array(suggestionSchema),
          rejected: z.array(
            z.object({ conflicts: z.array(z.string()).optional(), error: z.string().optional() }),
          ),
        }),
        {
          method: 'POST',
          data: {
            template,
            candidates: candidates
              .slice(offset, offset + 50)
              .map((r) => ({ id: r.id, revision: r.revision })),
          },
        },
      )
      suggestions.value.push(...data.suggestions)
      rejected.value.push(
        ...data.rejected.flatMap((r) => r.conflicts ?? (r.error ? [r.error] : [])),
      )
    }
  })
}
import { planSchema } from './planning-api'
watch(
  draft,
  () => {
    result.value = undefined
    reviewed.value = false
    suggestions.value = []
  },
  { deep: true },
)
watch(result, async (value) => {
  reviewed.value = false
  if (value) {
    await nextTick()
    document.getElementById('mealshop-review-title')?.focus()
  }
})
onMounted(load)
</script>
<template>
  <header class="page-header">
    <div>
      <h1 tabindex="-1">{{ t('plan') }}</h1>
    </div>
    <NcButton
      v-if="writable && !draft"
      variant="primary"
      :disabled="busy || !recipes?.recipes.length"
      @click="start()"
      >{{ t('newMeal') }}</NcButton
    >
  </header>
  <NcNoteCard v-if="error" type="error" role="alert"
    >{{ errorMessage(error)
    }}<NcButton :disabled="busy" @click="reload">{{
      t(draft ? 'reloadSaved' : 'retry')
    }}</NcButton></NcNoteCard
  ><NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
  <NcNoteCard v-if="plans && profiles && recipes && !recipes.recipes.length" type="info">
    {{ t('planPrerequisites') }}
    <div class="actions">
      <NcButton :to="`/h/${home}/recipes`">{{ t('recipes') }}</NcButton
      ><NcButton :to="`/h/${home}/household`">{{ t('profiles') }}</NcButton>
    </div>
  </NcNoteCard>
  <NcEmptyContent
    v-if="plans && !plans.meals.length && !draft"
    :name="t('emptyPlan')"
    :description="t('emptyPlanText')"
    ><template #action
      ><div class="actions">
        <NcButton :to="`/h/${home}/recipes`">{{ t('recipes') }}</NcButton
        ><NcButton :to="`/h/${home}/household`">{{ t('profiles') }}</NcButton>
      </div></template
    ></NcEmptyContent
  >
  <MealCalendar
    v-model:date="calendarDate"
    v-if="plans"
    :meals="plans.meals"
    :runs="cooking?.runs ?? []"
    :home="home"
    :members="profiles?.members ?? []"
    :title="title"
    :writable="writable"
    :disabled="busy || !recipes?.recipes.length"
    :review-slots="plans.review_required_slots"
    @select="selectSlot"
  />
  <NcAppSidebar
    v-if="draft"
    class="mealplan-sidebar"
    :name="t(editingExisting ? 'editMeal' : 'newMeal')"
    @close="cancelPlan"
  >
    <NcAppSidebarTab id="meal-details" :name="t('details')"
      ><div class="mealplan-details">
        <NcButton
          v-if="
            writable &&
            plans?.meals.some((plan) => plan.date === draft?.date && plan.meal === draft?.meal)
          "
          :disabled="busy"
          @click="preview(true)"
          >{{ t('remove') }}</NcButton
        >
        <NcButton
          v-if="editingExisting"
          :to="{
            path: `/h/${home}/cooking/new`,
            query: {
              recipe: draft.recipe.id,
              revision: String(draft.recipe.revision),
              servings: String(draft.servings),
              plan_date: draft.date,
              plan_meal: draft.meal,
              preparation: draft.preparation
                ? `${draft.preparation.date}T${draft.preparation.time}`
                : undefined,
            },
          }"
          >{{ t('cookingSessions') }}</NcButton
        >
        <form @submit.prevent="preview(false)">
          <fieldset
            class="editor-fields"
            :disabled="busy"
            :aria-label="t(editingExisting ? 'editMeal' : 'newMeal')"
          >
            <div v-if="!editingExisting" class="form-row">
              <label class="field"
                >{{ t('date') }}<input v-model="draft.date" type="date" required /></label
              ><label class="field"
                >{{ t('meal')
                }}<select v-model="draft.meal">
                  <option
                    v-if="
                      !['preparation', 'breakfast', 'lunch', 'dinner', 'snack'].includes(draft.meal)
                    "
                    :value="draft.meal"
                  >
                    {{ draft.meal }}
                  </option>
                  <option
                    v-for="meal in ['preparation', 'breakfast', 'lunch', 'dinner', 'snack']"
                    :key="meal"
                    :value="meal"
                  >
                    {{ mealLabel(meal) }}
                  </option>
                </select></label
              >
            </div>
            <label class="field"
              >{{ t('recipes')
              }}<select
                :value="`${draft.recipe.id}@${draft.recipe.revision}`"
                required
                @change="selectedRecipe"
              >
                <option value="" disabled>{{ t('recipes') }}</option>
                <option
                  v-for="r in recipeOptions"
                  :key="`${r.id}@${r.revision}`"
                  :value="`${r.id}@${r.revision}`"
                >
                  {{ r.title }} · {{ t('revision') }} {{ r.revision }}
                </option>
              </select></label
            >
            <fieldset v-if="draft.attendees">
              <legend>{{ t('attendees') }}</legend>
              <p class="muted">{{ t('attendanceHelp') }}</p>
              <NcCheckboxRadioSwitch
                v-for="member in profiles?.members"
                :key="member.id"
                v-model="draft.attendees"
                :value="member.id"
                >{{ member.name }}</NcCheckboxRadioSwitch
              >
            </fieldset>
            <NcTextField
              v-model="draft.servings"
              type="number"
              :label="t('servings')"
              min="1"
              max="1000000"
              step="1"
              required
            />
            <p class="muted">{{ t('productionHelp') }}</p>
            <NcButton v-if="!draft.preparation" @click="addPreparation">{{
              t('prepareSlot')
            }}</NcButton>
            <div v-else class="form-row">
              <label class="field"
                >{{ t('prepareSlot') }}<input v-model="draft.preparation.date" type="date" required
              /></label>
              <label class="field"
                >{{ t('preparationTime')
                }}<input v-model="draft.preparation.time" type="time" required
              /></label>
              <NcButton @click="draft.preparation = null">{{ t('remove') }}</NcButton>
            </div>
            <fieldset v-if="draft.allocations">
              <legend>{{ t('allocations') }}</legend>
              <div v-for="(allocation, index) in draft.allocations" :key="index" class="form-row">
                <label class="field"
                  >{{ t('date') }}<input v-model="allocation.date" type="date" required
                /></label>
                <label class="field"
                  >{{ t('meal')
                  }}<select v-model="allocation.meal">
                    <option
                      v-for="meal in ['breakfast', 'lunch', 'snack', 'dinner']"
                      :key="meal"
                      :value="meal"
                    >
                      {{ mealLabel(meal) }}
                    </option>
                  </select></label
                >
                <NcTextField
                  v-model="allocation.servings"
                  :label="t('servings')"
                  type="number"
                  min="1"
                  :max="draft.servings"
                  required
                />
                <NcButton @click="draft.allocations.splice(index, 1)">{{ t('remove') }}</NcButton>
              </div>
              <p>
                {{ t('allocatedPortions') }}:
                {{ draft.allocations.reduce((sum, a) => sum + Number(a.servings), 0) }} /
                {{ draft.servings }}
              </p>
            </fieldset>
            <NcButton @click="addEatingDay">{{ t('addEatingDay') }}</NcButton>
            <details>
              <summary>{{ t('timeLimits') }}</summary>
              <CookingDuration
                v-model="draft.active_seconds"
                :label="t('activeBudget')"
                :max="86400"
              /><CookingDuration
                v-model="draft.elapsed_seconds"
                :label="t('elapsedBudget')"
                :max="86400"
              /><CookingDuration
                v-model="draft.advance_seconds"
                :label="t('advanceBudget')"
                :max="2592000"
              />
            </details>
            <details>
              <summary>{{ t('calendar') }}</summary>
              <p>{{ t('calendarHelp') }}</p>
              <NcCheckboxRadioSwitch
                :model-value="Boolean(draft.calendar)"
                @update:model-value="setCalendar"
                >{{ t('consent') }}</NcCheckboxRadioSwitch
              ><template v-if="draft.calendar"
                ><NcCheckboxRadioSwitch
                  :model-value="draft.attendees !== null"
                  @update:model-value="
                    draft.attendees = $event ? [...draft.calendar.member_ids] : null
                  "
                  >{{ t('overrideAttendance') }}</NcCheckboxRadioSwitch
                ><NcTextField
                  v-model="draft.calendar.source"
                  :label="t('calendarSource')"
                  required
                  maxlength="200"
                /><NcTextField
                  v-model="draft.calendar.event_id"
                  :label="t('eventId')"
                  required
                  maxlength="200"
                />
                <fieldset>
                  <legend>{{ t('attendees') }}</legend>
                  <NcCheckboxRadioSwitch
                    v-for="m in profiles?.members"
                    :key="m.id"
                    v-model="draft.calendar.member_ids"
                    :value="m.id"
                    >{{ m.name }}</NcCheckboxRadioSwitch
                  >
                </fieldset></template
              >
            </details>
            <div class="actions editor-actions">
              <NcButton type="submit" variant="primary" :disabled="busy">{{ t('review') }}</NcButton
              ><NcButton :disabled="busy" @click="suggest">{{ t('suggestions') }}</NcButton
              ><NcButton :disabled="busy" @click="cancelPlan">{{ t('cancel') }}</NcButton>
            </div>
          </fieldset>
        </form>
        <section v-if="suggestions.length">
          <h2>{{ t('suggestions') }}</h2>
          <div
            v-for="suggestion in suggestions"
            :key="`${suggestion.plan.recipe.id}@${suggestion.plan.recipe.revision}`"
            class="inline-row"
          >
            <span>{{ title(suggestion.plan) }}</span
            ><NcButton
              :disabled="busy"
              @click="
                result = {
                  plan: planSchema.parse(suggestion.plan),
                  preview: suggestion,
                  remove: false,
                }
              "
              >{{ t('review') }}</NcButton
            >
          </div>
        </section>
        <NcNoteCard v-if="rejected.length" type="warning"
          >{{ t(suggestions.length ? 'someRejected' : 'noSuggestions') }}
          <ul>
            <li v-for="reason in [...new Set(rejected)]" :key="reason">
              {{ explanation(reason) }}
            </li>
          </ul></NcNoteCard
        >
        <PlanReview
          v-if="result"
          :value="result"
          v-model:reviewed="reviewed"
          :ingredients="catalogue?.ingredients ?? []"
          :profiles="profiles?.members ?? []"
          :recipe-title="title(result.plan)"
          :meal-title="mealLabel(result.plan.meal)"
          :writable="writable"
          :busy="busy"
          @apply="apply"
        /></div
    ></NcAppSidebarTab>
  </NcAppSidebar>
  <section v-if="plans?.shopping_requirements.length && !draft">
    <h2>{{ t('shopping') }}</h2>
    <ul class="plain-list">
      <li v-for="line in plans.shopping_requirements" :key="line.ingredient_id">
        {{ names(line.ingredient_id) }}: {{ compactExact(line.grams) }} g
      </li>
    </ul>
  </section>
</template>
