<script setup lang="ts">
import { computed, onMounted, ref, toRaw, watch } from 'vue'
import { onBeforeRouteLeave, useRoute } from 'vue-router'
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextField from '@nextcloud/vue/components/NcTextField'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { api, domain } from './api'
import { catalogueSchema, emptySchema, profilesSchema, recipesSchema } from './contracts'
import { plansSchema, previewSchema } from './planning-api'
import type { Plan, PlanReview as ReviewValue } from './planning-api'
import { homes, useTask, loadCatalogue } from './state'
import { exact, locale, t, explanation } from './i18n'
import PlanReview from './PlanReview.vue'
const route = useRoute(),
  home = computed(() => String(route.params.home))
const writable = computed(() => homes.value.find((h) => h.id === home.value)?.role !== 'reader')
const plans = ref<z.infer<typeof plansSchema>>(),
  profiles = ref<z.infer<typeof profilesSchema>>(),
  recipes = ref<z.infer<typeof recipesSchema>>(),
  catalogue = ref<z.infer<typeof catalogueSchema>>()
const draft = ref<Plan>(),
  reviewed = ref(false)
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
  meal === 'breakfast' || meal === 'lunch' || meal === 'dinner' || meal === 'snack' ? t(meal) : meal
async function load() {
  await run(async () => {
    const data = await Promise.all([
      api(path(''), plansSchema),
      api(domain(home.value, '/api/household/profiles'), profilesSchema),
      api(domain(home.value, '/api/recipes'), recipesSchema),
      loadCatalogue(),
    ])
    plans.value = data[0]
    profiles.value = data[1]
    recipes.value = data[2]
    catalogue.value = data[3]
  })
}
function start(plan?: Plan) {
  draft.value = plan
    ? structuredClone(toRaw(plan))
    : {
        date: new Intl.DateTimeFormat('en-CA', {
          year: 'numeric',
          month: '2-digit',
          day: '2-digit',
        }).format(new Date()),
        meal: 'dinner',
        recipe: recipes.value?.recipes[0]
          ? { id: recipes.value.recipes[0].id, revision: recipes.value.recipes[0].revision }
          : { id: '', revision: 1 },
        attendees: profiles.value?.members.map((m) => m.id) ?? [],
        calendar: null,
        servings: profiles.value?.members.length || 1,
        active_seconds: 1800,
        elapsed_seconds: 3600,
        advance_seconds: 0,
      }
  result.value = undefined
  suggestions.value = []
  rejected.value = []
}
function cancelPlan() {
  draft.value = undefined
  result.value = undefined
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
  p.active_seconds = Number(p.active_seconds)
  p.elapsed_seconds = Number(p.elapsed_seconds)
  p.advance_seconds = Number(p.advance_seconds)
  if (p.calendar) p.calendar.date = p.date
  return p
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
    draft.value = undefined
    result.value = undefined
    await load()
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
onBeforeRouteLeave(() => !draft.value || window.confirm(t('unsaved')))
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
      :disabled="busy || !recipes?.recipes.length || !profiles?.members.length"
      @click="start()"
      >{{ t('newMeal') }}</NcButton
    >
  </header>
  <NcNoteCard v-if="error" type="error" role="alert"
    >{{ errorMessage(error) }}<NcButton @click="load">{{ t('retry') }}</NcButton></NcNoteCard
  ><NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
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
  <ul v-if="plans && !draft" class="record-list">
    <li v-for="plan in plans.meals" :key="`${plan.date}-${plan.meal}`">
      <div class="inline-row">
        <strong>{{ plan.date }} · {{ mealLabel(plan.meal) }}</strong
        ><span
          >{{ title(plan) }} · {{ t('revision') }} {{ plan.recipe.revision }} · {{ plan.servings }}
          {{ t('servings') }}</span
        ><span
          v-if="
            plans.review_required_slots.some((s) => s.date === plan.date && s.meal === plan.meal)
          "
          >{{ t('reviewRequired') }}</span
        ><NcButton @click="start(plan)">{{ t('details') }}</NcButton
        ><NcButton v-if="writable" :disabled="busy" @click="preview(true, plan)">{{
          t('remove')
        }}</NcButton>
      </div>
    </li>
  </ul>
  <section v-if="draft" class="narrow">
    <h2>{{ t('newMeal') }}</h2>
    <form @submit.prevent="preview(false)">
      <div class="form-row">
        <label class="field"
          >{{ t('date') }}<input v-model="draft.date" type="date" required /></label
        ><label class="field"
          >{{ t('meal')
          }}<select v-model="draft.meal">
            <option
              v-for="meal in ['breakfast', 'lunch', 'dinner', 'snack']"
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
      <NcTextField
        v-model="draft.active_seconds"
        type="number"
        :label="t('activeBudget')"
        min="0"
        max="86400"
        step="1"
        required
      /><NcTextField
        v-model="draft.elapsed_seconds"
        type="number"
        :label="t('elapsedBudget')"
        min="0"
        max="86400"
        step="1"
        required
      /><NcTextField
        v-model="draft.advance_seconds"
        type="number"
        :label="t('advanceBudget')"
        min="0"
        max="2592000"
        step="1"
        required
      />
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
            @update:model-value="draft.attendees = $event ? [...draft.calendar.member_ids] : null"
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
      <div class="actions">
        <NcButton type="submit" variant="primary" :disabled="busy">{{ t('review') }}</NcButton
        ><NcButton :disabled="busy" @click="suggest">{{ t('suggestions') }}</NcButton
        ><NcButton @click="cancelPlan">{{ t('cancel') }}</NcButton>
      </div>
    </form>
  </section>
  <section v-if="suggestions.length">
    <h2>{{ t('suggestions') }}</h2>
    <div
      v-for="suggestion in suggestions"
      :key="`${suggestion.plan.recipe.id}@${suggestion.plan.recipe.revision}`"
      class="inline-row"
    >
      <span>{{ title(suggestion.plan) }}</span
      ><NcButton
        @click="
          result = { plan: planSchema.parse(suggestion.plan), preview: suggestion, remove: false }
        "
        >{{ t('review') }}</NcButton
      >
    </div>
  </section>
  <NcNoteCard v-if="rejected.length" type="warning"
    >{{ t(suggestions.length ? 'someRejected' : 'noSuggestions') }}
    <ul>
      <li v-for="reason in [...new Set(rejected)]" :key="reason">{{ explanation(reason) }}</li>
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
  />
  <section v-if="plans?.shopping_requirements.length && !draft">
    <h2>{{ t('shopping') }}</h2>
    <ul class="plain-list">
      <li v-for="line in plans.shopping_requirements" :key="line.ingredient_id">
        {{ names(line.ingredient_id) }}: {{ exact(line.grams) }} g
      </li>
    </ul>
  </section>
</template>
