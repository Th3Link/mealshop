<script setup lang="ts">
import { computed, onMounted, ref, toRaw } from 'vue'
import { onBeforeRouteLeave, useRoute } from 'vue-router'
import { z } from 'zod'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextField from '@nextcloud/vue/components/NcTextField'
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
const route = useRoute()
const home = computed(() => String(route.params.home)),
  current = computed(() => homes.value.find((h) => h.id === home.value))
const writable = computed(() => current.value?.role !== 'reader')
const access = ref<z.infer<typeof accessSchema>>()
const profiles = ref<z.infer<typeof profilesSchema>>()
const catalogue = ref<z.infer<typeof catalogueSchema>>()
const groups = ref<{ id: string; name: string; translations: Record<string, string> }[]>([])
const draft = ref<Member[]>([]),
  dirty = ref(false)
const inviteUser = ref(''),
  inviteRole = ref<'editor' | 'reader'>('reader')
const { busy, error, run, errorMessage } = useTask()
const base = computed(() => `/api/households/${home.value}`)
async function load() {
  await run(async () => {
    const results = await Promise.all([
      api(base.value, accessSchema),
      api(domain(home.value, '/api/household/profiles'), profilesSchema),
      loadCatalogue(),
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
    disliked_ingredients: [],
  })
  dirty.value = true
}
function remove(index: number) {
  if (window.confirm(t('deleteProfile'))) {
    draft.value.splice(index, 1)
    dirty.value = true
  }
}
function notes(event: Event, member: Member, key: 'allergies' | 'intolerances') {
  if (event.target instanceof HTMLTextAreaElement) {
    member[key] = event.target.value
      .split('\n')
      .map((s) => s.trim())
      .filter(Boolean)
    dirty.value = true
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
onBeforeRouteLeave(() => !dirty.value || window.confirm(t('unsaved')))
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
    >{{ errorMessage(error) }}<NcButton @click="load">{{ t('retry') }}</NcButton></NcNoteCard
  ><NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
  <section v-if="profiles" class="narrow">
    <h2>{{ t('profiles') }}</h2>
    <NcEmptyContent v-if="!draft.length" :name="t('noProfiles')" />
    <form @submit.prevent="save" @input="dirty = true" @change="dirty = true">
      <fieldset v-for="(member, index) in draft" :key="member.id" :disabled="!writable || busy">
        <legend>{{ member.name || `${t('profiles')} ${index + 1}` }}</legend>
        <NcTextField v-model="member.name" :label="t('name')" required maxlength="200" />
        <fieldset>
          <legend>{{ t('allergenGroups') }}</legend>
          <NcCheckboxRadioSwitch
            v-for="group in groups"
            :key="group.id"
            v-model="member.allergen_ids"
            :value="group.id"
            >{{ group.translations[locale] ?? group.name }}</NcCheckboxRadioSwitch
          >
        </fieldset>
        <label class="field"
          >{{ t('allergies') }} · {{ t('onePerLine')
          }}<textarea
            :value="member.allergies.join('\n')"
            @change="notes($event, member, 'allergies')"
          /></label
        ><label class="field"
          >{{ t('intolerances') }} · {{ t('onePerLine')
          }}<textarea
            :value="member.intolerances.join('\n')"
            @change="notes($event, member, 'intolerances')"
          />
        </label>
        <IngredientPicker
          @update:model-value="dirty = true"
          v-model="member.excluded_ingredients"
          :ingredients="catalogue?.ingredients ?? []"
          :label="t('excluded')"
          :disabled="!writable || busy"
        />
        <IngredientPicker
          @update:model-value="dirty = true"
          v-model="member.disliked_ingredients"
          :ingredients="catalogue?.ingredients ?? []"
          :label="t('disliked')"
          :disabled="!writable || busy"
        /><NcButton v-if="writable" @click="remove(index)">{{ t('remove') }}</NcButton>
      </fieldset>
      <div class="actions">
        <NcButton v-if="writable" :disabled="busy || draft.length >= 100" @click="add">{{
          t('newMember')
        }}</NcButton
        ><NcButton v-if="writable" type="submit" variant="primary" :disabled="busy || !dirty">{{
          t('save')
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
        <NcTextField
          v-model="inviteUser"
          :label="t('userId')"
          :helper-text="t('inviteHelp')"
          required
          maxlength="255"
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
