<script setup lang="ts">
import { computed, onBeforeUnmount, ref, toRaw, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { mdiPlus, mdiTrashCanOutline } from '@mdi/js'
import Icon from './Icon.vue'
import { api, domain, imageUrl } from './api'
import { recipesSchema } from './contracts'
import {
  collectionPath,
  collectionDraftSchema,
  collectionListSchema,
  collectionDetailSchema,
  librarySavedSchema,
  libraryRevisionSchema,
} from './library-api'
import type { CollectionDraft } from './library-api'
import CollectionEditor from './CollectionEditor.vue'
import ShareControls from './ShareControls.vue'
import LabeledField from './LabeledField.vue'
import { homes, useTask } from './state'
import { t, servingCount } from './i18n'
import { useUnsavedChanges } from './useUnsavedChanges'
const route = useRoute(),
  router = useRouter()
const home = computed(() => String(route.params.home)),
  id = computed(() => String(route.params.id ?? ''))
const current = computed(() => homes.value.find((h) => h.id === home.value))
const writable = computed(() => current.value?.role === 'owner' || current.value?.role === 'editor')
const listPath = computed(() => `/h/${home.value}/collections`)
const editing = computed(() => id.value === 'new' || route.query.edit === '1')
const page = computed(() => Math.max(1, Number(route.query.page) || 1))
const search = ref(typeof route.query.q === 'string' ? route.query.q : '')
const records = ref<z.infer<typeof collectionListSchema>>(),
  selected = ref<z.infer<typeof collectionDetailSchema>>()
const draft = ref<CollectionDraft>(),
  recipes = ref<z.infer<typeof recipesSchema>['recipes']>([])
const dirty = ref(false),
  saved = ref(false),
  validation = ref(''),
  revision = ref(0)
const { busy, error, run, errorMessage } = useTask()
const discard = useUnsavedChanges(() => dirty.value)
function markChanged() {
  dirty.value = true
  saved.value = false
}
let generation = 0
let searchTimer: ReturnType<typeof setTimeout> | undefined
onBeforeUnmount(() => {
  generation++
  clearTimeout(searchTimer)
})
async function load() {
  const token = ++generation
  selected.value = undefined
  records.value = undefined
  draft.value = undefined
  dirty.value = false
  validation.value = ''
  await run(async () => {
    if (!id.value || id.value === 'new') {
      const data = await api(
        `${collectionPath(home.value)}?page=${page.value}&q=${encodeURIComponent(search.value)}`,
        collectionListSchema,
      )
      if (token !== generation) return
      revision.value = data.revision
      if (!id.value) {
        records.value = data
        return
      }
    } else {
      const data = await api(
        `${collectionPath(home.value, id.value)}?page=${page.value}`,
        collectionDetailSchema,
      )
      if (token !== generation) return
      selected.value = data
      revision.value = data.revision
      if (!editing.value || !data.editable) return
    }
    const data = await api(domain(home.value, '/api/recipes'), recipesSchema)
    if (token !== generation) return
    recipes.value = data.recipes
    draft.value = selected.value
      ? structuredClone(toRaw(selected.value.collection))
      : { name: '', description: '', entries: [] }
  })
}
async function reload() {
  if (discard()) await load()
}
async function save() {
  if (!draft.value) return
  const checked = collectionDraftSchema.safeParse({
    ...draft.value,
    entries: draft.value.entries.map((entry) => ({ ...entry, revision: Number(entry.revision) })),
  })
  if (!checked.success) {
    validation.value = t('required')
    return
  }
  validation.value = ''
  await run(async () => {
    const value = await api(
      collectionPath(home.value, id.value === 'new' ? undefined : id.value),
      librarySavedSchema,
      {
        method: id.value === 'new' ? 'POST' : 'PUT',
        data: { expected_revision: revision.value, collection: checked.data },
      },
    )
    dirty.value = false
    saved.value = true
    await router.replace(`${listPath.value}/${value.id}`)
  })
}
async function remove() {
  if (!selected.value || !window.confirm(t('removeCollectionConfirm'))) return
  await run(async () => {
    await api(collectionPath(home.value, selected.value?.collection.id), libraryRevisionSchema, {
      method: 'DELETE',
      data: { expected_revision: revision.value },
    })
    dirty.value = false
    await router.replace(listPath.value)
  })
}
function paginate(delta: number) {
  void router.push({ query: { ...route.query, page: page.value + delta } })
}
watch(search, () => {
  clearTimeout(searchTimer)
  if (search.value === (route.query.q ?? '')) return
  searchTimer = setTimeout(() => {
    if (!id.value) void router.replace({ query: { q: search.value || undefined } })
  }, 250)
})
watch(
  () => route.fullPath,
  () => {
    if (!id.value) search.value = typeof route.query.q === 'string' ? route.query.q : ''
    void load()
  },
  { immediate: true },
)
</script>
<template>
  <NcNoteCard v-if="error" type="error" role="alert"
    >{{ errorMessage(error) }}
    <NcButton :disabled="busy" @click="reload">{{ t('reloadSaved') }}</NcButton></NcNoteCard
  >
  <p v-if="saved" role="status">{{ t('collectionSaved') }}</p>
  <NcLoadingIcon v-if="busy && id && !selected && !draft" :aria-label="t('loading')" />
  <template v-if="!id">
    <header class="page-header">
      <div>
        <h1 tabindex="-1">{{ t('collections') }}</h1>
        <p>{{ t('collectionIntro') }}</p>
      </div>
      <NcButton v-if="writable" variant="primary" :to="`${listPath}/new`"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('newCollection') }}</NcButton
      >
    </header>
    <div class="toolbar">
      <LabeledField
        v-model="search"
        type="search"
        :label="t('searchCollections')"
        :show-trailing-button="Boolean(search)"
        trailing-button-icon="close"
        :trailing-button-label="t('clearSearch')"
        @trailing-button-click="search = ''"
      /><span v-if="records" role="status">{{ t('collections') }}: {{ records.total }}</span
      ><NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
    </div>
    <NcEmptyContent
      v-if="records && !records.total"
      :name="search ? t('noResults') : t('noCollections')"
      :description="search ? '' : t('noCollectionsHelp')"
    />
    <ul class="record-list">
      <li v-for="record in records?.collections" :key="record.id">
        <RouterLink :to="`${listPath}/${record.id}`"
          ><div>
            <strong>{{ record.name }}</strong
            ><span v-if="record.description">{{ record.description }}</span>
          </div>
          <span>{{ t('recipes') }}: {{ record.count }}</span></RouterLink
        >
      </li>
    </ul>
  </template>
  <template v-else-if="editing && draft">
    <NcButton :disabled="busy" :to="selected ? `${listPath}/${id}` : listPath">{{
      t('cancel')
    }}</NcButton>
    <header class="page-header">
      <h1 tabindex="-1">{{ selected ? t('editCollection') : t('newCollection') }}</h1>
      <NcButton
        variant="primary"
        type="submit"
        form="collection-form"
        :disabled="busy || !writable"
        >{{ busy ? t('saving') : t('save') }}</NcButton
      >
    </header>
    <NcNoteCard v-if="validation" type="error" role="alert">{{ validation }}</NcNoteCard>
    <form id="collection-form" @submit.prevent="save">
      <CollectionEditor v-model="draft" :recipes="recipes" :busy="busy" @changed="markChanged" />
    </form>
  </template>
  <template v-else-if="selected">
    <NcButton :to="listPath">{{ t('back') }}</NcButton>
    <header class="page-header">
      <div>
        <h1 tabindex="-1">{{ selected.collection.name }}</h1>
        <p v-if="selected.collection.description" class="collection-description">
          {{ selected.collection.description }}
        </p>
      </div>
      <div class="actions library-header-actions">
        <ShareControls
          v-if="current?.role === 'owner'"
          :home="home"
          :title="selected.collection.name"
          :resource="{ kind: 'collection', collection_id: selected.collection.id }"
        /><NcButton v-if="selected.editable" :to="{ query: { edit: '1' } }">{{
          t('editCollection')
        }}</NcButton
        ><NcButton v-if="selected.editable" :disabled="busy" @click="remove"
          ><template #icon><Icon :path="mdiTrashCanOutline" /></template
          >{{ t('removeCollection') }}</NcButton
        >
      </div>
    </header>
    <NcNoteCard v-if="writable && !selected.editable" type="info">{{
      t('collectionOwnerOnly')
    }}</NcNoteCard>
    <p class="muted">{{ t('collectionRevisionHelp') }}</p>
    <NcEmptyContent v-if="!selected.collection.entries.length" :name="t('emptyCollection')" />
    <ul class="record-list">
      <li v-for="entry in selected.items" :key="entry.recipe_id">
        <RouterLink
          :to="{
            path: `/h/${home}/recipes/${entry.recipe_id}`,
            query: {
              revision: entry.revision,
              collection: selected.collection.id,
              collectionPage: page,
            },
          }"
          ><img
            v-if="entry.image_id"
            :src="imageUrl(home, entry.image_id)"
            alt=""
            width="64"
            height="64"
            class="recipe-thumbnail"
            loading="lazy"
          />
          <div>
            <strong :lang="entry.language">{{ entry.title }}</strong
            ><span>{{ servingCount(entry.servings) }}</span>
          </div>
          <span>{{ t('revision') }} {{ entry.revision }}</span></RouterLink
        >
      </li>
    </ul>
  </template>
  <nav
    v-if="!editing && (records?.total ?? selected?.collection.entries.length ?? 0) > 20"
    class="actions"
    :aria-label="t('libraryPage')"
  >
    <NcButton :disabled="busy || page <= 1" @click="paginate(-1)">{{
      t('libraryPreviousPage')
    }}</NcButton
    ><span>{{ t('libraryPage') }} {{ page }}</span
    ><NcButton
      :disabled="busy || page * 20 >= (records?.total ?? selected?.collection.entries.length ?? 0)"
      @click="paginate(1)"
      >{{ t('libraryNextPage') }}</NcButton
    >
  </nav>
</template>
