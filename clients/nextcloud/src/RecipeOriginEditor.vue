<script setup lang="ts">
import { computed } from 'vue'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import LabeledField from './LabeledField.vue'
import type { RecipeOrigin } from './contracts'
import { t } from './i18n'
const model = defineModel<RecipeOrigin | undefined>()
const props = defineProps<{ locked?: boolean }>()
const imported = computed({
  get: () => model.value?.kind === 'imported',
  set: (value: boolean) => {
    model.value = value ? { kind: 'imported', source_url: '' } : undefined
  },
})
const url = computed({
  get: () => (model.value?.kind === 'imported' ? model.value.source_url : ''),
  set: (source_url: string | number) => {
    model.value = { kind: 'imported', source_url: String(source_url) }
  },
})
</script>
<template>
  <section class="editor-fields">
    <NcCheckboxRadioSwitch v-model="imported" :disabled="props.locked">{{
      t('recipeFromSource')
    }}</NcCheckboxRadioSwitch>
    <template v-if="imported">
      <LabeledField
        v-model="url"
        type="url"
        :label="t('originalRecipeUrl')"
        :readonly="props.locked"
        pattern="[hH][tT][tT][pP][sS]://.*"
        maxlength="2048"
        required
        :helper-text="t('recipeOriginHelp')"
      />
      <NcNoteCard type="info">{{ t('sourceOnlyShareHelp') }}</NcNoteCard>
    </template>
  </section>
</template>
