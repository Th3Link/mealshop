<script setup lang="ts">
import { watch, computed, ref, toRaw } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import LabeledField from './LabeledField.vue'
import CookingLocalTime from './CookingLocalTime.vue'
import RecipeReferences from './RecipeReferences.vue'
import type { BrewState, BrewBatchDraft, BrewCommand } from './brewing-api'
import { zone, localInput, localDate } from './cooking-format'
import { t, compactExact } from './i18n'
const props = defineProps<{ data: BrewState; disabled: boolean; recipeId?: string; base: string }>()
const emit = defineEmits<{ review: [value: BrewCommand]; dirty: [value: boolean] }>()
const recipes = computed(() => props.data.recipes.filter((r) => !r.archived))
const draft = ref<BrewBatchDraft>({
  name: '',
  recipe_id: props.recipeId ?? '',
  recipe_revision: 1,
  brew_day: {
    local: localInput(props.data.server_now + 86400, zone()),
    timezone: zone(),
    fold: 'reject',
  },
  yeast_preparation: { choice: 'skip' },
  equipment: {},
})
const selected = computed(
  () => recipes.value.find((r) => r.record.id === draft.value.recipe_id)?.record,
)
const equipment = computed(() =>
  props.data.equipment
    .filter((e) => !e.archived)
    .map((e) => ({
      id: e.id,
      label: `${e.details.name}${e.details.capacity ? ` · ${compactExact(e.details.capacity.amount)} ${e.details.capacity.unit}` : ''}`,
    })),
)
function reservations(phaseId: string) {
  const selected = draft.value.equipment[phaseId] ?? []
  return props.data.batches.flatMap((b) =>
    b.run.status === 'active'
      ? b.run.steps.flatMap((s) => {
          if (
            s.timer.state === 'completed' ||
            s.timer.state === 'cancelled' ||
            !s.stage.equipment.some((id) => selected.includes(id))
          )
            return []
          const timing = b.run.schedule.find((t) => t.id === s.stage.id)
          return [
            {
              id: `${b.run.id}:${s.stage.id}`,
              runId: b.run.id,
              name: b.run.name,
              phase: s.stage.name,
              held: s.timer.state === 'running' || s.timer.state === 'paused',
              at: timing?.start,
              timezone: b.run.timezone,
            },
          ]
        })
      : [],
  )
}
function choose() {
  const r = selected.value
  if (!r) return
  draft.value.name = r.recipe.title
  draft.value.recipe_revision = r.revision
  draft.value.yeast_preparation = { choice: r.recipe.yeast_preparation ? 'recipe' : 'skip' }
  draft.value.equipment = Object.fromEntries(r.recipe.phases.map((p) => [p.stage.id, []]))
  emit('dirty', true)
}
if (selected.value) choose()
function prep(event: Event) {
  if (!(event.target instanceof HTMLSelectElement)) return
  const choice = event.target.value
  draft.value.yeast_preparation =
    choice === 'custom'
      ? { choice, at: { ...draft.value.brew_day }, instruction: '' }
      : choice === 'recipe'
        ? { choice }
        : { choice: 'skip' }
}
function review() {
  emit('review', { operation: 'create_batch', draft: structuredClone(toRaw(draft.value)) })
}
watch(draft, () => emit('dirty', true), { deep: true })
</script>
<template>
  <form
    class="brew-editor"
    @submit.prevent="review"
    @input="emit('dirty', true)"
    @change="emit('dirty', true)"
  >
    <fieldset :disabled="disabled">
      <legend class="visually-hidden">{{ t('brewNewBatch') }}</legend>
      <NcNoteCard v-if="!recipes.length" type="info">{{ t('brewFirstRecipe') }}</NcNoteCard
      ><label class="field"
        >{{ t('brewRecipes')
        }}<select v-model="draft.recipe_id" required @change="choose">
          <option value="" disabled>{{ t('brewRecipes') }}</option>
          <option v-for="r in recipes" :key="r.record.id" :value="r.record.id">
            {{ r.record.recipe.title }} · {{ r.record.revision }}
          </option>
        </select></label
      ><template v-if="selected"
        ><LabeledField v-model="draft.name" :label="t('brewBatchName')" required maxlength="200" />
        <p class="muted">
          {{ t('brewPinned') }}: {{ selected.revision }} ·
          {{ compactExact(selected.recipe.volume.amount) }} {{ selected.recipe.volume.unit }}
        </p>
        <CookingLocalTime zone-editable v-model="draft.brew_day" :label="t('brewDay')" />
        <section>
          <h2>{{ t('brewYeast') }}</h2>
          <label class="field"
            ><span class="visually-hidden">{{ t('brewYeast') }}</span
            ><select
              :value="draft.yeast_preparation.choice"
              :aria-label="t('brewYeast')"
              @change="prep"
            >
              <option value="skip">{{ t('brewSkipPreparation') }}</option>
              <option v-if="selected.recipe.yeast_preparation" value="recipe">
                {{ t('brewRecipePreparation') }}
              </option>
              <option value="custom">{{ t('brewCustomPreparation') }}</option>
            </select></label
          >
          <p
            v-if="draft.yeast_preparation.choice === 'recipe' && selected.recipe.yeast_preparation"
          >
            {{ selected.recipe.yeast_preparation.instruction }} ·
            {{ selected.recipe.yeast_preparation.days_before }} {{ t('brewDaysBefore') }} ·
            {{ selected.recipe.yeast_preparation.local_time }}
          </p>
          <template v-if="draft.yeast_preparation.choice === 'custom'"
            ><CookingLocalTime
              zone-editable
              v-model="draft.yeast_preparation.at"
              :label="t('brewYeast')" /><LabeledField
              v-model="draft.yeast_preparation.instruction"
              :label="t('instruction')"
              required
              maxlength="4000"
          /></template>
        </section>
        <section>
          <h2>{{ t('brewEquipment') }}</h2>
          <p class="muted">{{ t('brewEquipmentHelp') }}</p>
          <NcNoteCard v-if="!equipment.length" type="info">{{
            t('brewEmptyEquipment')
          }}</NcNoteCard>
          <div
            v-for="phase in selected.recipe.phases"
            :key="phase.stage.id"
            class="brew-assignment"
          >
            <RecipeReferences
              :model-value="draft.equipment[phase.stage.id] ?? []"
              :label="phase.stage.name"
              :options="equipment"
              :disabled="disabled"
              @update:model-value="
                (value) => {
                  draft.equipment[phase.stage.id] = value
                  emit('dirty', true)
                }
              "
            />
            <ul v-if="reservations(phase.stage.id).length" class="brew-resource-uses">
              <li v-for="use in reservations(phase.stage.id)" :key="use.id">
                {{ t(use.held ? 'brewInUse' : 'brewReservation') }}:
                <RouterLink :to="`${base}/batches/${use.runId}`">{{ use.name }}</RouterLink> ·
                {{ use.phase }}
                <template v-if="use.at">
                  · {{ localDate(use.at, use.timezone) }} · {{ use.timezone }}</template
                >
              </li>
            </ul>
            <span v-if="phase.volume" class="muted"
              >{{ t('brewPhaseVolume') }}: {{ compactExact(phase.volume.amount) }}
              {{ phase.volume.unit }}</span
            ><span v-else class="muted">{{ t('brewPhaseVolume') }}: {{ t('unknown') }}</span>
          </div>
        </section>
        <NcButton type="submit" variant="primary">{{ t('brewReview') }}</NcButton></template
      >
    </fieldset>
  </form>
</template>
