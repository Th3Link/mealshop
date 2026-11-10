<script setup lang="ts">
import { computed, onMounted, ref, toRaw, watch, nextTick } from 'vue'
import { useRoute } from 'vue-router'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcSelect from '@nextcloud/vue/components/NcSelect'
import NcTextField from './LabeledField.vue'
import CookingStageEditor from './CookingStageEditor.vue'
import CookingLocalTime from './CookingLocalTime.vue'
import CookingReview from './CookingReview.vue'
import { api, domain } from './api'
import { useTask } from './state'
import { recipesSchema } from './contracts'
import { templateSchema } from './cooking-api'
import type { CookingDraft, CookingPreview, Proposal } from './cooking-api'
import { localInput, zone } from './cooking-format'
import { t } from './i18n'
import type { z } from 'zod'
const props = defineProps<{
  home: string
  disabled: boolean
  preview: (proposal: Proposal, revision: number) => Promise<CookingPreview | undefined>
  apply: (preview: CookingPreview) => Promise<boolean>
}>()
const emit = defineEmits<{ dirty: [value: boolean] }>()
const route = useRoute(),
  recipes = ref<z.infer<typeof recipesSchema>['recipes']>([]),
  selected = ref(''),
  servings = ref(1),
  draft = ref<CookingDraft>(),
  revision = ref(0),
  review = ref<CookingPreview>(),
  reviewed = ref(false)
const { busy, error, run, errorMessage } = useTask()
const options = computed(() => recipes.value.map((r) => ({ ...r, label: r.title })))
const chosen = computed(() => options.value.find((r) => r.id === selected.value) ?? null)
const stageOptions = computed(
  () => draft.value?.stages.map((s) => ({ id: s.id, label: s.name })) ?? [],
)
const selection = computed({
  get: () => chosen.value,
  set: (value: unknown) => {
    const result = recipesSchema.shape.recipes.element.safeParse(value)
    if (result.success) {
      selected.value = result.data.id
      servings.value = result.data.servings
    }
  },
})
const target = ref({
  local: localInput(Math.floor(Date.now() / 1000) + 86400, zone()),
  timezone: zone(),
  fold: 'reject' as const,
})
watch(draft, () => emit('dirty', !!draft.value), { deep: true })
async function template() {
  await run(async () => {
    if (!chosen.value) return
    const data = await api(
      domain(
        props.home,
        `/api/household/cooking/template?${new URLSearchParams({ recipe_id: chosen.value.id, revision: String(route.query.recipe === chosen.value.id && /^\d+$/.test(String(route.query.revision)) ? route.query.revision : chosen.value.revision), servings: String(servings.value), timezone: zone() })}`,
      ),
      templateSchema,
    )
    draft.value = data.draft
    revision.value = data.revision
  })
}
async function preview() {
  if (!draft.value) return
  const value = structuredClone(toRaw(draft.value))
  value.servings = Number(value.servings)
  value.target = value.kind === 'cooking' ? null : structuredClone(toRaw(target.value))
  value.timezone = value.target?.timezone ?? value.timezone
  for (const step of value.stages) {
    step.repeat = Number(step.repeat)
    if (step.repeat === 1) step.wait_between_seconds = 0
  }
  review.value = await props.preview({ kind: 'create', draft: value }, revision.value)
  reviewed.value = false
  if (review.value) {
    await nextTick()
    document.getElementById('mealshop-cooking-review')?.focus()
  }
}
async function apply() {
  if (review.value && (await props.apply(review.value))) emit('dirty', false)
}
function add() {
  draft.value?.stages.push({
    id: `custom-${crypto.randomUUID()}`,
    name: '',
    instruction: '',
    origin: { kind: 'custom', note: '' },
    dependencies: [],
    active_seconds: 0,
    passive_seconds: 0,
    passive_min_seconds: 0,
    passive_max_seconds: 0,
    equipment: [],
    lead_seconds: 300,
    repeat: 1,
    wait_between_seconds: 0,
    uncertainty: '',
  })
}
function remove(id: string) {
  if (!draft.value) return
  draft.value.stages = draft.value.stages.filter((s) => s.id !== id)
  for (const step of draft.value.stages)
    step.dependencies = step.dependencies.filter((d) => d !== id)
}
onMounted(() =>
  run(async () => {
    recipes.value = (await api(domain(props.home, '/api/recipes'), recipesSchema)).recipes
    const match = recipes.value.find((r) => r.id === route.query.recipe)
    if (match) {
      selected.value = match.id
      servings.value = Number(route.query.servings) || match.servings
      await template()
    }
  }),
)
</script>
<template>
  <section>
    <p>{{ t('cookCreateHelp') }}</p>
    <p v-if="error" role="alert">{{ errorMessage(error) }}</p>
    <CookingReview
      v-if="review"
      v-model:reviewed="reviewed"
      :preview="review"
      :busy="disabled"
      :writable="true"
      @back="review = undefined"
      @apply="apply"
    />
    <form v-else-if="!draft" class="editor-fields" @submit.prevent="template">
      <div class="labeled-field">
        <label for="cooking-recipe">{{ t('recipes') }}</label
        ><NcSelect
          v-model="selection"
          input-id="cooking-recipe"
          :input-label="t('recipes')"
          label-outside
          :options="options"
          label="label"
          :clearable="false"
          ><template #option="option"
            ><span class="mealshop-selection-name">{{ option.label }}</span></template
          ><template #selected-option="option"
            ><span class="mealshop-selection-name">{{ option.label }}</span></template
          ><template #no-options>{{ t('cookNoRecipes') }}</template></NcSelect
        >
      </div>
      <NcTextField
        v-model="servings"
        type="number"
        :label="t('servings')"
        min="1"
        max="1000000"
        step="1"
        required
      /><NcButton type="submit" variant="primary" :disabled="disabled || busy || !chosen">{{
        t('cookLoadSteps')
      }}</NcButton>
    </form>
    <form v-else class="editor-fields" @submit.prevent="preview">
      <p class="muted">{{ draft.recipe_id }} · {{ t('revision') }} {{ draft.recipe_revision }}</p>
      <NcTextField v-model="draft.name" :label="t('name')" required maxlength="200" /><NcTextField
        v-model="draft.servings"
        type="number"
        :label="t('servings')"
        min="1"
        max="1000000"
        step="1"
        required
      /><label class="field"
        >{{ t('cookKind')
        }}<select v-model="draft.kind" :aria-label="t('cookKind')">
          <option value="cooking">{{ t('cookKindCooking') }}</option>
          <option value="bread">{{ t('cookKindBread') }}</option>
          <option value="rolls">{{ t('cookKindRolls') }}</option>
          <option value="pizza">{{ t('cookKindPizza') }}</option>
          <option value="baguette">{{ t('cookKindBaguette') }}</option>
        </select></label
      ><CookingLocalTime
        v-if="draft.kind !== 'cooking'"
        v-model="target"
        :label="t('cookTarget')"
        zone-editable
      />
      <p>{{ t('cookStagesHelp') }}</p>
      <CookingStageEditor
        v-for="(stage, index) in draft.stages"
        :key="stage.id"
        :model-value="stage"
        :index="index"
        :options="stageOptions"
        @update:model-value="draft.stages[index] = $event"
        @remove="remove(stage.id)"
      />
      <div class="actions">
        <NcButton :disabled="disabled || draft.stages.length >= 100" @click="add">{{
          t('cookAddStage')
        }}</NcButton
        ><NcButton type="submit" variant="primary" :disabled="disabled || !draft.stages.length">{{
          t('cookPreview')
        }}</NcButton>
      </div>
    </form>
  </section>
</template>
