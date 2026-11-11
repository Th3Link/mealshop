<script setup lang="ts">
import { computed, ref } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import { api } from './api'
import { emptySchema } from './contracts'
import { sharedPath } from './library-api'
import type { ShareMetadata } from './library-api'
import { homes, useTask } from './state'
import { t } from './i18n'
const props = defineProps<{ share: ShareMetadata }>()
const emit = defineEmits<{ changed: [] }>()
const destination = ref('')
const owned = computed(() =>
  homes.value.filter((h) => h.role === 'owner' && h.id !== props.share.source_home),
)
const { busy, error, run, errorMessage } = useTask()
async function accept() {
  if (!owned.value.some((h) => h.id === destination.value)) return
  await run(async () => {
    await api(`${sharedPath(props.share.source_home, props.share.id)}/accept`, emptySchema, {
      method: 'PUT',
      data: { expected_version: props.share.version, household_id: destination.value },
    })
    emit('changed')
  })
}
async function decline() {
  await run(async () => {
    await api(sharedPath(props.share.source_home, props.share.id), emptySchema, {
      method: 'DELETE',
      data: { expected_version: props.share.version },
    })
    emit('changed')
  })
}
</script>
<template>
  <section class="shared-invitation">
    <h2>{{ t('sharingInvitation') }}</h2>
    <p>
      {{ t('sharingFrom') }} {{ props.share.owner }} ·
      {{ props.share.resource_kind === 'recipe' ? t('collectionRecipe') : t('collection') }}
    </p>
    <NcNoteCard v-if="error" type="error" role="alert">{{ errorMessage(error) }}</NcNoteCard>
    <form v-if="owned.length" class="editor-fields" @submit.prevent="accept">
      <label class="field"
        >{{ t('sharingDestination')
        }}<select v-model="destination" :disabled="busy" required>
          <option value="" disabled>{{ t('sharingDestination') }}</option>
          <option v-for="home in owned" :key="home.id" :value="home.id">{{ home.name }}</option>
        </select></label
      >
      <p class="muted">{{ t('sharingAcceptHelp') }}</p>
      <div class="actions">
        <NcButton type="submit" variant="primary" :disabled="busy || !destination">{{
          t('sharingAccept')
        }}</NcButton
        ><NcButton type="button" :disabled="busy" @click="decline">{{
          t('sharingDecline')
        }}</NcButton>
      </div>
    </form>
    <template v-else
      ><NcNoteCard type="info">{{ t('sharingNoOwnedHousehold') }}</NcNoteCard>
      <div class="actions">
        <NcButton to="/households">{{ t('households') }}</NcButton
        ><NcButton :disabled="busy" @click="decline">{{ t('sharingDecline') }}</NcButton>
      </div></template
    >
  </section>
</template>
