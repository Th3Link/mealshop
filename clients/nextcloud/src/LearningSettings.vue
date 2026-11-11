<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { api, domain } from './api'
import { learningSchema, historyRevisionSchema, historyPath } from './feedback-api'
import type { Learning } from './feedback-api'
import type { z } from 'zod'
import { profilesSchema } from './contracts'
import { useTask } from './state'
import { t } from './i18n'
import { useUnsavedChanges } from './useUnsavedChanges'
const props = defineProps<{
  home: string
  owner: boolean
  profiles: z.infer<typeof profilesSchema> | undefined
  total: number
}>()
const emit = defineEmits<{ changed: [] }>()
const state = ref<Learning>(),
  enabled = ref(false),
  members = ref<string[]>([]),
  notice = ref('')
const { busy, error, run, errorMessage } = useTask()
useUnsavedChanges(() =>
  Boolean(
    state.value &&
    (enabled.value !== state.value.policy.enabled ||
      [...members.value].sort().join('\n') !==
        [...state.value.policy.member_ids].sort().join('\n')),
  ),
)
const missing = computed(() =>
  members.value.filter((id) => !props.profiles?.members.some((p) => p.id === id)),
)
async function load() {
  await run(async () => {
    const data = await api(domain(props.home, '/api/household/learning'), learningSchema)
    state.value = data
    enabled.value = data.policy.enabled
    members.value = [...data.policy.member_ids]
  })
}
async function save(reset = false) {
  if (!state.value || !props.owner || !props.profiles) return
  if (
    (reset || (!enabled.value && state.value.policy.enabled)) &&
    !window.confirm(t('historyLearningResetConfirm'))
  )
    return
  await run(async () => {
    state.value = await api(domain(props.home, '/api/household/learning'), learningSchema, {
      method: 'PUT',
      data: {
        expected_revision: state.value?.revision,
        expected_profile_revision: props.profiles?.revision,
        policy: {
          enabled: reset ? false : enabled.value,
          member_ids: !reset && enabled.value ? members.value : [],
        },
      },
    })
    enabled.value = state.value.policy.enabled
    members.value = [...state.value.policy.member_ids]
    notice.value = t(reset ? 'historyLearningResetDone' : 'historyLearningChanged')
    emit('changed')
  })
}
async function clear() {
  if (!props.owner || !state.value || !window.confirm(t('historyClearConfirm'))) return
  await run(async () => {
    await api(domain(props.home, historyPath()), historyRevisionSchema, {
      method: 'DELETE',
      data: { expected_revision: state.value?.revision, confirmed: true },
    })
    notice.value = t('historyCleared')
    emit('changed')
  })
  if (!error.value) await load()
}
onMounted(load)
</script>
<template>
  <section class="history-learning editor-fields">
    <h2>{{ t('historyLearning') }}</h2>
    <NcLoadingIcon v-if="busy && !state" :aria-label="t('loading')" />
    <NcNoteCard v-if="error" type="error" role="alert"
      >{{ errorMessage(error)
      }}<NcButton :disabled="busy" @click="load">{{ t('reloadSaved') }}</NcButton></NcNoteCard
    >
    <p v-if="notice" role="status">{{ notice }}</p>
    <template v-if="state"
      ><p>
        <strong>{{ t(state.policy.enabled ? 'historyLearningOn' : 'historyLearningOff') }}</strong>
        · {{ t('historyLearnedCooks') }}: {{ state.learned_cooks }} ·
        {{ t('historyLearnedRecipes') }}: {{ state.learned_recipes }}
      </p>
      <p class="muted">{{ t('historyLearningHelp') }}</p>
      <template v-if="owner"
        ><form class="editor-fields" @submit.prevent="save()">
          <fieldset class="editor-fields" :disabled="busy">
            <NcCheckboxRadioSwitch v-model="enabled">{{
              t('historyLearningStart')
            }}</NcCheckboxRadioSwitch>
            <fieldset v-if="enabled" class="history-attendees">
              <legend>{{ t('historyLearningScope') }}</legend>
              <NcCheckboxRadioSwitch
                v-for="person in profiles?.members"
                :key="person.id"
                v-model="members"
                :value="person.id"
                >{{ person.name }}</NcCheckboxRadioSwitch
              >
              <div v-for="id in missing" :key="id" class="actions">
                <span>{{ t('historyRemovedPerson') }}</span
                ><NcButton
                  type="button"
                  @click="members = members.filter((member) => member !== id)"
                  >{{ t('remove') }}</NcButton
                >
              </div>
            </fieldset>
            <p class="muted">{{ t('historyLearningNew') }}</p>
            <div class="actions">
              <NcButton type="submit" :disabled="busy || !profiles || missing.length > 0">{{
                t('historyLearningSave')
              }}</NcButton
              ><NcButton
                type="button"
                :disabled="busy || !state.policy.enabled"
                @click="save(true)"
                >{{ t('historyLearningReset') }}</NcButton
              >
            </div>
          </fieldset>
        </form>
        <NcButton :disabled="busy || total === 0" variant="tertiary" @click="clear">{{
          t('historyClear')
        }}</NcButton></template
      >
      <p v-else class="muted">{{ t('historyOwnerOnly') }}</p></template
    >
  </section>
</template>
