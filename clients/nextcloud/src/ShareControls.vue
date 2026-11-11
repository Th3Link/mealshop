<script setup lang="ts">
import { computed, ref, watch } from 'vue'
import { z } from 'zod'
import { generateUrl } from '@nextcloud/router'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcDialog from '@nextcloud/vue/components/NcDialog'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { mdiShareVariantOutline } from '@mdi/js'
import Icon from './Icon.vue'
import NextcloudPersonPicker from './NextcloudPersonPicker.vue'
import ShareGrantList from './ShareGrantList.vue'
import { api } from './api'
import {
  grantPath,
  grantsSchema,
  librarySavedSchema,
  libraryRevisionSchema,
  sharedRoute,
} from './library-api'
import type { ShareResource, ShareRecipient } from './library-api'
import { homes, currentUser, loadHomes, useTask } from './state'
import { t } from './i18n'
const props = defineProps<{
  home: string
  resource: ShareResource
  title: string
  sourceOnly?: boolean
}>()
const open = ref(false),
  kind = ref<ShareRecipient['kind']>('account'),
  person = ref(''),
  destination = ref(''),
  notice = ref(''),
  page = ref(1)
const data = ref<z.infer<typeof grantsSchema>>(),
  returnFocus = ref<HTMLElement>()
const { busy, error, run, errorMessage } = useTask()
const owned = computed(() => homes.value.filter((h) => h.role === 'owner' && h.id !== props.home))
const valid = computed(() =>
  kind.value === 'household'
    ? owned.value.some((h) => h.id === destination.value)
    : Boolean(person.value),
)
const excluded = computed(() => [
  currentUser.value,
  ...(data.value?.shares.flatMap((item) =>
    item.grant.recipient.kind === kind.value && item.grant.recipient.kind !== 'household'
      ? [item.grant.recipient.user]
      : [],
  ) ?? []),
])
watch(kind, () => {
  person.value = ''
  destination.value = ''
})
async function fetchGrants() {
  const resourceId =
    props.resource.kind === 'recipe' ? props.resource.recipe_id : props.resource.collection_id
  const query = new URLSearchParams({
    kind: props.resource.kind,
    resource_id: resourceId,
    page: String(page.value),
  })
  data.value = await api(`${grantPath(props.home)}?${query}`, grantsSchema)
  const lastPage = Math.max(1, Math.ceil(data.value.total / data.value.per_page))
  if (page.value > lastPage) {
    page.value = lastPage
    await fetchGrants()
  }
}
async function show() {
  returnFocus.value =
    document.activeElement instanceof HTMLElement ? document.activeElement : undefined
  open.value = true
  page.value = 1
  notice.value = ''
  data.value = undefined
  await run(async () => {
    await loadHomes()
    await fetchGrants()
  })
}
function canClose() {
  return !busy.value && ((!person.value && !destination.value) || window.confirm(t('unsaved')))
}
function closed(value: boolean) {
  open.value = value
  if (!value) {
    data.value = undefined
    person.value = ''
    destination.value = ''
    notice.value = ''
  }
}
async function add() {
  if (!data.value || !valid.value) return
  const recipient: ShareRecipient =
    kind.value === 'household'
      ? { kind: 'household', home_id: destination.value }
      : { kind: kind.value, user: person.value }
  await run(async () => {
    await api(grantPath(props.home), librarySavedSchema, {
      method: 'POST',
      data: { expected_revision: data.value?.revision, resource: props.resource, recipient },
    })
    person.value = ''
    destination.value = ''
    notice.value = t('sharingGranted')
    await fetchGrants()
  })
}
async function revoke(id: string) {
  if (!data.value || !window.confirm(t('sharingRevokeConfirm'))) return
  await run(async () => {
    await api(grantPath(props.home, id), libraryRevisionSchema, {
      method: 'DELETE',
      data: { expected_revision: data.value?.revision },
    })
    notice.value = t('sharingRevoked')
    await fetchGrants()
  })
}
async function publish(id: string) {
  if (!data.value || !window.confirm(t('sharingPublishConfirm'))) return
  await run(async () => {
    await api(grantPath(props.home, id), libraryRevisionSchema, {
      method: 'PUT',
      data: { expected_revision: data.value?.revision, resource: props.resource },
    })
    notice.value = t('sharingRevisionPublished')
    await fetchGrants()
  })
}
async function copy(id: string) {
  try {
    await navigator.clipboard.writeText(
      new URL(
        generateUrl(`/apps/app_api/embedded/mealshop/main${sharedRoute(props.home, id)}`),
        location.origin,
      ).href,
    )
    notice.value = t('sharingLinkCopied')
  } catch {
    notice.value = t('sharingCopyFailed')
  }
}
async function paginate(delta: number) {
  page.value += delta
  await run(fetchGrants)
}
</script>
<template>
  <NcButton @click="show"
    ><template #icon><Icon :path="mdiShareVariantOutline" /></template>{{ t('share') }}</NcButton
  >
  <NcDialog
    v-if="open"
    :name="t('sharing')"
    content-classes="mealshop sharing-dialog"
    :set-return-focus="returnFocus"
    :no-close="busy || Boolean(person || destination)"
    :buttons="[{ label: t('sharingClose'), disabled: busy, callback: canClose }]"
    @update:open="closed"
  >
    <div class="editor-fields">
      <div>
        <strong>{{ props.title }}</strong>
        <p v-if="props.resource.kind === 'recipe'" class="muted">
          {{ t('revision') }} {{ props.resource.revision }}
        </p>
      </div>
      <NcNoteCard v-if="props.sourceOnly" type="info">{{ t('sourceOnlyShareHelp') }}</NcNoteCard>
      <NcNoteCard v-else-if="props.resource.kind === 'collection'" type="info">{{
        t('sharingCollectionSourceHelp')
      }}</NcNoteCard>
      <NcNoteCard v-if="error" type="error" role="alert"
        >{{ errorMessage(error) }}
        <NcButton :disabled="busy" @click="run(fetchGrants)">{{
          t('reloadSaved')
        }}</NcButton></NcNoteCard
      >
      <p v-if="notice" role="status">{{ notice }}</p>
      <NcLoadingIcon v-if="busy && !data" :aria-label="t('loading')" />
      <section v-if="data" class="editor-section">
        <h3>
          {{ data.total ? t('sharingRecipients') : t('sharingPrivate')
          }}<template v-if="data.total"> · {{ data.total }}</template>
        </h3>
        <ShareGrantList
          :items="data.shares"
          :resource="props.resource"
          :busy="busy"
          @revoke="revoke"
          @publish="publish"
          @copy="copy"
        />
        <nav v-if="data.total > data.per_page" class="actions" :aria-label="t('libraryPage')">
          <NcButton :disabled="busy || page <= 1" @click="paginate(-1)">{{
            t('libraryPreviousPage')
          }}</NcButton
          ><span>{{ t('libraryPage') }} {{ page }}</span
          ><NcButton :disabled="busy || page * data.per_page >= data.total" @click="paginate(1)">{{
            t('libraryNextPage')
          }}</NcButton>
        </nav>
      </section>
      <form class="editor-fields" @submit.prevent="add">
        <fieldset class="editor-fields" :disabled="busy || !data">
          <label class="field"
            >{{ t('sharingRecipientKind')
            }}<select v-model="kind">
              <option value="account">{{ t('sharingAccount') }}</option>
              <option value="household_invitation">{{ t('sharingHouseholdInvitation') }}</option>
              <option v-if="owned.length" value="household">
                {{ t('sharingOwnedHousehold') }}
              </option>
            </select></label
          >
          <label v-if="kind === 'household'" class="field"
            >{{ t('sharingDestination')
            }}<select v-model="destination" required>
              <option value="" disabled>{{ t('sharingDestination') }}</option>
              <option v-for="home in owned" :key="home.id" :value="home.id">{{ home.name }}</option>
            </select></label
          >
          <NextcloudPersonPicker
            v-else
            v-model="person"
            :excluded="excluded"
            :disabled="busy || !data"
            :label="kind === 'account' ? t('sharingPerson') : t('sharingHouseholdOwner')"
          />
          <p v-if="kind === 'household_invitation'" class="muted">
            {{ t('sharingHouseholdHelp') }}
          </p>
          <NcButton type="submit" variant="primary" :disabled="busy || !data || !valid">{{
            busy ? t('saving') : t('sharingGrant')
          }}</NcButton>
        </fieldset>
      </form>
      <p class="muted">{{ t('sharingReadOnlyHelp') }}</p>
    </div>
  </NcDialog>
</template>
