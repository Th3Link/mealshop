<script setup lang="ts">
import { watch, computed, ref, toRaw } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import { mdiPlus } from '@mdi/js'
import Icon from './Icon.vue'
import LabeledField from './LabeledField.vue'
import BrewNumber from './BrewNumber.vue'
import CookingLocalTime from './CookingLocalTime.vue'
import type { BrewReadingInput, BrewReading, BrewRun, BrewCommand } from './brewing-api'
import { readingUnits } from './brewing-format'
import { localInput } from './cooking-format'
import { t } from './i18n'
const props = defineProps<{
  run: BrewRun
  now: number
  correcting?: BrewReading
  disabled: boolean
}>()
const emit = defineEmits<{ review: [value: BrewCommand]; dirty: [value: boolean]; close: [] }>()
const draft = ref<BrewReadingInput>(
  props.correcting
    ? structuredClone(toRaw(props.correcting.input))
    : {
        phase_id: null,
        measurement: { value: '', unit: 'sg' },
        measured_at: {
          local: localInput(props.now, props.run.timezone),
          timezone: props.run.timezone,
          fold: 'reject',
        },
        source: { kind: 'manual' },
        calibration: { kind: 'unknown' },
        notes: '',
      },
)
const reason = ref('')
const source = computed({
  get: () => draft.value.source.kind,
  set: (value: string) => {
    draft.value.source =
      value === 'instrument'
        ? { kind: 'instrument', device_id: '', name: '', acquisition: 'transcribed' }
        : { kind: 'manual' }
  },
})
const calibration = computed({
  get: () => draft.value.calibration.kind,
  set: (value: string) => {
    draft.value.calibration =
      value === 'declared'
        ? {
            kind: 'declared',
            details: '',
            calibrated_at: null,
            reference_temperature: null,
            source_url: null,
          }
        : { kind: 'unknown' }
  },
})
function review() {
  emit('review', {
    operation: 'add_reading',
    run_id: props.run.id,
    reading: structuredClone(toRaw(draft.value)),
    corrects: props.correcting?.id ?? null,
    reason: props.correcting ? reason.value : null,
  })
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
      <legend class="visually-hidden">
        {{ t(correcting ? 'brewCorrectReading' : 'brewAddReading') }}
      </legend>
      <h2>{{ t(correcting ? 'brewCorrectReading' : 'brewAddReading') }}</h2>
      <p class="muted">{{ t('brewOriginalUnits') }}</p>
      <div class="form-row">
        <BrewNumber
          v-model="draft.measurement.value"
          :label="t('brewReadingValue')"
          required
          signed
        /><label class="field"
          >{{ t('unit')
          }}<select v-model="draft.measurement.unit">
            <option v-for="(label, unit) in readingUnits" :key="unit" :value="unit">
              {{ t(label) }}
            </option>
          </select></label
        ><label class="field"
          >{{ t('brewPhase')
          }}<select v-model="draft.phase_id">
            <option :value="null">{{ t('brewNone') }}</option>
            <option v-for="s in run.steps" :key="s.stage.id" :value="s.stage.id">
              {{ s.stage.name }}
            </option>
          </select></label
        >
      </div>
      <CookingLocalTime
        zone-editable
        v-model="draft.measured_at"
        :label="t('brewMeasuredAt')"
      /><label class="field"
        >{{ t('brewReadingSource')
        }}<select v-model="source">
          <option value="manual">{{ t('brewManual') }}</option>
          <option value="instrument">{{ t('brewInstrumentReading') }}</option>
        </select></label
      ><template v-if="draft.source.kind === 'instrument'"
        ><p class="muted">
          {{ t(draft.source.acquisition === 'reported' ? 'brewReported' : 'brewDeviceHelp') }}
        </p>
        <div class="form-row">
          <LabeledField
            v-model="draft.source.name"
            :label="t('brewDeviceName')"
            required
            maxlength="200"
          /><LabeledField
            v-model="draft.source.device_id"
            :label="t('brewDeviceId')"
            required
            maxlength="120"
          /></div
      ></template>
      <label class="field"
        >{{ t('brewCalibration')
        }}<select v-model="calibration">
          <option value="unknown">{{ t('brewCalibrationUnknown') }}</option>
          <option value="declared">{{ t('brewCalibrationDeclared') }}</option>
        </select></label
      ><template v-if="draft.calibration.kind === 'declared'"
        ><LabeledField
          v-model="draft.calibration.details"
          :label="t('brewCalibrationDetails')"
          required
          maxlength="2000"
        />
        <div class="actions">
          <NcButton
            v-if="!draft.calibration.calibrated_at"
            variant="tertiary"
            @click="draft.calibration.calibrated_at = { ...draft.measured_at }"
            ><template #icon><Icon :path="mdiPlus" /></template
            >{{ t('brewCalibrationAt') }}</NcButton
          ><NcButton
            v-if="!draft.calibration.reference_temperature"
            variant="tertiary"
            @click="draft.calibration.reference_temperature = { value: '', unit: 'celsius' }"
            ><template #icon><Icon :path="mdiPlus" /></template
            >{{ t('brewReferenceTemperature') }}</NcButton
          ><NcButton
            v-if="draft.calibration.source_url === null"
            variant="tertiary"
            @click="draft.calibration.source_url = ''"
            ><template #icon><Icon :path="mdiPlus" /></template>{{ t('source') }}</NcButton
          >
        </div>
        <template v-if="draft.calibration.calibrated_at"
          ><CookingLocalTime
            zone-editable
            v-model="draft.calibration.calibrated_at"
            :label="t('brewCalibrationAt')"
          /><NcButton variant="tertiary" @click="draft.calibration.calibrated_at = null"
            >{{ t('remove') }} · {{ t('brewCalibrationAt') }}</NcButton
          ></template
        >
        <div v-if="draft.calibration.reference_temperature" class="form-row">
          <BrewNumber
            v-model="draft.calibration.reference_temperature.value"
            :label="t('brewReferenceTemperature')"
            signed
            required
          /><label class="field"
            >{{ t('unit')
            }}<select v-model="draft.calibration.reference_temperature.unit">
              <option value="celsius">°C</option>
              <option value="fahrenheit">°F</option>
            </select></label
          ><NcButton variant="tertiary" @click="draft.calibration.reference_temperature = null">{{
            t('remove')
          }}</NcButton>
        </div>
        <template v-if="draft.calibration.source_url !== null"
          ><LabeledField
            v-model="draft.calibration.source_url"
            :label="t('source')"
            type="url"
            required
            maxlength="2048"
          /><NcButton variant="tertiary" @click="draft.calibration.source_url = null"
            >{{ t('remove') }} · {{ t('source') }}</NcButton
          ></template
        ></template
      >
      <label class="field"
        >{{ t('brewNotes') }}<textarea v-model="draft.notes" maxlength="2000" rows="2" /></label
      ><LabeledField
        v-if="correcting"
        v-model="reason"
        :label="t('brewCorrectionReason')"
        required
        maxlength="1000"
      />
      <div class="actions">
        <NcButton type="submit" variant="primary">{{ t('brewReview') }}</NcButton
        ><NcButton @click="emit('close')">{{ t('cancel') }}</NcButton>
      </div>
    </fieldset>
  </form>
</template>
