<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref, useId } from 'vue'
import axios from '@nextcloud/axios'
import { generateOcsUrl } from '@nextcloud/router'
import NcSelect from '@nextcloud/vue/components/NcSelect'
import NcAvatar from '@nextcloud/vue/components/NcAvatar'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import { z } from 'zod'
import { t } from './i18n'
const model = defineModel<string>({ required: true })
const props = defineProps<{ excluded: string[]; label?: string; disabled?: boolean }>()
const id = useId()
const personSchema = z.object({
  label: z.string(),
  value: z.object({ shareType: z.literal(0), shareWith: z.string() }),
})
const resultSchema = z.object({
  ocs: z.object({
    data: z.object({
      users: z.array(personSchema),
      exact: z.object({ users: z.array(personSchema) }),
    }),
  }),
})
type Person = { id: string; label: string }
const people = ref<Person[]>([]),
  loading = ref(false),
  failed = ref(false)
const selected = computed(() => people.value.find((person) => person.id === model.value) ?? null)
const options = computed(() => people.value.filter((person) => !props.excluded.includes(person.id)))
let timer: ReturnType<typeof setTimeout> | undefined, controller: AbortController | undefined
let generation = 0
function search(query: string) {
  clearTimeout(timer)
  controller?.abort()
  generation++
  timer = setTimeout(() => {
    void load(query, generation)
  }, 250)
}
async function load(query: string, current: number) {
  controller = new AbortController()
  loading.value = true
  failed.value = false
  try {
    const response = await axios.get<unknown>(
      generateOcsUrl('/apps/files_sharing/api/v1/sharees'),
      {
        params: {
          search: query,
          itemType: 'file',
          shareType: [0],
          perPage: 20,
          lookup: false,
          format: 'json',
        },
        signal: controller.signal,
      },
    )
    const data = resultSchema.parse(response.data).ocs.data
    if (current !== generation) return
    people.value = [
      ...new Map(
        [...data.exact.users, ...data.users].map((person) => [
          person.value.shareWith,
          { id: person.value.shareWith, label: person.label },
        ]),
      ).values(),
    ]
  } catch {
    if (current === generation) failed.value = true
  } finally {
    if (current === generation) loading.value = false
  }
}
function select(value: unknown) {
  const parsed = z.object({ id: z.string() }).nullable().safeParse(value)
  if (parsed.success) model.value = parsed.data?.id ?? ''
}
onMounted(() => search(''))
onBeforeUnmount(() => {
  generation++
  clearTimeout(timer)
  controller?.abort()
})
</script>
<template>
  <div class="labeled-field">
    <label :for="id">{{ props.label ?? t('invitePerson') }}</label>
    <NcSelect
      :input-id="id"
      label-outside
      :input-label="props.label ?? t('invitePerson')"
      :placeholder="t('searchPeople')"
      :model-value="selected"
      :options="options"
      :filterable="false"
      :loading="loading"
      :disabled="props.disabled"
      label="label"
      @search="search"
      @update:model-value="select"
    >
      <template #option="person"
        ><div class="mealshop-person-option">
          <NcAvatar :user="person.id" :size="32" /><span
            >{{ person.label }}<small>{{ person.id }}</small></span
          >
        </div></template
      >
      <template #no-options>{{ t('noSearchResults') }}</template>
    </NcSelect>
    <p class="muted">{{ t('searchPeopleHelp') }}</p>
    <NcNoteCard v-if="failed" type="error" role="alert">{{ t('error') }}</NcNoteCard>
  </div>
</template>
