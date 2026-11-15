<script setup lang="ts">
import { computed, onMounted, ref, toRaw } from 'vue'
import { useRoute } from 'vue-router'
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextField from './LabeledField.vue'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { api, domain } from './api'
import { accessSchema, catalogueSchema, emptySchema, profilesSchema } from './contracts'
import type { Member } from './contracts'
import { homes, loadHomes, useTask, loadCatalogue } from './state'
import { locale, t } from './i18n'
import IngredientPicker from './IngredientPicker.vue'
import intoleranceTriggers from '../../../data/intolerance-triggers.json'
import NextcloudPersonPicker from './NextcloudPersonPicker.vue'
import ProfileNotes from './ProfileNotes.vue'
import PersonRulesEditor from './PersonRulesEditor.vue'
import { useUnsavedChanges } from './useUnsavedChanges'
const route = useRoute()
const home = computed(() => String(route.params.home)),
  current = computed(() => homes.value.find((h) => h.id === home.value))
const writable = computed(() => current.value?.role === 'owner' || current.value?.role === 'editor')
const access = ref<z.infer<typeof accessSchema>>()
const profiles = ref<z.infer<typeof profilesSchema>>()
const catalogue = ref<z.infer<typeof catalogueSchema>>()
const groups = ref<{ id: string; name: string; translations: Record<string, string> }[]>([])
const draft = ref<Member[]>([]),
  dirty = ref(false)
const saved = ref(false)
const confirmDiscard = useUnsavedChanges(() => dirty.value)
async function reload() {
  if (confirmDiscard()) await load()
}
function reset() {
  if (!profiles.value || !confirmDiscard()) return
  draft.value = structuredClone(toRaw(profiles.value.members))
  dirty.value = false
  saved.value = false
}
function markChanged() {
  dirty.value = true
  saved.value = false
}
const inviteUser = ref(''),
  inviteRole = ref<'editor' | 'reader'>('reader')
