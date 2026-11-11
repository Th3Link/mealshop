<script setup lang="ts">
import { computed, onBeforeUnmount, ref, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { api } from './api'
import { sharedListSchema, sharedRoute, sharedImageUrl } from './library-api'
import { loadHomes, useTask } from './state'
import { t, servingCount } from './i18n'
import SharedInvitation from './SharedInvitation.vue'
const route = useRoute(),
  router = useRouter()
const page = computed(() => Math.max(1, Number(route.query.page) || 1))
const data = ref<z.infer<typeof sharedListSchema>>()
const { busy, error, run, errorMessage } = useTask()
let generation = 0
async function load() {
  const token = ++generation
  await run(async () => {
    await loadHomes()
    const result = await api(`/api/shared?page=${page.value}`, sharedListSchema)
    if (token === generation) data.value = result
  })
  if (error.value && token === generation) data.value = undefined
}
function visible() {
  if (document.visibilityState === 'visible') void load()
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
    void load()
  },
  { immediate: true },
)
function paginate(delta: number) {
  void router.push({ query: { page: page.value + delta } })
}
</script>
<template>
  <header class="page-header">
    <div>
      <h1 tabindex="-1">{{ t('sharedWithMe') }}</h1>
      <p>{{ t('sharedIntro') }}</p>
    </div>
    <NcButton :disabled="busy" @click="load">{{ t('refreshAccess') }}</NcButton>
  </header>
  <NcNoteCard v-if="error" type="error" role="alert">{{ errorMessage(error) }}</NcNoteCard>
  <NcLoadingIcon v-if="busy && !data" :aria-label="t('loading')" />
  <NcEmptyContent
    v-if="data && !data.total"
    :name="t('noSharedContent')"
    :description="t('noSharedContentHelp')"
  />
  <div v-for="item in data?.items" :key="`${item.share.source_home}/${item.share.id}`">
    <SharedInvitation v-if="item.share.pending" :share="item.share" @changed="load" />
    <ul v-else-if="item.content" class="record-list">
      <li>
        <RouterLink :to="sharedRoute(item.share.source_home, item.share.id)"
          ><img
            v-if="item.content.kind === 'recipe' && item.content.has_image"
            :src="
              sharedImageUrl(
                item.share.source_home,
                item.share.id,
                undefined,
                item.content.revision,
              )
            "
            alt=""
            class="recipe-thumbnail"
            width="64"
            height="64"
            loading="lazy"
          />
          <div>
            <strong :lang="item.content.kind === 'recipe' ? item.content.language : undefined">{{
              item.content.kind === 'recipe'
                ? item.content.title
                : item.content.kind === 'collection'
                  ? item.content.name
                  : t('originalRecipe')
            }}</strong
            ><span
              >{{ t('sharingFrom') }} {{ item.share.owner
              }}<template v-if="item.share.recipient.kind === 'household'">
                · {{ item.share.recipient_label }}</template
              ></span
            ><span v-if="item.content.kind === 'source_link'" class="source-url">{{
              item.content.source_url
            }}</span
            ><span v-if="item.content.kind === 'recipe'"
              >{{ servingCount(item.content.servings) }} · {{ t('revision') }}
              {{ item.content.revision }}</span
            ><span v-if="item.content.kind === 'collection'"
              >{{ t('recipes') }}: {{ item.content.count }}</span
            >
          </div></RouterLink
        >
      </li>
    </ul>
  </div>
  <nav v-if="data && data.total > data.per_page" class="actions" :aria-label="t('libraryPage')">
    <NcButton :disabled="busy || page <= 1" @click="paginate(-1)">{{
      t('libraryPreviousPage')
    }}</NcButton
    ><span>{{ t('libraryPage') }} {{ page }}</span
    ><NcButton :disabled="busy || page * data.per_page >= data.total" @click="paginate(1)">{{
      t('libraryNextPage')
    }}</NcButton>
  </nav>
</template>
