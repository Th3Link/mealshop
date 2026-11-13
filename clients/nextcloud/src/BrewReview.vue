<script setup lang="ts">
import { computed } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import BrewRecipeSummary from './BrewRecipeSummary.vue'
import type { BrewCommand, BrewPreview } from './brewing-api'
import { localDate } from './cooking-format'
import { readingUnits } from './brewing-format'
import { t, exact, compactExact } from './i18n'
const props = defineProps<{ command: BrewCommand; preview: BrewPreview; disabled: boolean }>()
const reviewed = defineModel<boolean>('reviewed', { required: true })
defineEmits<{ apply: []; back: [] }>()
const affected = computed(() => {
  const command = props.command
  if (command.operation === 'create_batch')
    return props.preview.after.batches.find(
      (b) => !props.preview.before.batches.some((old) => old.run.id === b.run.id),
    )
  if ('run_id' in command)
    return props.preview.after.batches.find((b) => b.run.id === command.run_id)
  return undefined
})
function equipmentName(id: string) {
  return affected.value?.batch.equipment.find((e) => e.id === id)?.details.name ?? id
}
</script>
<template>
  <section class="brew-review">
    <h2 id="brew-review-title" tabindex="-1">{{ t('brewReview') }}</h2>
    <p>{{ t('brewReviewHelp') }}</p>
    <template v-if="command.operation === 'save_recipe'"
      ><h3>{{ command.recipe.title }}</h3>
      <BrewRecipeSummary :recipe="command.recipe" /></template
    ><template v-else-if="command.operation === 'save_equipment'"
      ><h3>{{ command.equipment.name }}</h3>
      <p>{{ command.equipment.model }}</p>
      <p>
        {{ t('brewCapacity') }}:
        {{
          command.equipment.capacity
            ? `${compactExact(command.equipment.capacity.amount)} ${command.equipment.capacity.unit}`
            : t('unknown')
        }}
      </p>
      <p>{{ command.equipment.notes }}</p></template
    ><template v-else-if="command.operation === 'add_reading'"
      ><h3>
        {{ exact(command.reading.measurement.value) }}
        {{ t(readingUnits[command.reading.measurement.unit]) }}
      </h3>
      <p>{{ command.reading.measured_at.local }} · {{ command.reading.measured_at.timezone }}</p>
      <p>
        {{ t(command.reading.source.kind === 'manual' ? 'brewManual' : 'brewInstrumentReading')
        }}<template v-if="command.reading.source.kind === 'instrument'">
          · {{ command.reading.source.name }} · {{ command.reading.source.device_id }}</template
        >
      </p>
      <p>
        {{
          command.reading.calibration.kind === 'declared'
            ? command.reading.calibration.details
            : t('brewCalibrationUnknown')
        }}
      </p>
      <template v-if="command.reading.calibration.kind === 'declared'">
        <p v-if="command.reading.calibration.calibrated_at">
          {{ t('brewCalibrationAt') }}: {{ command.reading.calibration.calibrated_at.local }} ·
          {{ command.reading.calibration.calibrated_at.timezone }}
        </p>
        <p v-if="command.reading.calibration.reference_temperature">
          {{ t('brewReferenceTemperature') }}:
          {{ exact(command.reading.calibration.reference_temperature.value) }}
          {{ t(readingUnits[command.reading.calibration.reference_temperature.unit]) }}
        </p>
        <p v-if="command.reading.calibration.source_url">
          <a
            :href="command.reading.calibration.source_url"
            target="_blank"
            rel="noopener noreferrer"
            >{{ t('source') }}</a
          >
        </p>
      </template>
      <p v-if="command.corrects">
        {{ t('brewCorrection') }} {{ command.corrects }} · {{ command.reason }}
      </p>
      <p>{{ command.reading.notes }}</p></template
    ><template v-else-if="affected"
      ><h3>{{ affected.run.name }}</h3>
      <p>
        {{ t('brewPinned') }}: {{ affected.run.brew_recipe.title }} ·
        {{ affected.run.brew_recipe.revision }} ·
        {{ compactExact(affected.run.brew_recipe.volume.amount) }}
        {{ affected.run.brew_recipe.volume.unit }}
      </p>
      <ol class="brew-phase-list">
        <li v-for="step in affected.run.steps" :key="step.stage.id">
          <strong>{{ step.stage.name }}</strong>
          <p>{{ step.stage.instruction }}</p>
          <p>
            {{
              localDate(
                affected.run.schedule.find((s) => s.id === step.stage.id)?.start ??
                  preview.after.server_now,
                affected.run.timezone,
              )
            }}
            <template v-if="step.stage.active_seconds + step.stage.passive_seconds > 0">
              →
              {{
                localDate(
                  affected.run.schedule.find((s) => s.id === step.stage.id)?.finish ??
                    preview.after.server_now,
                  affected.run.timezone,
                )
              }}
            </template>
            · {{ affected.run.timezone }}
          </p>
          <p v-if="step.stage.active_seconds + step.stage.passive_seconds === 0" class="muted">
            {{ t('brewUnknownTiming') }}
          </p>
          <p v-if="command.operation === 'replan'" class="muted">
            {{ t('stockBefore') }}:
            {{
              localDate(
                preview.before.batches
                  .find((b) => b.run.id === affected?.run.id)
                  ?.run.schedule.find((s) => s.id === step.stage.id)?.start ??
                  preview.before.server_now,
                affected.run.timezone,
              )
            }}
          </p>
          <p class="muted">
            {{
              step.stage.equipment.length
                ? step.stage.equipment.map(equipmentName).join(', ')
                : t('brewNoEquipment')
            }}
          </p>
        </li>
      </ol></template
    >
    <p v-else>
      {{
        t(
          command.operation === 'archive_recipe' ||
            command.operation === 'archive_equipment' ||
            command.operation === 'archive_batch'
            ? command.archived
              ? 'brewArchive'
              : 'brewRestore'
            : 'save',
        )
      }}
    </p>
    <NcCheckboxRadioSwitch v-model="reviewed" :disabled="disabled">{{
      t('brewReviewed')
    }}</NcCheckboxRadioSwitch>
    <div class="actions">
      <NcButton variant="primary" :disabled="disabled || !reviewed" @click="$emit('apply')">{{
        t('save')
      }}</NcButton
      ><NcButton :disabled="disabled" @click="$emit('back')">{{ t('back') }}</NcButton>
    </div>
  </section>
</template>
