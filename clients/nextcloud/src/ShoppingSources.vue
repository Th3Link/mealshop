<script setup lang="ts">
import { computed, ref } from 'vue'
import type { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcTextField from './LabeledField.vue'
import ShoppingOrigin from './ShoppingOrigin.vue'
import type { recipesSchema } from './contracts'
import type { plansSchema } from './planning-api'
import type { Shopping, ShoppingCommand } from './shopping-api'
import { servingCount, calendarDate, mealName, t } from './i18n'
const props = defineProps<{
  data: Shopping
  recipes: z.infer<typeof recipesSchema>
  plans: z.infer<typeof plansSchema>
  busy: boolean
  writable: boolean
  perform: (command: ShoppingCommand) => Promise<boolean>
}>()
const emit = defineEmits<{
  manual: [source: Shopping['sources'][number]]
  dirty: [value: boolean]
}>()
const showRecipe = ref(false),
  showPlan = ref(false),
  recipeId = ref(''),
  recipeRevision = ref(1),
  servings = ref(1),
  sourceId = ref<string>(),
  slots = ref<string[]>([]),
  changed = ref(false)
const options = computed(() => {
  const values = props.recipes.recipes.map((r) => ({
    id: r.id,
    revision: r.revision,
    title: r.title,
    servings: r.servings,
  }))
  const pinned = props.data.sources.find((s) => s.id === sourceId.value)?.origin
  if (
    pinned?.kind === 'recipe' &&
    !values.some((r) => r.id === pinned.recipe_id && r.revision === pinned.revision)
  )
    values.push({
      id: pinned.recipe_id,
      revision: pinned.revision,
      title: pinned.title,
      servings: pinned.servings,
    })
  return values
})
function mark() {
  changed.value = true
  emit('dirty', true)
}
function recipe(source?: Shopping['sources'][number]) {
  if (changed.value && !window.confirm(t('unsaved'))) return
  const origin = source?.origin
  sourceId.value = source?.id
  recipeId.value = origin?.kind === 'recipe' ? origin.recipe_id : (options.value[0]?.id ?? '')
  recipeRevision.value =
    origin?.kind === 'recipe' ? origin.revision : (options.value[0]?.revision ?? 1)
  servings.value = origin?.kind === 'recipe' ? origin.servings : (options.value[0]?.servings ?? 1)
  showRecipe.value = true
  showPlan.value = false
  changed.value = false
  emit('dirty', false)
}
function selected(event: Event) {
  if (event.target instanceof HTMLSelectElement) {
    const value = event.target.value
    const option = options.value.find((r) => `${r.id}@${r.revision}` === value)
    if (option) {
      recipeId.value = option.id
      recipeRevision.value = option.revision
      servings.value = option.servings
      mark()
    }
  }
}
function cancel() {
  if (!changed.value || window.confirm(t('unsaved'))) {
    showRecipe.value = false
    showPlan.value = false
    changed.value = false
    emit('dirty', false)
  }
}
function plan() {
  if (changed.value && !window.confirm(t('unsaved'))) return
  slots.value = props.plans.meals.map((p) => `${p.date}|${p.meal}`)
  showPlan.value = true
  showRecipe.value = false
  changed.value = false
  emit('dirty', false)
}
async function addRecipe() {
  if (
    await props.perform({
      operation: 'add_recipe',
      source_id: sourceId.value,
      recipe: {
        recipe_id: recipeId.value,
        revision: recipeRevision.value,
        servings: Number(servings.value),
      },
    })
  ) {
    showRecipe.value = false
    changed.value = false
    emit('dirty', false)
  }
}
async function sync() {
  if (
    await props.perform({
      operation: 'sync_plan',
      plan_revision: props.plans.revision,
      slots: props.plans.meals
        .filter((p) => slots.value.includes(`${p.date}|${p.meal}`))
        .map((p) => ({ date: p.date, meal: p.meal })),
    })
  ) {
    showPlan.value = false
    changed.value = false
    emit('dirty', false)
  }
}
async function remove(id: string) {
  if (window.confirm(t('shoppingRemoveHelp')))
    await props.perform({ operation: 'remove_source', id })
}
</script>
<template>
  <section>
    <div v-if="writable" class="actions">
      <NcButton :disabled="busy || !recipes.recipes.length" @click="recipe()">{{
        t('shoppingAddRecipe')
      }}</NcButton
      ><NcButton :disabled="busy" @click="plan">{{ t('shoppingFromPlan') }}</NcButton>
    </div>
    <form v-if="showRecipe" class="narrow" @submit.prevent="addRecipe" @input="mark">
      <h2>{{ t('shoppingAddRecipe') }}</h2>
      <fieldset class="editor-fields" :disabled="busy">
        <label class="field"
          >{{ t('recipes')
          }}<select :value="`${recipeId}@${recipeRevision}`" required @change="selected">
            <option
              v-for="r in options"
              :key="`${r.id}@${r.revision}`"
              :value="`${r.id}@${r.revision}`"
            >
              {{ r.title }} · {{ t('revision') }} {{ r.revision }}
            </option>
          </select></label
        ><NcTextField
          v-model="servings"
          type="number"
          :label="t('servings')"
          min="1"
          max="1000000"
          step="1"
          required
        />
        <p>{{ t('shoppingSourceHelp') }}</p>
        <div class="actions">
          <NcButton type="submit" variant="primary" :disabled="busy || !recipeId">{{
            t('save')
          }}</NcButton
          ><NcButton :disabled="busy" @click="cancel">{{ t('cancel') }}</NcButton>
        </div>
      </fieldset>
    </form>
    <form v-if="showPlan" class="narrow" @submit.prevent="sync">
      <h2>{{ t('shoppingFromPlan') }}</h2>
      <p>{{ t('shoppingPlanHelp') }}</p>
      <fieldset :disabled="busy">
        <NcCheckboxRadioSwitch
          v-for="p in plans.meals"
          :key="`${p.date}|${p.meal}`"
          v-model="slots"
          :value="`${p.date}|${p.meal}`"
          @update:model-value="mark"
          >{{ calendarDate(p.date) }} · {{ mealName(p.meal) }} ·
          {{
            plans.recipe_revisions.find(
              (r) => r.id === p.recipe.id && r.revision === p.recipe.revision,
            )?.title ?? p.recipe.id
          }}
          · {{ servingCount(p.servings) }}</NcCheckboxRadioSwitch
        >
      </fieldset>
      <div class="actions">
        <NcButton variant="primary" type="submit" :disabled="busy">{{
          t('shoppingSyncPlan')
        }}</NcButton
        ><NcButton :disabled="busy" @click="cancel">{{ t('cancel') }}</NcButton>
      </div>
    </form>
    <template v-if="data.sources.some((source) => source.active)"
      ><h2>{{ t('shoppingSources') }}</h2>
      <ul class="plain-list">
        <li v-for="source in data.sources.filter((source) => source.active)" :key="source.id">
          <ShoppingOrigin :origin="source.origin" />
          <div v-if="writable" class="actions">
            <NcButton
              v-if="['manual', 'recipe'].includes(source.origin.kind)"
              :disabled="busy"
              @click="source.origin.kind === 'manual' ? emit('manual', source) : recipe(source)"
              >{{ t(source.origin.kind === 'manual' ? 'shoppingEditItem' : 'edit') }}</NcButton
            ><NcButton :disabled="busy" @click="remove(source.id)">{{ t('remove') }}</NcButton>
          </div>
        </li>
      </ul></template
    >
  </section>
</template>
