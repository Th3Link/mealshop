<script setup lang="ts">
import { ref, watch } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import LabeledField from './LabeledField.vue'
import { api, domain } from './api'
import { useTask } from './state'
import { t } from './i18n'
import { importPreviewSchema, type ImportPreview } from './recipe-import'
const props = defineProps<{ home: string; writable: boolean }>()
const url = defineModel<string>({ required: true })
const emit = defineEmits<{ review: [preview: ImportPreview, candidate: number]; changed: [] }>()
const consent = ref(false),
  preview = ref<ImportPreview>(),
  choice = ref(0)
const { busy, error, run, errorMessage } = useTask()
watch([url, consent], () => {
  preview.value = undefined
  emit('changed')
})
async function fetchSource(manual = false) {
  await run(async () => {
    const result = await api(
      domain(props.home, '/api/household/imports/preview'),
      importPreviewSchema,
      {
        method: 'POST',
        data: manual
          ? { mode: 'manual', url: url.value }
          : { mode: 'url', url: url.value, consent: consent.value },
      },
    )
    choice.value = 0
    if (manual) emit('review', result, 0)
    else preview.value = result
  })
}
</script>
<template>
  <section class="import-entry">
    <p>{{ t('importIntro') }}</p>
    <NcNoteCard type="info">{{ t('importPrivacy') }}</NcNoteCard>
    <NcNoteCard v-if="error" type="error" role="alert">{{ errorMessage(error) }}</NcNoteCard>
    <form @submit.prevent="fetchSource()">
      <fieldset :disabled="busy || !writable">
        <LabeledField v-model="url" :label="t('importUrl')" type="url" required maxlength="2048" />
        <NcCheckboxRadioSwitch v-model="consent">{{ t('importConsent') }}</NcCheckboxRadioSwitch>
        <div class="actions">
          <NcButton type="submit" variant="primary" :disabled="!consent || !url.trim()">{{
            t('importFetch')
          }}</NcButton>
          <NcButton :disabled="!url.trim()" @click="fetchSource(true)">{{
            t('importManual')
          }}</NcButton>
        </div>
      </fieldset>
    </form>
    <NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
    <section v-if="preview" class="import-preview" aria-live="polite">
      <p>{{ t('importNotSaved') }}</p>
      <label class="field"
        >{{ t('importChoose')
        }}<select v-model.number="choice">
          <option v-for="(candidate, index) in preview.candidates" :key="index" :value="index">
            {{ candidate.title || t('importUnnamed') }}
          </option>
        </select></label
      >
      <p>
        {{ t('importSourceYield') }}:
        {{ preview.candidates[choice]?.yield_text || t('importUnknown') }}
      </p>
      <NcButton variant="primary" @click="emit('review', preview, choice)">{{
        t('importContinue')
      }}</NcButton>
    </section>
  </section>
</template>
