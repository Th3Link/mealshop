<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { api, ApiError } from './api'
import { emptySchema } from './contracts'
import { sharedPath, sharedRoute, sharedDetailSchema, sharedImageUrl } from './library-api'
import { currentUser, homes, loadHomes, useTask } from './state'
import { t } from './i18n'
import SharedRecipe from './SharedRecipe.vue'
const route = useRoute(),
  router = useRouter()
const home = computed(() => String(route.params.sourceHome)),
  id = computed(() => String(route.params.share))
const recipe = computed(() =>
  typeof route.params.recipe === 'string' ? route.params.recipe : undefined,
)
const path = computed(() => sharedPath(home.value, id.value, recipe.value))
const page = computed(() => Math.max(1, Number(route.query.page) || 1))
const data = ref<z.infer<typeof sharedDetailSchema>>(),
  servings = ref<string | number>(1),
  unit = ref<'g' | 'kg'>('g')
const { busy, error, run, errorMessage } = useTask()
const unavailable = computed(
  () => error.value instanceof ApiError && [403, 404].includes(error.value.status),
)
const canRemove = computed(() => {
  const recipient = data.value?.share.recipient
  return recipient?.kind === 'account'
    ? recipient.user === currentUser.value
    : recipient?.kind === 'household' &&
        homes.value.some((h) => h.id === recipient.home_id && h.role === 'owner')
})
const back = computed(() =>
  recipe.value
    ? { path: sharedRoute(home.value, id.value), query: { page: route.query.fromPage ?? '1' } }
    : '/shared',
)
const image = computed(() =>
  sharedImageUrl(
    home.value,
    id.value,
    recipe.value,
    data.value?.content.kind === 'recipe' ? data.value.content.record.revision : undefined,
  ),
)
let generation = 0
async function load(preview = false, focus = false) {
  const token = ++generation
  const query = new URLSearchParams({ page: String(page.value) })
  if (recipe.value && typeof route.query.revision === 'string')
    query.set('revision', route.query.revision)
  if (preview) {
    query.set('servings', String(servings.value))
    query.set('display_unit', unit.value)
  }
  await run(async () => {
    await loadHomes()
    const result = await api(`${path.value}?${query}`, sharedDetailSchema)
    if (token !== generation) return
    data.value = result
    if (!preview && result.content.kind === 'recipe')
      servings.value = result.content.record.recipe.servings
  })
  if (token === generation && error.value) data.value = undefined
  if (token === generation && focus) {
    await nextTick()
    document.querySelector<HTMLElement>('.mealshop h1')?.focus()
  }
}
async function remove() {
  if (!data.value || !canRemove.value || !window.confirm(t('sharingRemoveIncomingConfirm'))) return
  await run(async () => {
    await api(sharedPath(home.value, id.value), emptySchema, {
      method: 'DELETE',
      data: { expected_version: data.value?.share.version },
    })
    data.value = undefined
    await router.replace('/shared')
  })
}
function visible() {
  if (document.visibilityState === 'visible') void load(data.value?.content.kind === 'recipe')
}
document.addEventListener('visibilitychange', visible)
onBeforeUnmount(() => {
  generation++
  data.value = undefined
  document.removeEventListener('visibilitychange', visible)
})
watch(
  () => route.fullPath,
  () => {
    data.value = undefined
    unit.value = 'g'
    void load(false, true)
  },
  { immediate: true },
)
function paginate(delta: number) {
  void router.push({ query: { ...route.query, page: page.value + delta } })
}
</script>
<template>
  <div class="actions">
    <NcButton :to="back">{{ t('back') }}</NcButton
    ><NcButton :disabled="busy" @click="load(data?.content.kind === 'recipe')">{{
      t('refreshAccess')
    }}</NcButton
    ><NcButton v-if="canRemove" :disabled="busy" @click="remove">{{
      t('sharingRemoveIncoming')
    }}</NcButton>
  </div>
  <NcEmptyContent
    v-if="unavailable"
    :name="t('sharingUnavailable')"
    :description="t('sharingUnavailableHelp')"
  />
  <NcNoteCard v-else-if="error" type="error" role="alert">{{ errorMessage(error) }}</NcNoteCard>
  <NcLoadingIcon v-if="busy && !data" :aria-label="t('loading')" />
  <template v-if="data">
    <p class="context">
      {{
        data.share.owner === currentUser
          ? t('sharingPreview')
          : `${t('sharingFrom')} ${data.share.owner}`
      }}<template v-if="data.share.recipient.kind === 'household'">
        · {{ data.share.recipient_label }}</template
      >
    </p>
    <SharedRecipe
      v-if="data.content.kind === 'recipe'"
      v-model:servings="servings"
      v-model:unit="unit"
      :value="data.content"
      :image-url="image"
      :busy="busy"
      @preview="load(true)"
    />
    <template v-else-if="data.content.kind === 'source_link'"
      ><h1 tabindex="-1">{{ t('originalRecipe') }}</h1>
      <NcNoteCard type="info">{{ t('sourceOnlyShareHelp') }}</NcNoteCard>
      <p>{{ t('sourceOnlyContent') }}</p>
      <a
        class="source-url"
        :href="data.content.source_url"
        target="_blank"
        rel="noopener noreferrer nofollow"
        >{{ data.content.source_url }}</a
      ></template
    >
    <template v-else>
      <header class="page-header">
        <div>
          <h1 tabindex="-1">{{ data.content.name }}</h1>
          <p v-if="data.content.description" class="collection-description">
            {{ data.content.description }}
          </p>
        </div>
      </header>
      <p class="muted">{{ t('collectionRevisionHelp') }}</p>
      <NcEmptyContent v-if="!data.content.count" :name="t('emptyCollection')" />
      <ul class="record-list">
        <li v-for="entry in data.content.items" :key="entry.recipe_id">
          <RouterLink
            :to="{
              path: sharedRoute(home, id, entry.recipe_id),
              query: { revision: entry.revision, fromPage: page },
            }"
            ><img
              v-if="entry.content.kind === 'recipe' && entry.content.has_image"
              :src="sharedImageUrl(home, id, entry.recipe_id, entry.revision)"
              alt=""
              class="recipe-thumbnail"
              width="64"
              height="64"
              loading="lazy"
            />
            <div>
              <strong
                :lang="entry.content.kind === 'recipe' ? entry.content.language : undefined"
                >{{
                  entry.content.kind === 'recipe' ? entry.content.title : t('originalRecipe')
                }}</strong
              ><span v-if="entry.content.kind === 'source_link'" class="source-url">{{
                entry.content.source_url
              }}</span
              ><span v-else>{{ t('revision') }} {{ entry.revision }}</span>
            </div></RouterLink
          >
        </li>
      </ul>
      <nav
        v-if="data.content.count > data.content.per_page"
        class="actions"
        :aria-label="t('libraryPage')"
      >
        <NcButton :disabled="busy || page <= 1" @click="paginate(-1)">{{
          t('libraryPreviousPage')
        }}</NcButton
        ><span>{{ t('libraryPage') }} {{ page }}</span
        ><NcButton
          :disabled="busy || page * data.content.per_page >= data.content.count"
          @click="paginate(1)"
          >{{ t('libraryNextPage') }}</NcButton
        >
      </nav>
    </template>
  </template>
</template>
