<script setup lang="ts">
import { watch, ref, toRaw } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import { mdiPlus } from '@mdi/js'
import Icon from './Icon.vue'
import LabeledField from './LabeledField.vue'
import BrewQuantity from './BrewQuantity.vue'
import { newEquipment, equipmentKinds } from './brewing-format'
import type { BrewState, BrewCommand } from './brewing-api'
import { t } from './i18n'
const props = defineProps<{ equipment?: BrewState['equipment'][number]; disabled: boolean }>()
const emit = defineEmits<{ review: [command: BrewCommand]; dirty: [value: boolean] }>()
const original = props.equipment ? structuredClone(toRaw(props.equipment.details)) : null
const draft = ref(
  props.equipment ? structuredClone(toRaw(props.equipment.details)) : newEquipment(),
)
function review() {
  emit('review', {
    operation: 'save_equipment',
    id: props.equipment?.id ?? null,
    expected_equipment: original,
    equipment: structuredClone(toRaw(draft.value)),
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
      <legend class="visually-hidden">{{ t('brewEquipment') }}</legend>
      <div class="form-row">
        <LabeledField v-model="draft.name" :label="t('name')" required maxlength="120" /><label
          class="field"
          >{{ t('brewEquipmentKind')
          }}<select v-model="draft.kind">
            <option v-for="(label, kind) in equipmentKinds" :key="kind" :value="kind">
              {{ t(label) }}
            </option>
          </select></label
        >
      </div>
      <LabeledField v-model="draft.model" :label="t('brewModel')" maxlength="200" /><BrewQuantity
        v-if="draft.capacity"
        v-model="draft.capacity"
        :label="t('brewCapacity')"
        volume
      /><NcButton v-else variant="tertiary" @click="draft.capacity = { amount: '', unit: 'l' }"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('brewCapacity') }}</NcButton
      ><NcButton v-if="draft.capacity" variant="tertiary" @click="draft.capacity = null"
        >{{ t('remove') }} · {{ t('brewCapacity') }}</NcButton
      >
      <p class="muted">{{ t('brewVolumeHelp') }}</p>
      <label class="field"
        >{{ t('brewNotes') }}<textarea v-model="draft.notes" maxlength="2000" rows="3" /></label
      ><NcButton type="submit" variant="primary">{{ t('brewReview') }}</NcButton>
    </fieldset>
  </form>
</template>
