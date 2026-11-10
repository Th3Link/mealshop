<script setup lang="ts">
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import type { CookingPreview } from './cooking-api'
import { localDate, duration } from './cooking-format'
import { t, explanation } from './i18n'
defineProps<{ preview: CookingPreview; busy: boolean; writable: boolean }>()
const reviewed = defineModel<boolean>('reviewed', { required: true })
defineEmits<{ apply: []; back: [] }>()
</script>
<template>
  <section class="cooking-review">
    <h2 id="mealshop-cooking-review" tabindex="-1">{{ t('cookReview') }}</h2>
    <p>{{ preview.run.name }} · {{ preview.run.timezone }}</p>
    <p>
      {{ t('cookPlannedFinish') }}:
      <strong>{{ localDate(preview.proposal.finish, preview.run.timezone) }}</strong>
    </p>
    <p>{{ t('cookReviewExpires') }} {{ localDate(preview.expires_at, preview.run.timezone) }}</p>
    <NcNoteCard v-for="(warning, index) in preview.proposal.warnings" :key="index" type="warning"
      >{{ explanation(warning.code)
      }}<span v-if="warning.steps.length">
        ·
        {{
          warning.steps
            .map((id) => preview.run.steps.find((s) => s.stage.id === id)?.stage.name ?? id)
            .join(' / ')
        }}</span
      ></NcNoteCard
    >
    <ol class="cooking-timeline">
      <li v-for="step in preview.run.steps" :key="step.stage.id">
        <h3>
          {{ step.stage.name
          }}<template v-if="step.stage.repeat > 1">
            · {{ step.cycle }}/{{ step.stage.repeat }}</template
          >
        </h3>
        <p>{{ step.stage.instruction }}</p>
        <dl>
          <dt>{{ t('cookPlannedStart') }}</dt>
          <dd>{{ preview.run.local_schedule.find((s) => s.id === step.stage.id)?.start }}</dd>
          <dt>{{ t('cookPlannedFinish') }}</dt>
          <dd>{{ preview.run.local_schedule.find((s) => s.id === step.stage.id)?.finish }}</dd>
          <template v-if="preview.previous"
            ><dt>{{ t('cookPreviousStart') }}</dt>
            <dd>
              {{ preview.previous.local_schedule.find((s) => s.id === step.stage.id)?.start }}
            </dd></template
          >
          <dt>{{ t('cookStartWindow') }}</dt>
          <dd>
            {{ preview.run.local_schedule.find((s) => s.id === step.stage.id)?.earliest }} –
            {{ preview.run.local_schedule.find((s) => s.id === step.stage.id)?.latest }}
          </dd>
          <dt>{{ t('cookActiveDuration') }} / {{ t('cookPassiveDuration') }}</dt>
          <dd>
            {{ duration(step.stage.active_seconds) }} / {{ duration(step.stage.passive_seconds) }}
          </dd>
          <template v-if="step.stage.dependencies.length"
            ><dt>{{ t('cookDependencies') }}</dt>
            <dd>
              {{
                step.stage.dependencies
                  .map((id) => preview.run.steps.find((s) => s.stage.id === id)?.stage.name ?? id)
                  .join(', ')
              }}
            </dd></template
          ><template v-if="step.stage.equipment.length"
            ><dt>{{ t('cookEquipment') }}</dt>
            <dd>{{ step.stage.equipment.join(', ') }}</dd></template
          >
          <dt>{{ t('cookUncertainty') }}</dt>
          <dd>{{ step.stage.uncertainty || t('unknown') }}</dd>
        </dl>
      </li>
    </ol>
    <NcCheckboxRadioSwitch v-model="reviewed" :disabled="busy || !writable">{{
      t('cookReviewed')
    }}</NcCheckboxRadioSwitch>
    <div class="actions">
      <NcButton
        variant="primary"
        :disabled="busy || !reviewed || !writable"
        @click="$emit('apply')"
        >{{ t('cookApply') }}</NcButton
      ><NcButton :disabled="busy" @click="$emit('back')">{{ t('cookEditDraft') }}</NcButton>
    </div>
  </section>
</template>
