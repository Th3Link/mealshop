<script setup lang="ts">
import { ref } from 'vue'
import type { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import { mdiStarOutline } from '@mdi/js'
import Icon from './Icon.vue'
import LabeledField from './LabeledField.vue'
import StarRating from './StarRating.vue'
import { api, domain } from './api'
import { profilesSchema, type recipeRecordSchema } from './contracts'
import { historyListSchema, historyPath, historySavedSchema } from './feedback-api'
import { localInput, zone } from './cooking-format'
import { useTask } from './state'
import { t } from './i18n'
const props = defineProps<{ home: string; recipe: z.infer<typeof recipeRecordSchema> }>()
const emit = defineEmits<{ saved: [] }>()
const open = ref(false),
  saved = ref(false)
const rating = ref<number | null>(null),
  date = ref(''),
  attendees = ref<string[]>([])
const profiles = ref<z.infer<typeof profilesSchema>>(),
  revision = ref(0)
const { busy, error, run, errorMessage } = useTask()
async function start() {
  saved.value = false
  await run(async () => {
    const [people, history] = await Promise.all([
      api(domain(props.home, '/api/household/profiles'), profilesSchema),
      api(domain(props.home, historyPath()), historyListSchema),
    ])
    profiles.value = people
    revision.value = history.revision
    attendees.value = people.members.map((member) => member.id)
    date.value = localInput(Math.floor(Date.now() / 1000), zone()).slice(0, 10)
    rating.value = null
    open.value = true
  })
}
async function save() {
  await run(async () => {
    await api(domain(props.home, historyPath()), historySavedSchema, {
      method: 'POST',
      data: {
        source: {
          kind: 'manual',
          recipe_id: props.recipe.id,
          revision: props.recipe.revision,
          servings: props.recipe.recipe.servings,
        },
        input: {
          date: date.value,
          timezone: zone(),
          attendees: attendees.value,
          active_seconds: null,
          household_rating: rating.value,
          ratings: {},
        },
        confirmed_completed: true,
        expected_revision: revision.value,
        expected_profile_revision: profiles.value?.revision,
      },
    })
    open.value = false
    saved.value = true
    emit('saved')
  })
}
</script>
<template>
  <div class="recipe-rate">
    <NcNoteCard v-if="error" type="error" role="alert">{{ errorMessage(error) }}</NcNoteCard>
    <NcNoteCard v-if="saved" type="success">{{ t('rateSaved') }}</NcNoteCard>
    <NcButton v-if="!open" :disabled="busy" @click="start"
      ><template #icon><Icon :path="mdiStarOutline" /></template>{{ t('rateRecipe') }}</NcButton
    >
    <form v-else class="recipe-rate-form" @submit.prevent="save">
      <h2>{{ t('rateTitle') }}</h2>
      <NcNoteCard v-if="!profiles?.members.length" type="info"
        >{{ t('rateNoPeople') }}
        <NcButton :to="`/h/${home}/household`">{{ t('household') }}</NcButton></NcNoteCard
      >
      <template v-else>
        <StarRating v-model="rating" :label="t('historyHouseholdRating')" :disabled="busy" />
        <LabeledField v-model="date" type="date" :label="t('rateDate')" required />
        <fieldset :disabled="busy">
          <legend>{{ t('rateWho') }}</legend>
          <NcCheckboxRadioSwitch
            v-for="person in profiles.members"
            :key="person.id"
            v-model="attendees"
            :value="person.id"
            name="rate-attendees"
            >{{ person.name }}</NcCheckboxRadioSwitch
          >
        </fieldset>
      </template>
      <div class="actions">
        <NcButton
          v-if="profiles?.members.length"
          type="submit"
          variant="primary"
          :disabled="busy || !attendees.length || !date"
          >{{ t('rateSave') }}</NcButton><NcButton :disabled="busy" @click="open = false">{{
          t('cancel')
        }}</NcButton>
      </div>
    </form>
  </div>
</template>
