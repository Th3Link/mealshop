<script setup lang="ts">
import { computed, useId } from 'vue'
import NcSelect from '@nextcloud/vue/components/NcSelect'
import { z } from 'zod'
import { t } from './i18n'
const model = defineModel<string[]>({ required: true })
const props = defineProps<{
  label: string
  disabled?: boolean
  options: { id: string; label: string }[]
}>()
const id = useId()
const selected = computed(() => props.options.filter((o) => model.value.includes(o.id)))
function update(value: unknown) {
  const parsed = z.array(z.object({ id: z.string() })).safeParse(value)
  if (parsed.success) model.value = parsed.data.map((o) => o.id)
}
</script>
<template>
  <div class="labeled-field">
    <label :for="id">{{ label }}</label
    ><NcSelect
      :input-id="id"
      :disabled="disabled"
      :input-label="label"
      label-outside
      :model-value="selected"
      :options="options"
      multiple
      :close-on-select="false"
      label="label"
      @update:model-value="update"
      ><template #option="option"
        ><span class="mealshop-selection-name">{{ option.label }}</span></template
      ><template #selected-option="option"
        ><span class="mealshop-selection-name">{{ option.label }}</span></template
      ><template #no-options>{{ t('noSearchResults') }}</template></NcSelect
    >
  </div>
</template>
