<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, toRaw, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { mdiPlus, mdiTrashCanOutline } from '@mdi/js'
import Icon from './Icon.vue'
import LabeledField from './LabeledField.vue'
import HistoryEditor from './HistoryEditor.vue'
import LearningSettings from './LearningSettings.vue'
import { api, domain, ApiError } from './api'
import { profilesSchema, recipesSchema } from './contracts'
import { cookingDetailSchema } from './cooking-api'
import {
  historyListSchema,
  historyDetailSchema,
  historyDraftSchema,
  historySavedSchema,
  historyRevisionSchema,
  historyPath,
} from './feedback-api'
import type { HistoryDraft, HistoryEntry } from './feedback-api'
import { homes, loadHomes, useTask } from './state'
import { t, locale, servingCount } from './i18n'
import { duration, localInput, zone } from './cooking-format'
import { useUnsavedChanges } from './useUnsavedChanges'
const route = useRoute(),
  router = useRouter()
const home = computed(() => String(route.params.home)),
  id = computed(() => String(route.params.id ?? '')),
  base = computed(() => `/h/${home.value}/history`)
const current = computed(() => homes.value.find((h) => h.id === home.value)),
  writable = computed(() => current.value?.role === 'owner' || current.value?.role === 'editor')
const editing = computed(() => id.value === 'new' || route.query.edit === '1')
const profiles = ref<z.infer<typeof profilesSchema>>(),
  recipes = ref<z.infer<typeof recipesSchema>['recipes']>([])
const list = ref<z.infer<typeof historyListSchema>>(),
  selected = ref<HistoryEntry>(),
  draft = ref<HistoryDraft>(),
  sourceTitle = ref('')
const revision = ref(0),
  dirty = ref(false),
  validation = ref(''),
  notice = ref(''),
  search = ref(typeof route.query.q === 'string' ? route.query.q : '')
const page = computed(() => Math.max(1, Number(route.query.page) || 1))
const { busy, error, run, errorMessage } = useTask()
const discard = useUnsavedChanges(() => dirty.value)
let generation = 0,
  timer: ReturnType<typeof setTimeout> | undefined
