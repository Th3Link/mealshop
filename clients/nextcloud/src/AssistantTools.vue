<script setup lang="ts">
import { computed, onMounted, ref } from 'vue'
import { useRoute } from 'vue-router'
import { generateUrl } from '@nextcloud/router'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { mdiContentCopy } from '@mdi/js'
import Icon from './Icon.vue'
import LabeledField from './LabeledField.vue'
import { api, domain, ApiError } from './api'
import { loadHomes, useTask } from './state'
import { t, locale } from './i18n'
import { useUnsavedChanges } from './useUnsavedChanges'
import { mcpSettingsSchema, mcpActivitySchema } from './mcp-api'
import type { McpScope, McpSettings } from './mcp-api'
import type { z } from 'zod'
const route = useRoute(),
  home = computed(() => String(route.params.home))
const settings = ref<McpSettings>(),
  draft = ref<Record<string, McpScope>>({}),
  activity = ref<z.infer<typeof mcpActivitySchema>>()
const notice = ref(''),
  page = ref(1)
const { busy, error, run, errorMessage } = useTask()
const path = (part: string) => domain(home.value, `/api/household/mcp/${part}`)
const normalized = (value: Record<string, McpScope>) =>
  JSON.stringify(
    Object.fromEntries(
      Object.entries(value)
        .filter(([, scope]) => scope !== 'disabled')
        .sort(([a], [b]) => a.localeCompare(b)),
    ),
  )