const { busy, error, run, errorMessage } = useTask()
const base = computed(() => `/api/households/${home.value}`)
async function load() {
  await run(async () => {
    const results = await Promise.all([
      api(base.value, accessSchema),
      api(domain(home.value, '/api/household/profiles'), profilesSchema),
      loadCatalogue(home.value),
      api(
        domain(home.value, '/api/household/allergens'),
        z.object({
          groups: z.array(
            z.object({
              id: z.string(),
              name: z.string(),
              translations: z.record(z.string(), z.string()),
            }),
          ),
        }),
      ),
    ])
    access.value = results[0]
    profiles.value = results[1]
    catalogue.value = results[2]
    groups.value = results[3].groups
    draft.value = structuredClone(toRaw(results[1].members))
    dirty.value = false
    saved.value = false
  })
}
async function save() {
  if (!profiles.value) return
  await run(async () => {
    profiles.value = await api(domain(home.value, '/api/household/profiles'), profilesSchema, {
      method: 'PUT',
      data: { expected_revision: profiles.value?.revision, members: draft.value },
    })
    dirty.value = false
    saved.value = true
  })
}
async function refreshAccess() {
  await run(async () => {
    access.value = await api(base.value, accessSchema)
    await loadHomes()
  })
}
async function invite() {
  await run(async () => {
    await api(`${base.value}/invitations`, emptySchema, {
      method: 'POST',
      data: { user: inviteUser.value, role: inviteRole.value },
    })
    access.value = await api(base.value, accessSchema)
    inviteUser.value = ''
  })
}
async function change(user: string, role?: 'owner' | 'editor' | 'reader') {
  if (!window.confirm(role === 'owner' ? t('transfer') : t('confirm'))) return
  await run(async () => {
    await api(`${base.value}/members/${encodeURIComponent(user)}`, emptySchema, {
      method: role ? 'PUT' : 'DELETE',
      data: role ? { role } : undefined,
    })
    access.value = await api(base.value, accessSchema)
    await loadHomes()
  })
}
function add() {
  draft.value.push({
    id: `member-${crypto.randomUUID()}`,
    name: '',
    allergies: [],
    intolerances: [],
    allergen_ids: [],
    excluded_ingredients: [],
    excluded_tags: [],
    disliked_ingredients: [],
  })
  markChanged()
}
function remove(index: number) {
  if (window.confirm(t('deleteProfile'))) {
    draft.value.splice(index, 1)
    markChanged()
  }
}
function exportProfiles() {
  if (!profiles.value) return
  const url = URL.createObjectURL(
    new Blob([JSON.stringify(profiles.value, null, 2)], { type: 'application/json' }),
  )
  const a = document.createElement('a')
  a.href = url
  a.download = 'mealshop-dietary-profiles.json'
  a.click()
  URL.revokeObjectURL(url)
}
onMounted(load)
</script>
<template>
  <header class="page-header">
    <div>
      <h1 tabindex="-1">{{ t('household') }}</h1>
      <p>{{ t('profileIntro') }}</p>
    </div>
  </header>
  <NcNoteCard v-if="error" type="error" role="alert"
    >{{ errorMessage(error)
    }}<NcButton :disabled="busy" @click="reload">{{
      t(dirty ? 'reloadSaved' : 'retry')
    }}</NcButton></NcNoteCard
  ><NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
  <section v-if="profiles" class="narrow">
    <h2>{{ t('profiles') }}</h2>
    <NcEmptyContent v-if="!draft.length" :name="t('noProfiles')" />
    <p v-if="saved" role="status" class="save-status">{{ t('changesSaved') }}</p>
    <form @submit.prevent="save" @input="markChanged" @change="markChanged">
      <fieldset v-for="(member, index) in draft" :key="member.id" :disabled="!writable || busy">
        <legend>{{ member.name || `${t('profiles')} ${index + 1}` }}</legend>
        <NcTextField v-model="member.name" :label="t('name')" required maxlength="200" />
        <fieldset>
          <legend>{{ t('allergenGroups') }}</legend>
          <div class="allergen-options">
            <NcCheckboxRadioSwitch
              v-for="group in groups"
              :key="group.id"
              v-model="member.allergen_ids"
              :value="group.id"
              >{{ group.translations[locale] ?? group.name }}</NcCheckboxRadioSwitch
            >
          </div>
        </fieldset>
        <PersonRulesEditor
          v-if="catalogue"
          :member="member"
          :ingredients="catalogue.ingredients"
          @changed="markChanged"
        />
        <ProfileNotes :member="member" @changed="markChanged" />
        <fieldset>
          <legend>{{ t('intoleranceTriggers') }}</legend>
          <NcCheckboxRadioSwitch
            v-for="trigger in intoleranceTriggers"
            :key="trigger.id"
            v-model="member.intolerances"
            :value="trigger.id"
            >{{ t('sorbitol') }}</NcCheckboxRadioSwitch
          >
        </fieldset>
        <IngredientPicker
          @update:model-value="markChanged"
          v-model="member.excluded_ingredients"
          :ingredients="catalogue?.ingredients ?? []"
          :label="t('excluded')"
          :disabled="!writable || busy"
        />
        <IngredientPicker
          @update:model-value="markChanged"
          v-model="member.disliked_ingredients"
          :ingredients="catalogue?.ingredients ?? []"
          :label="t('disliked')"
          :disabled="!writable || busy"
        /><NcButton v-if="writable" @click="remove(index)">{{ t('remove') }}</NcButton>
      </fieldset>
      <div class="actions editor-actions">
        <NcButton v-if="writable" :disabled="busy || draft.length >= 100" @click="add">{{
          t('newMember')
        }}</NcButton
        ><NcButton v-if="writable" type="submit" variant="primary" :disabled="busy || !dirty">{{
          t(busy ? 'saving' : 'save')
        }}</NcButton
        ><NcButton v-if="writable && dirty" :disabled="busy" @click="reset">{{
          t('discardChanges')
        }}</NcButton
        ><NcButton :disabled="busy" @click="exportProfiles">{{ t('exportProfiles') }}</NcButton>
      </div>
    </form>
  </section>
  <section v-if="access">
    <div class="inline-row">
      <h2>{{ t('members') }}</h2>
      <NcButton :disabled="busy" @click="refreshAccess">{{ t('refreshAccess') }}</NcButton>
    </div>
    <p>{{ t('accessHelp') }}</p>
    <ul class="access-list">
      <li v-for="(role, user) in access.members" :key="user" class="inline-row">
        <strong>{{ user }}</strong
        ><span>{{ t(role) }}</span
        ><template v-if="current?.role === 'owner' && role !== 'owner'"
          ><NcButton
            :disabled="busy"
            @click="change(user, role === 'reader' ? 'editor' : 'reader')"
            >{{ t(role === 'reader' ? 'editor' : 'reader') }}</NcButton
          ><NcButton :disabled="busy" @click="change(user, 'owner')">{{ t('transfer') }}</NcButton
          ><NcButton :disabled="busy" @click="change(user)">{{ t('revoke') }}</NcButton></template
        >
      </li>
    </ul>
    <template v-if="current?.role === 'owner'"
      ><h3>{{ t('pending') }}</h3>
      <p v-if="!Object.keys(access.invitations ?? {}).length">{{ t('noInvitations') }}</p>
      <div v-for="(role, user) in access.invitations" :key="user" class="inline-row">
        <strong>{{ user }}</strong
        ><span>{{ t(role) }}</span
        ><NcButton :disabled="busy" @click="change(user)">{{ t('revoke') }}</NcButton>
      </div>
      <form class="narrow" @submit.prevent="invite">
        <h3>{{ t('invite') }}</h3>
        <NextcloudPersonPicker
          v-model="inviteUser"
          :excluded="[...Object.keys(access.members), ...Object.keys(access.invitations ?? {})]"
        /><label class="field"
          >{{ t('role')
          }}<select v-model="inviteRole">
            <option value="reader">{{ t('reader') }}</option>
            <option value="editor">{{ t('editor') }}</option>
          </select></label
        ><NcButton type="submit" :disabled="busy || !inviteUser" variant="primary">{{
          t('invite')
        }}</NcButton>
      </form></template
    >
  </section>
</template>