onBeforeUnmount(() => {
  generation++
  clearTimeout(timer)
})
function changed() {
  dirty.value = true
  notice.value = ''
  validation.value = ''
}
function date(value: string) {
  return new Intl.DateTimeFormat(locale.value, { timeZone: 'UTC', dateStyle: 'medium' }).format(
    new Date(`${value}T12:00:00Z`),
  )
}
function name(value: string) {
  return profiles.value?.members.find((p) => p.id === value)?.name ?? t('historyRemovedPerson')
}
function seed(): HistoryDraft {
  return {
    source: { kind: 'manual', recipe_id: '', revision: 1, servings: 1 },
    input: {
      date: localInput(Math.floor(Date.now() / 1000), zone()).slice(0, 10),
      timezone: zone(),
      attendees: [],
      active_seconds: null,
      household_rating: null,
      ratings: {},
    },
    confirmed_completed: false,
  }
}
async function load() {
  const token = ++generation,
    target = id.value
  selected.value = undefined
  draft.value = undefined
  list.value = undefined
  validation.value = ''
  dirty.value = false
  sourceTitle.value = ''
  profiles.value = undefined
  notice.value = ''
  await run(async () => {
    await loadHomes()
    const people = await api(domain(home.value, '/api/household/profiles'), profilesSchema)
    if (token !== generation) return
    profiles.value = people
    if (!target) {
      const data = await api(
        `${domain(home.value, historyPath())}?${new URLSearchParams({ page: String(page.value), q: search.value })}`,
        historyListSchema,
      )
      if (token === generation) {
        list.value = data
        revision.value = data.revision
      }
      return
    }
    if (target === 'new') {
      if (!writable.value) throw new ApiError('household_write_required', 403)
      const session = typeof route.query.session === 'string' ? route.query.session : undefined
      const existing = await api(
        `${domain(home.value, historyPath())}${session ? `?session=${encodeURIComponent(session)}` : ''}`,
        historyListSchema,
      )
      if (token !== generation) return
      revision.value = existing.revision
      if (session && existing.entries[0]) {
        await router.replace(`${base.value}/${existing.entries[0].id}`)
        return
      }
      const value = seed()
      if (session) {
        const data = await api(
          domain(home.value, `/api/household/cooking/${encodeURIComponent(session)}`),
          cookingDetailSchema,
        )
        if (data.run.status !== 'completed') throw new ApiError('history_session_incomplete', 422)
        value.source = { kind: 'session', run_id: session }
        value.input.timezone = data.run.timezone
        const finished = data.run.steps.reduce(
          (at, s) => (s.timer.state === 'completed' ? Math.max(at, s.timer.completed_at) : at),
          data.run.created_at,
        )
        value.input.date = localInput(finished, data.run.timezone).slice(0, 10)
        sourceTitle.value = `${data.run.recipe.title} · ${t('revision')} ${data.run.recipe.revision} · ${servingCount(data.run.servings)}`
      }
      if (token === generation) draft.value = value
    } else {
      const data = await api(domain(home.value, historyPath(target)), historyDetailSchema)
      if (token !== generation) return
      selected.value = data.entry
      revision.value = data.revision
      if (editing.value && writable.value) {
        const entry = data.entry
        draft.value = {
          source:
            entry.source.kind === 'session'
              ? entry.source
              : {
                  kind: 'manual',
                  recipe_id: entry.recipe.id,
                  revision: entry.recipe.revision,
                  servings: entry.servings,
                },
          input: structuredClone(toRaw(entry.input)),
          confirmed_completed: false,
        }
        sourceTitle.value = `${entry.recipe.title} · ${t('revision')} ${entry.recipe.revision} · ${servingCount(entry.servings)}`
      }
    }
    if (draft.value?.source.kind === 'manual') {
      const data = await api(domain(home.value, '/api/recipes'), recipesSchema)
      if (token === generation) recipes.value = data.recipes
    }
  })
  if (token === generation && target) {
    await nextTick()
    document.querySelector<HTMLElement>('.mealshop h1')?.focus()
  }
}
async function save() {
  if (!draft.value || !profiles.value) return
  const source =
    draft.value.source.kind === 'manual'
      ? {
          ...draft.value.source,
          revision: Number(draft.value.source.revision),
          servings: Number(draft.value.source.servings),
        }
      : draft.value.source
  if (source.kind === 'manual' && !source.recipe_id) {
    validation.value = t('historyChooseRecipe')
    return
  }
  if (!draft.value.input.attendees.length) {
    validation.value = t('historyChooseAttendees')
    return
  }
  const data = historyDraftSchema.safeParse({ ...draft.value, source })
  if (!data.success) {
    validation.value =
      draft.value.input.active_seconds === -1 ? t('cookDurationError') : t('invalid')
    return
  }
  validation.value = ''
  await run(async () => {
    const saved = await api(
      domain(home.value, historyPath(id.value === 'new' ? undefined : id.value)),
      historySavedSchema,
      {
        method: id.value === 'new' ? 'POST' : 'PUT',
        data: {
          ...data.data,
          expected_revision: revision.value,
          expected_profile_revision: profiles.value?.revision,
        },
      },
    )
    dirty.value = false
    await router.replace(`${base.value}/${saved.id}`)
    notice.value = t('historySaved')
  })
}
async function remove() {
  if (!selected.value || !window.confirm(t('historyDeleteConfirm'))) return
  await run(async () => {
    await api(domain(home.value, historyPath(id.value)), historyRevisionSchema, {
      method: 'DELETE',
      data: { expected_revision: revision.value, confirmed: true },
    })
    dirty.value = false
    await router.push(base.value)
  })
}
async function learningChanged(message: string) {
  await load()
  notice.value = message
}
async function reload() {
  if (discard()) await load()
}
watch(search, () => {
  clearTimeout(timer)
  timer = setTimeout(() => {
    void router.replace({ query: { q: search.value || undefined } })
  }, 250)
})
watch(
  () => route.fullPath,
  () => {
    const q = typeof route.query.q === 'string' ? route.query.q : ''
    if (search.value !== q) search.value = q
    void load()
  },
  { immediate: true },
)
function paginate(delta: number) {
  void router.push({ query: { q: search.value || undefined, page: page.value + delta } })
}
</script>
<template>
  <NcNoteCard v-if="error" type="error" role="alert"
    >{{ errorMessage(error)
    }}<NcButton :disabled="busy" @click="reload">{{ t('historyReload') }}</NcButton></NcNoteCard
  >
  <NcButton v-if="id && !selected && !draft && !busy" :to="base">{{ t('back') }}</NcButton>
  <p v-if="notice" role="status">{{ notice }}</p>
  <NcLoadingIcon v-if="busy && !draft && !selected" :aria-label="t('loading')" />
  <template v-if="!id"
    ><header class="page-header">
      <div>
        <h1 tabindex="-1">{{ t('history') }}</h1>
        <p>{{ t('historyIntro') }}</p>
      </div>
      <NcButton v-if="writable" variant="primary" :to="`${base}/new`"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('historyNew') }}</NcButton
      >
    </header>
    <div class="toolbar">
      <LabeledField
        v-model="search"
        type="search"
        :label="t('historySearch')"
        :show-trailing-button="Boolean(search)"
        trailing-button-icon="close"
        :trailing-button-label="t('clearSearch')"
        @trailing-button-click="search = ''"
      /><span v-if="list" role="status">{{ t('history') }}: {{ list.total }}</span>
    </div>
    <NcEmptyContent
      v-if="list && !list.total"
      :name="search ? t('noResults') : t('historyEmpty')"
      :description="search ? '' : t('historyEmptyHelp')"
    />
    <ul class="record-list">
      <li v-for="entry in list?.entries" :key="entry.id">
        <RouterLink :to="`${base}/${entry.id}`"
          ><div>
            <strong :lang="entry.recipe.language">{{ entry.recipe.title }}</strong
            ><span>{{ date(entry.input.date) }} · {{ servingCount(entry.servings) }}</span
            ><span>{{ entry.input.attendees.map(name).join(', ') }}</span>
          </div>
          <div class="history-summary">
            <span>{{
              entry.input.household_rating === null
                ? t('historyNoRating')
                : `${entry.input.household_rating} / 5`
            }}</span
            ><span>{{
              entry.input.active_seconds === null
                ? t('historyEffortUnknown')
                : duration(entry.input.active_seconds)
            }}</span>
          </div></RouterLink
        >
      </li>
    </ul>
    <nav v-if="list && list.total > list.per_page" class="actions" :aria-label="t('libraryPage')">
      <NcButton :disabled="busy || page <= 1" @click="paginate(-1)">{{
        t('libraryPreviousPage')
      }}</NcButton
      ><span>{{ t('libraryPage') }} {{ page }}</span
      ><NcButton :disabled="busy || page * list.per_page >= list.total" @click="paginate(1)">{{
        t('libraryNextPage')
      }}</NcButton>
    </nav>
    <LearningSettings
      v-if="profiles"
      :home="home"
      :owner="current?.role === 'owner'"
      :profiles="profiles"
      @changed="learningChanged"
    />
  </template>
  <template v-else-if="editing && draft"
    ><NcButton :to="selected ? `${base}/${selected.id}` : base" :disabled="busy">{{
      t('cancel')
    }}</NcButton>
    <header class="page-header">
      <h1 tabindex="-1">{{ t(selected ? 'historyEdit' : 'historyNew') }}</h1>
      <NcButton
        variant="primary"
        type="submit"
        form="history-form"
        :disabled="busy || !writable || !draft.confirmed_completed"
        >{{ t('save') }}</NcButton
      >
    </header>
    <NcNoteCard v-if="validation" type="error" role="alert">{{ validation }}</NcNoteCard
    ><NcButton v-if="profiles && !profiles.members.length" :to="`/h/${home}/household`">{{
      t('household')
    }}</NcButton>
    <form id="history-form" @submit.prevent="save">
      <HistoryEditor
        v-model="draft"
        :recipes="recipes"
        :people="profiles?.members ?? []"
        :busy="busy"
        :session-title="sourceTitle"
        @changed="changed"
      /></form
  ></template>
  <template v-else-if="selected"
    ><NcButton :to="base">{{ t('back') }}</NcButton>
    <header class="page-header">
      <div>
        <h1 tabindex="-1" :lang="selected.recipe.language">{{ selected.recipe.title }}</h1>
        <p>
          {{ date(selected.input.date) }} · {{ selected.input.timezone }} ·
          {{ servingCount(selected.servings) }}
        </p>
      </div>
      <div v-if="writable" class="actions library-header-actions">
        <NcButton :to="{ query: { edit: '1' } }">{{ t('historyEdit') }}</NcButton
        ><NcButton :disabled="busy" :aria-label="t('historyDelete')" @click="remove"
          ><template #icon><Icon :path="mdiTrashCanOutline" /></template
          >{{ t('historyDelete') }}</NcButton
        >
      </div>
    </header>
    <p class="muted">
      {{ t(selected.source.kind === 'session' ? 'historySession' : 'historyManual') }}
    </p>
    <p>
      <RouterLink
        :to="{
          path: `/h/${home}/recipes/${selected.recipe.id}`,
          query: { revision: selected.recipe.revision, history: selected.id },
        }"
        >{{ t('historySourceRecipe') }} {{ selected.recipe.revision }}</RouterLink
      >
    </p>
    <dl class="history-facts">
      <div>
        <dt>{{ t('historyPeople') }}</dt>
        <dd>{{ selected.input.attendees.map(name).join(', ') }}</dd>
      </div>
      <div>
        <dt>{{ t('historyEffort') }}</dt>
        <dd>
          {{
            selected.input.active_seconds === null
              ? t('historyEffortUnknown')
              : duration(selected.input.active_seconds)
          }}
        </dd>
      </div>
      <div>
        <dt>{{ t('historyHouseholdRating') }}</dt>
        <dd>
          {{
            selected.input.household_rating === null
              ? t('historyNoRating')
              : `${selected.input.household_rating} / 5`
          }}
        </dd>
      </div>
    </dl>
    <section>
      <h2>{{ t('historyAttendeeRating') }}</h2>
      <ul class="plain-list">
        <li v-for="person in selected.input.attendees" :key="person">
          {{ name(person) }} ·
          {{
            selected.input.ratings[person] === undefined
              ? t('historyNoRating')
              : `${selected.input.ratings[person]} / 5`
          }}
        </li>
      </ul>
    </section>
    <p class="muted">
      {{ t('historyRecordedBy') }} {{ selected.recorded_by
      }}<template v-if="selected.updated_at !== selected.recorded_at">
        · {{ t('historyUpdatedBy') }} {{ selected.updated_by }}</template
      >
    </p>
  </template>
</template>