const dirty = computed(
  () => !!settings.value && normalized(draft.value) !== normalized(settings.value.grants),
)
const discard = useUnsavedChanges(() => dirty.value)
function readDraft(value: McpSettings) {
  draft.value = {
    ...Object.fromEntries(Object.keys(value.members).map((user) => [user, 'disabled' as McpScope])),
    ...value.grants,
  }
}
const missing = computed(() =>
  Object.keys(draft.value).filter(
    (user) => !settings.value?.members[user] && draft.value[user] !== 'disabled',
  ),
)
const endpoint = computed(() =>
  new URL(
    generateUrl(`/apps/app_api/proxy/mealshop${domain(home.value, '/api/household/mcp')}`),
    window.location.origin,
  ).toString(),
)
const example = computed(() =>
  JSON.stringify(
    {
      MEALSHOP_MCP_URL: endpoint.value,
      MEALSHOP_MCP_USER: settings.value?.user ?? '',
      MEALSHOP_MCP_APP_PASSWORD: '<protected-environment-value>',
    },
    null,
    2,
  ),
)
const scopeLabel = {
  disabled: 'mcpScopeDisabled',
  read: 'mcpScopeRead',
  create: 'mcpScopeCreate',
} as const
function stamp(seconds: number) {
  return new Intl.DateTimeFormat(locale.value, { dateStyle: 'medium', timeStyle: 'short' }).format(
    new Date(seconds * 1000),
  )
}
function privateFailure(error: unknown) {
  if (error instanceof ApiError && (error.status === 401 || error.status === 403)) {
    settings.value = undefined
    draft.value = {}
    activity.value = undefined
  }
  throw error
}
async function load() {
  notice.value = ''
  settings.value = undefined
  activity.value = undefined
  await run(async () => {
    await loadHomes()
    const data = await Promise.all([
      api(path('settings'), mcpSettingsSchema),
      api(`${path('activity')}?page=${page.value}`, mcpActivitySchema),
    ])
    settings.value = data[0]
    readDraft(data[0])
    activity.value = data[1]
    page.value = data[1].page
  })
}
async function reload() {
  if (discard()) await load()
}
async function save() {
  if (!settings.value?.owner) return
  await run(async () => {
    try {
      settings.value = await api(path('settings'), mcpSettingsSchema, {
        method: 'PUT',
        data: { expected_revision: settings.value?.revision, grants: draft.value },
      })
      readDraft(settings.value)
      notice.value = t('mcpSaved')
    } catch (error) {
      privateFailure(error)
    }
  })
}
async function paginate(delta: number) {
  await run(async () => {
    try {
      const data = await api(`${path('activity')}?page=${page.value + delta}`, mcpActivitySchema)
      activity.value = data
      page.value = data.page
    } catch (error) {
      privateFailure(error)
    }
  })
}
async function copy() {
  try {
    await navigator.clipboard.writeText(endpoint.value)
    notice.value = t('mcpCopied')
  } catch {
    notice.value = t('mcpCopyFailed')
  }
}
onMounted(load)
</script>
<template>
  <header class="page-header">
    <div>
      <h1 tabindex="-1">{{ t('assistants') }}</h1>
      <p>{{ t('mcpIntro') }}</p>
    </div>
    <NcButton :disabled="busy" @click="reload">{{ t('reloadSaved') }}</NcButton>
  </header>
  <NcNoteCard v-if="error" type="error" role="alert">{{ errorMessage(error) }}</NcNoteCard>
  <p v-if="notice" role="status">{{ notice }}</p>
  <NcLoadingIcon v-if="busy && !settings" :aria-label="t('loading')" />
  <template v-if="settings">
    <NcNoteCard type="info">{{ t('mcpPrivacy') }}</NcNoteCard>
    <p class="mcp-current">
      <strong>{{ t('mcpYourScope') }}: {{ t(scopeLabel[settings.scope]) }}</strong>
    </p>
    <section v-if="settings.owner" class="mcp-section">
      <h2>{{ t('mcpGrants') }}</h2>
      <p class="muted">{{ t('mcpGrantHelp') }}</p>
      <form @submit.prevent="save">
        <fieldset class="mcp-grants" :disabled="busy">
          <div v-for="(role, user) in settings.members" :key="user">
            <label class="field"
              >{{ t('mcpAccessFor') }} · {{ user }} · {{ t(role) }}
              <select v-model="draft[user]">
                <option value="disabled">{{ t('mcpScopeDisabled') }}</option>
                <option value="read">{{ t('mcpScopeRead') }}</option>
                <option value="create" :disabled="role === 'reader'">
                  {{ t('mcpScopeCreate') }}
                </option>
              </select>
            </label>
            <p v-if="role === 'reader' && draft[user] === 'create'" class="muted">
              {{ t('mcpRoleLimited') }}
            </p>
          </div>
          <NcNoteCard v-if="missing.length" type="warning"
            >{{ t('mcpGrantMissing') }}
            <div v-for="user in missing" :key="user" class="actions">
              <span>{{ user }}</span
              ><NcButton type="button" @click="delete draft[user]">{{ t('remove') }}</NcButton>
            </div></NcNoteCard
          >
          <NcButton
            type="submit"
            variant="primary"
            :disabled="busy || !dirty || missing.length > 0"
            >{{ t('mcpSave') }}</NcButton
          >
        </fieldset>
      </form>
    </section>
    <p v-else class="muted">{{ t('mcpOwnerOnly') }}</p>
    <section class="mcp-section">
      <h2>{{ t('mcpConnect') }}</h2>
      <p v-if="settings.scope === 'disabled'" class="muted">{{ t('mcpDisabledHelp') }}</p>
      <div class="mcp-connection">
        <LabeledField :model-value="endpoint" :label="t('mcpEndpoint')" readonly /><NcButton
          @click="copy"
          ><template #icon><Icon :path="mdiContentCopy" /></template>{{ t('mcpCopy') }}</NcButton
        >
      </div>
      <p>{{ t('mcpSetup') }}</p>
      <h3>{{ t('mcpEnvironment') }}</h3>
      <pre class="mcp-example">{{ example }}</pre>
      <p class="muted">{{ t('mcpCredentialHelp') }}</p>
      <p>{{ t('mcpCreationHelp') }}</p>
    </section>
    <section class="mcp-section">
      <h2>{{ t('mcpActivity') }}</h2>
      <p class="muted">{{ t('mcpActivityHelp') }}</p>
      <NcEmptyContent v-if="activity && !activity.total" :name="t('mcpActivityEmpty')" />
      <ul class="record-list">
        <li v-for="entry in activity?.entries" :key="entry.recipe_id">
          <RouterLink
            :to="{
              path: `/h/${home}/recipes/${entry.recipe_id}`,
              query: { revision: entry.revision, assistant: '1' },
            }"
            ><div>
              <strong :lang="entry.language">{{ entry.title }}</strong
              ><span
                >{{ t('revision') }} {{ entry.revision }} · {{ t('mcpRecordedBy') }}
                {{ entry.actor }}</span
              >
            </div>
            <time :datetime="new Date(entry.recorded_at * 1000).toISOString()">{{
              stamp(entry.recorded_at)
            }}</time></RouterLink
          >
        </li>
      </ul>
      <nav
        v-if="activity && activity.total > activity.per_page"
        class="actions"
        :aria-label="t('libraryPage')"
      >
        <NcButton :disabled="busy || page <= 1" @click="paginate(-1)">{{
          t('libraryPreviousPage')
        }}</NcButton
        ><span>{{ t('libraryPage') }} {{ page }}</span
        ><NcButton
          :disabled="busy || page * activity.per_page >= activity.total"
          @click="paginate(1)"
          >{{ t('libraryNextPage') }}</NcButton
        >
      </nav>
    </section>
  </template>
</template>
