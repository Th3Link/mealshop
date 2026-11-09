<script setup lang="ts">
import type { PlanReview } from './planning-api'
import type { Ingredient, Member } from './contracts'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import Nutrition from './Nutrition.vue'
import { exact, locale, t, explanation, allergenName } from './i18n'
const props = defineProps<{
  value: PlanReview
  ingredients: Ingredient[]
  profiles: Member[]
  recipeTitle: string
  mealTitle: string
  writable: boolean
  busy: boolean
}>()
const reviewed = defineModel<boolean>('reviewed', { required: true })
defineEmits<{ apply: [] }>()
function names(id: string) {
  const i = props.ingredients.find((i) => i.id === id)
  return i?.translations[locale.value]?.name ?? i?.name ?? id
}
</script>
<template>
  <section>
    <h2>{{ t('review') }}</h2>
    <p>
      {{ value.plan.date }} · {{ mealTitle }} · {{ recipeTitle }} · {{ value.plan.servings }}
      {{ t('servings') }}
    </p>
    <p>
      {{ t('attendees') }}:
      {{
        (value.plan.attendees ?? value.plan.calendar?.member_ids ?? [])
          .map((id) => profiles.find((m) => m.id === id)?.name ?? id)
          .join(', ')
      }}
    </p>
    <p v-if="value.plan.calendar">
      {{ t('calendarSource') }}: {{ value.plan.calendar.source }} · {{ value.plan.calendar.event_id
      }}<span v-if="value.plan.attendees !== null"> · {{ t('attendanceOverridden') }}</span>
    </p>
    <template v-if="value.preview.timing"
      ><h3>{{ t('timingEstimate') }}</h3>
      <dl>
        <dt>{{ t('referenceServings') }}</dt>
        <dd>{{ value.preview.timing.reference_servings }}</dd>
        <dt>{{ t('activeSeconds') }}</dt>
        <dd>{{ value.preview.timing.active_seconds }}</dd>
        <dt>{{ t('elapsedSeconds') }}</dt>
        <dd>{{ value.preview.timing.elapsed_seconds }}</dd>
        <dt>{{ t('advanceSeconds') }}</dt>
        <dd>{{ value.preview.timing.advance_preparation_seconds }}</dd>
      </dl>
      <p>{{ t('timingWarning') }}</p></template
    >
    <NcNoteCard v-if="value.preview.conflicts.length" type="error"
      ><strong>{{ t('conflicts') }}</strong>
      <ul>
        <li v-for="conflict in value.preview.conflicts" :key="conflict">
          {{ explanation(conflict) }}
        </li>
      </ul></NcNoteCard
    ><NcNoteCard v-else type="warning"
      >{{ t('reviewRequired') }} · {{ t('profileIntro') }}</NcNoteCard
    >
    <template v-if="value.preview.review"
      ><div v-for="note in value.preview.review.review_notes" :key="note.member_id">
        <h3>{{ note.name }}</h3>
        <p v-if="note.allergen_ids.length">
          {{ t('allergenGroups') }}: {{ note.allergen_ids.map(allergenName).join(', ') }}
        </p>
        <p v-if="note.allergies.length">{{ t('allergies') }}: {{ note.allergies.join(', ') }}</p>
        <p v-if="note.intolerances.length">
          {{ t('intolerances') }}: {{ note.intolerances.join(', ') }}
        </p>
      </div>
      <p v-if="value.preview.review.excluded_ingredients.length">
        {{ t('excluded') }}: {{ value.preview.review.excluded_ingredients.map(names).join(', ') }}
      </p>
      <p v-if="value.preview.review.disliked_ingredients.length">
        {{ t('disliked') }}: {{ value.preview.review.disliked_ingredients.map(names).join(', ') }}
      </p>
      <ul>
        <li v-for="warning in value.preview.review.warning_codes" :key="warning">
          {{ explanation(warning) }}
        </li>
      </ul>
      <p
        v-for="match in value.preview.review.allergen_matches"
        :key="`${match.member_id}-${match.ingredient_id}-${match.allergen_id}`"
      >
        {{ profiles.find((m) => m.id === match.member_id)?.name }} ·
        {{ names(match.ingredient_id) }} · {{ allergenName(match.allergen_id) }} ·
        <a :href="match.source_url" target="_blank" rel="noopener noreferrer">{{ t('source') }}</a>
      </p></template
    >
    <h3>{{ t('shopping') }}</h3>
    <ul class="plain-list">
      <li v-for="line in value.preview.shopping_delta" :key="line.ingredient_id">
        {{ names(line.ingredient_id) }}: {{ exact(line.grams_delta) }} g
      </li>
    </ul>
    <NcCheckboxRadioSwitch v-model="reviewed">{{ t('acknowledge') }}</NcCheckboxRadioSwitch
    ><NcButton
      v-if="writable"
      variant="primary"
      :disabled="busy || !reviewed || !value.preview.feasible"
      @click="$emit('apply')"
      >{{ value.remove ? t('remove') : t('apply') }}</NcButton
    >
    <Nutrition v-if="value.preview.nutrition" :value="value.preview.nutrition" />
  </section>
</template>
