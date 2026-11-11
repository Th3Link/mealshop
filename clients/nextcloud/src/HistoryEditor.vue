<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { z } from 'zod'
import NcSelect from '@nextcloud/vue/components/NcSelect'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import LabeledField from './LabeledField.vue'
import HistoryRating from './HistoryRating.vue'
import { recipesSchema, profilesSchema } from './contracts'
import type { HistoryDraft } from './feedback-api'
import { t } from './i18n'
const model = defineModel<HistoryDraft>({ required: true })
const props = defineProps<{
  recipes: z.infer<typeof recipesSchema>['recipes']
  people: z.infer<typeof profilesSchema>['members']
  busy: boolean
  sessionTitle?: string
}>()
const emit = defineEmits<{ changed: [] }>()
const selection = computed(() =>
  model.value.source.kind === 'manual'
    ? (props.recipes.find(
        (r) => model.value.source.kind === 'manual' && r.id === model.value.source.recipe_id,
      ) ?? null)
    : null,
)
function choose(value: unknown) {
  const parsed = recipesSchema.shape.recipes.element.nullable().safeParse(value)
  if (!parsed.success) return
  model.value.source = {
    kind: 'manual',
    recipe_id: parsed.data?.id ?? '',
    revision: parsed.data?.revision ?? 1,
    servings: parsed.data?.servings ?? 1,
  }
  emit('changed')
}
const removed = computed(() =>
  model.value.input.attendees.filter((id) => !props.people.some((p) => p.id === id)),
)
function unselect(id: string) {
  model.value.input.attendees = model.value.input.attendees.filter((p) => p !== id)
  delete model.value.input.ratings[id]
  emit('changed')
}
function rate(id: string, value: number | null) {
  if (value === null) delete model.value.input.ratings[id]
  else model.value.input.ratings[id] = value
  emit('changed')
}
const effort = ref(''),
  unit = ref(60)
function readEffort() {
  const n = model.value.input.active_seconds
  unit.value = n !== null && n % 60 !== 0 ? 1 : 60
  effort.value = n === null ? '' : String(n / unit.value)
}
readEffort()
function updateEffort(value: string | number) {
  effort.value = String(value)
  const number = Number(effort.value) * unit.value
  model.value.input.active_seconds =
    effort.value === ''
      ? null
      : /^\d+$/.test(effort.value) && Number.isSafeInteger(number) && number <= 604800
        ? number
        : -1
  emit('changed')
}
watch(
  () => model.value.input.active_seconds,
  (n) => {
    if (
      n !== -1 &&
      ((n === null && effort.value !== '') ||
        (n !== null && Number(effort.value) * unit.value !== n))
    )
      readEffort()
  },
)
watch(
  () => model.value.input.attendees,
  (ids) => {
    for (const id of Object.keys(model.value.input.ratings))
      if (!ids.includes(id)) delete model.value.input.ratings[id]
  },
  { deep: true },
)
</script>
<template>
  <fieldset class="editor-fields" :disabled="busy">
    <NcNoteCard type="info">{{ t('historyVisibility') }}</NcNoteCard>
    <div v-if="model.source.kind === 'manual'" class="history-source editor-fields">
      <NcSelect
        :model-value="selection"
        :options="recipes"
        label="title"
        :input-label="t('historyRecipe')"
        label-outside
        :disabled="busy"
        :clearable="true"
        @update:model-value="choose"
        ><template #option="recipe"
          ><span>{{ recipe.title }}</span></template
        ><template #no-options>{{ t('noResults') }}</template></NcSelect
      >
      <div class="form-row">
        <LabeledField
          v-model="model.source.revision"
          type="number"
          :label="t('historySourceRecipe')"
          min="1"
          :max="selection?.revision"
          step="1"
          required
          @update:model-value="emit('changed')"
        />
        <LabeledField
          v-model="model.source.servings"
          type="number"
          :label="t('servings')"
          min="1"
          max="1000000"
          step="1"
          required
          @update:model-value="emit('changed')"
        />
      </div>
    </div>
    <div v-else>
      <strong>{{ sessionTitle }}</strong>
      <p class="muted">{{ t('historyPinned') }}</p>
    </div>
    <div class="form-row">
      <LabeledField
        v-model="model.input.date"
        type="date"
        :label="t('historyDate')"
        required
        @update:model-value="emit('changed')"
      /><LabeledField
        v-model="model.input.timezone"
        :label="t('historyZone')"
        maxlength="80"
        required
        @update:model-value="emit('changed')"
      />
    </div>
    <fieldset class="history-attendees">
      <legend>{{ t('historyPeople') }}</legend>
      <NcCheckboxRadioSwitch
        v-for="person in people"
        :key="person.id"
        v-model="model.input.attendees"
        :value="person.id"
        @update:model-value="emit('changed')"
        >{{ person.name }}</NcCheckboxRadioSwitch
      >
    </fieldset>
    <NcNoteCard v-if="!people.length" type="info">{{ t('historyPeopleRequired') }}</NcNoteCard>
    <NcNoteCard v-if="removed.length" type="warning"
      >{{ t('historyRemovedHelp') }}
      <div v-for="id in removed" :key="id" class="actions">
        <span>{{ t('historyRemovedPerson') }}</span
        ><NcButton type="button" @click="unselect(id)">{{ t('remove') }}</NcButton>
      </div></NcNoteCard
    >
    <div class="editor-fields">
      <div class="cooking-duration">
        <LabeledField
          :model-value="effort"
          :label="t('historyEffort')"
          inputmode="numeric"
          :aria-invalid="model.input.active_seconds === -1"
          :helper-text="
            model.input.active_seconds === -1 ? t('cookDurationError') : t('historyEffortHelp')
          "
          @update:model-value="updateEffort"
        /><label class="field"
          ><span class="visually-hidden">{{ t('historyEffort') }} · {{ t('unit') }}</span
          ><select
            v-model.number="unit"
            :aria-label="`${t('historyEffort')} · ${t('unit')}`"
            @change="updateEffort(effort)"
          >
            <option :value="1">{{ t('cookSeconds') }}</option>
            <option :value="60">{{ t('cookMinutes') }}</option>
          </select></label
        >
      </div>
    </div>
    <HistoryRating
      v-model="model.input.household_rating"
      :label="t('historyHouseholdRating')"
      @update:model-value="emit('changed')"
    />
    <p class="muted">{{ t('historyRatingHelp') }}</p>
    <div class="history-ratings">
      <HistoryRating
        v-for="person in people.filter((p) => model.input.attendees.includes(p.id))"
        :key="person.id"
        :model-value="model.input.ratings[person.id] ?? null"
        :label="`${t('historyPersonalRating')} · ${person.name}`"
        @update:model-value="rate(person.id, $event)"
      />
    </div>
    <NcCheckboxRadioSwitch
      v-model="model.confirmed_completed"
      @update:model-value="emit('changed')"
      >{{ t('historyConfirm') }}</NcCheckboxRadioSwitch
    >
  </fieldset>
</template>
