<script setup lang="ts">
import { computed } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import { mdiPlus, mdiTrashCanOutline } from '@mdi/js'
import Icon from './Icon.vue'
import LabeledField from './LabeledField.vue'
import CookingDuration from './CookingDuration.vue'
import RecipeReferences from './RecipeReferences.vue'
import BrewQuantity from './BrewQuantity.vue'
import BrewNumber from './BrewNumber.vue'
import type { BrewRecipe, BrewTemplates } from './brewing-api'
import { localized } from './brewing-format'
import { t } from './i18n'
const phase = defineModel<BrewRecipe['phases'][number]>({ required: true })
const props = defineProps<{
  disabled?: boolean
  index: number
  options: { id: string; label: string }[]
  catalogue: BrewTemplates
}>()
const kinds = computed(() => [
  ...new Map(Object.values(props.catalogue.phases).map((p) => [p.kind, p])).values(),
])
defineEmits<{ remove: [] }>()
function timing(known: boolean) {
  phase.value.timing_known = known
  if (!known) {
    phase.value.stage.active_seconds = 0
    phase.value.stage.passive_seconds = 0
    phase.value.stage.passive_min_seconds = 0
    phase.value.stage.passive_max_seconds = 0
  }
}
function waiting(v: number) {
  const s = phase.value.stage
  if (s.passive_min_seconds === s.passive_seconds && s.passive_max_seconds === s.passive_seconds) {
    s.passive_min_seconds = v
    s.passive_max_seconds = v
  }
  s.passive_seconds = v
}
</script>
<template>
  <section class="brew-phase">
    <div class="brew-phase-heading">
      <span class="brew-step-number" aria-hidden="true">{{ index + 1 }}</span
      ><LabeledField
        v-model="phase.stage.name"
        :label="t('cookStageName')"
        required
        maxlength="200"
      /><NcButton
        variant="tertiary"
        :aria-label="`${t('brewRemovePhase')}: ${phase.stage.name}`"
        @click="$emit('remove')"
        ><template #icon><Icon :path="mdiTrashCanOutline" /></template
      ></NcButton>
    </div>
    <label class="field"
      ><span>{{ t('instruction') }}</span
      ><textarea v-model="phase.stage.instruction" required maxlength="4000" rows="2" />
    </label>
    <div class="form-row">
      <label class="field"
        >{{ t('brewPhase')
        }}<select v-model="phase.kind">
          <option v-for="p in kinds" :key="p.kind" :value="p.kind">
            {{ localized(p) }}
          </option>
          <option value="custom">{{ t('brewOther') }}</option>
        </select></label
      ><RecipeReferences
        v-model="phase.stage.dependencies"
        :label="t('cookDependencies')"
        :disabled="disabled"
        :options="options.filter((o) => o.id !== phase.stage.id)"
      />
    </div>
    <NcCheckboxRadioSwitch :model-value="phase.timing_known" @update:model-value="timing">{{
      t('brewKnownTiming')
    }}</NcCheckboxRadioSwitch>
    <p v-if="!phase.timing_known" class="muted">{{ t('brewUnknownTiming') }}</p>
    <div v-if="phase.timing_known" class="form-row">
      <CookingDuration
        v-model="phase.stage.active_seconds"
        :label="t('cookActiveDuration')"
        :max="31536000"
      /><CookingDuration
        :model-value="phase.stage.passive_seconds"
        :label="t('cookPassiveDuration')"
        :max="31536000"
        @update:model-value="waiting"
      /><CookingDuration
        v-model="phase.stage.passive_min_seconds"
        :label="t('cookPassiveMin')"
        :max="31536000"
      /><CookingDuration
        v-model="phase.stage.passive_max_seconds"
        :label="t('cookPassiveMax')"
        :max="31536000"
      />
    </div>
    <div class="form-row">
      <div v-if="phase.temperature" class="brew-quantity">
        <BrewNumber
          v-model="phase.temperature.value"
          :label="t('brewTemperature')"
          required
          signed
        /><label class="field"
          ><span class="visually-hidden">{{ t('brewTemperature') }} · {{ t('unit') }}</span
          ><select
            v-model="phase.temperature.unit"
            :aria-label="`${t('brewTemperature')} · ${t('unit')}`"
          >
            <option value="celsius">°C</option>
            <option value="fahrenheit">°F</option>
          </select></label
        ><NcButton
          variant="tertiary"
          :aria-label="t('removeTemperature')"
          @click="phase.temperature = null"
          ><template #icon><Icon :path="mdiTrashCanOutline" /></template
        ></NcButton>
      </div>
      <BrewQuantity
        v-if="phase.volume"
        v-model="phase.volume"
        :label="t('brewPhaseVolume')"
        volume
      />
    </div>
    <div class="actions">
      <NcButton
        v-if="!phase.temperature"
        variant="tertiary"
        @click="phase.temperature = { value: '', unit: 'celsius' }"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('brewTemperature') }}</NcButton
      ><NcButton
        v-if="!phase.volume"
        variant="tertiary"
        @click="phase.volume = { amount: '', unit: 'l' }"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('brewPhaseVolume') }}</NcButton
      ><NcButton v-else variant="tertiary" @click="phase.volume = null"
        >{{ t('remove') }} · {{ t('brewPhaseVolume') }}</NcButton
      >
    </div>
    <div class="form-row">
      <CookingDuration
        v-model="phase.stage.lead_seconds"
        :label="t('cookReminderLead')"
      /><LabeledField
        v-model="phase.stage.uncertainty"
        :label="t('cookUncertainty')"
        maxlength="1000"
      />
    </div>
  </section>
</template>
