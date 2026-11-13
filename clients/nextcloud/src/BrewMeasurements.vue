<script setup lang="ts">
import NcButton from '@nextcloud/vue/components/NcButton'
import { mdiPlus } from '@mdi/js'
import Icon from './Icon.vue'
import BrewReadingEditor from './BrewReadingEditor.vue'
import type { BrewDetail, BrewReading, BrewCommand } from './brewing-api'
import { readingUnits } from './brewing-format'
import { t, exact } from './i18n'
import { localDate } from './cooking-format'
defineProps<{
  detail: BrewDetail
  measurements: BrewReading[]
  writable: boolean
  disabled: boolean
  busy: boolean
  offline: boolean
  replan: boolean
  now: number
  nextCursor: string | null
}>()
const reading = defineModel<boolean>('reading', { required: true })
const correcting = defineModel<BrewReading | undefined>('correcting')
const emit = defineEmits<{ review: [command: BrewCommand]; dirty: [value: boolean]; earlier: [] }>()
</script>
<template>
  <section class="brew-readings">
    <div class="section-heading">
      <h2>
        {{ t('brewMeasurements') }}
        <span class="muted">({{ detail.measurements.total }})</span>
      </h2>
      <NcButton
        v-if="writable && !reading"
        :disabled="disabled || !!replan"
        @click="
          () => {
            reading = true
            correcting = undefined
            emit('dirty', true)
          }
        "
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('brewAddReading') }}</NcButton
      >
    </div>
    <BrewReadingEditor
      v-if="reading"
      :key="correcting?.id ?? 'new'"
      :run="detail.run"
      :now="now"
      :correcting="correcting"
      :disabled="disabled"
      @dirty="emit('dirty', $event)"
      @review="emit('review', $event)"
      @close="
        () => {
          reading = false
          correcting = undefined
          emit('dirty', false)
        }
      "
    />
    <p v-if="!measurements.length" class="muted">{{ t('brewReadingEmpty') }}</p>
    <ul class="plain-list">
      <li v-for="r in measurements" :key="r.id">
        <div class="section-heading">
          <strong
            >{{ exact(r.input.measurement.value) }}
            {{ t(readingUnits[r.input.measurement.unit]) }}</strong
          ><NcButton
            v-if="writable && !measurements.some((next) => next.corrects === r.id)"
            variant="tertiary"
            :disabled="disabled || reading || !!replan"
            @click="
              () => {
                reading = true
                correcting = r
                emit('dirty', true)
              }
            "
            >{{ t('brewCorrectReading') }}</NcButton
          >
        </div>
        <p>
          {{ localDate(r.measured_instant, r.input.measured_at.timezone) }} ·
          {{ r.input.measured_at.timezone }} ·
          {{
            r.input.phase_id
              ? detail.run.steps.find((s) => s.stage.id === r.input.phase_id)?.stage.name
              : t('brewNone')
          }}
        </p>
        <p class="muted">
          {{ t(r.input.source.kind === 'manual' ? 'brewManual' : 'brewInstrumentReading')
          }}<template v-if="r.input.source.kind === 'instrument'">
            · {{ r.input.source.name }} · {{ r.input.source.device_id }} ·
            {{
              r.input.source.acquisition === 'reported' ? t('brewReported') : t('brewInstrument')
            }}</template
          >
        </p>
        <p>
          {{
            r.input.calibration.kind === 'declared'
              ? r.input.calibration.details
              : t('brewCalibrationUnknown')
          }}
        </p>
        <p
          v-if="
            r.input.calibration.kind === 'declared' && r.input.calibration.reference_temperature
          "
        >
          {{ t('brewReferenceTemperature') }}:
          {{ exact(r.input.calibration.reference_temperature.value) }}
          {{ t(readingUnits[r.input.calibration.reference_temperature.unit]) }}
        </p>
        <template v-if="r.input.calibration.kind === 'declared'">
          <p v-if="r.input.calibration.calibrated_at" class="muted">
            {{ t('brewCalibrationAt') }}: {{ r.input.calibration.calibrated_at.local }} ·
            {{ r.input.calibration.calibrated_at.timezone }}
          </p>
          <p v-if="r.input.calibration.source_url">
            <a :href="r.input.calibration.source_url" target="_blank" rel="noopener noreferrer">{{
              t('source')
            }}</a>
          </p>
        </template>
        <p v-if="r.corrects">{{ t('brewCorrection') }} {{ r.corrects }} · {{ r.reason }}</p>
        <p v-if="measurements.some((next) => next.corrects === r.id)" class="muted">
          {{ t('brewCorrected') }}
        </p>
        <p v-if="r.input.notes">{{ r.input.notes }}</p>
        <small class="muted"
          >{{ t('brewEnteredAt') }}: {{ localDate(r.recorded_at, detail.run.timezone) }} ·
          {{ r.actor }}</small
        >
      </li>
    </ul>
    <NcButton v-if="nextCursor" :disabled="busy || offline" @click="emit('earlier')">{{
      t('brewEarlier')
    }}</NcButton>
  </section>
</template>
