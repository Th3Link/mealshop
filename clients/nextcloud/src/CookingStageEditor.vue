<script setup lang="ts">
import { computed, ref } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextField from './LabeledField.vue'
import RecipeReferences from './RecipeReferences.vue'
import CookingLocalTime from './CookingLocalTime.vue'
import { localInput, zone } from './cooking-format'
import CookingDuration from './CookingDuration.vue'
import type { Stage } from './cooking-api'
import { duration } from './cooking-format'
import { t } from './i18n'
const stage = defineModel<Stage>({ required: true })
const initiallyOpen = ref(!stage.value.name)
const props = defineProps<{ index: number; options: { id: string; label: string }[] }>()
defineEmits<{ remove: [] }>()
const equipment = computed({
  get: () => stage.value.equipment.join('\n'),
  set: (value: string) =>
    (stage.value.equipment = value
      .split('\n')
      .map((s) => s.trim())
      .filter(Boolean)),
})
function passive(value: number) {
  if (
    stage.value.passive_min_seconds === stage.value.passive_seconds &&
    stage.value.passive_max_seconds === stage.value.passive_seconds
  ) {
    stage.value.passive_min_seconds = value
    stage.value.passive_max_seconds = value
  }
  stage.value.passive_seconds = value
}
</script>
<template>
  <details class="cooking-stage-editor" :open="initiallyOpen">
    <summary>
      <strong>{{ index + 1 }}. {{ stage.name || t('cookStageName') }}</strong
      ><span class="muted">
        · {{ duration(stage.active_seconds + stage.passive_seconds)
        }}<template v-if="stage.repeat > 1"> · ×{{ stage.repeat }}</template></span
      >
    </summary>
    <div class="editor-fields">
      <NcTextField v-model="stage.name" :label="t('cookStageName')" required maxlength="200" />
      <label class="field"
        >{{ t('instruction') }}<textarea v-model="stage.instruction" required maxlength="4000" />
      </label>
      <NcTextField
        v-if="stage.origin.kind === 'custom'"
        v-model="stage.origin.note"
        :label="t('cookSourceNote')"
        required
        maxlength="500"
      />
      <p v-else class="muted">{{ t('cookRecipeStep') }} {{ stage.origin.index + 1 }}</p>
      <div class="form-row">
        <CookingDuration
          v-model="stage.active_seconds"
          :label="t('cookActiveDuration')"
        /><CookingDuration
          :model-value="stage.passive_seconds"
          :label="t('cookPassiveDuration')"
          @update:model-value="passive"
        />
      </div>
      <RecipeReferences
        v-model="stage.dependencies"
        :label="t('cookDependencies')"
        :options="props.options.filter((s) => s.id !== stage.id)"
      />
      <CookingLocalTime
        v-if="stage.start_at"
        v-model="stage.start_at"
        :label="t('preparationTime')"
      />
      <NcButton
        v-if="!stage.start_at"
        variant="tertiary"
        @click="
          stage.start_at = {
            local: localInput(Math.floor(Date.now() / 1000), zone()),
            timezone: zone(),
            fold: 'reject',
          }
        "
        >{{ t('preparationTime') }}</NcButton
      >
      <NcButton v-else variant="tertiary" @click="stage.start_at = undefined"
        >{{ t('remove') }} {{ t('preparationTime') }}</NcButton
      >
      <details>
        <summary>{{ t('cookStageAdvanced') }}</summary>
        <div class="editor-fields">
          <label class="field"
            >{{ t('cookEquipment')
            }}<textarea
              v-model="equipment"
              maxlength="2500"
              :aria-description="t('cookEquipmentHelp')"
            />
          </label>
          <p class="muted">{{ t('cookEquipmentHelp') }}</p>
          <CookingDuration v-model="stage.lead_seconds" :label="t('cookReminderLead')" />
          <div class="form-row">
            <NcTextField
              v-model="stage.repeat"
              type="number"
              :label="t('cookRepeat')"
              min="1"
              max="12"
              step="1"
              required
            /><CookingDuration
              v-if="Number(stage.repeat) > 1"
              v-model="stage.wait_between_seconds"
              :label="t('cookRepeatWait')"
            />
          </div>
          <div class="form-row">
            <CookingDuration
              v-model="stage.passive_min_seconds"
              :label="t('cookPassiveMin')"
            /><CookingDuration v-model="stage.passive_max_seconds" :label="t('cookPassiveMax')" />
          </div>
          <label class="field"
            >{{ t('cookUncertainty') }}<textarea v-model="stage.uncertainty" maxlength="1000" />
          </label>
          <p class="muted">{{ t('cookUncertaintyHelp') }}</p>
        </div>
      </details>
      <NcButton variant="tertiary" @click="$emit('remove')">{{ t('cookRemoveStage') }}</NcButton>
    </div>
  </details>
</template>
