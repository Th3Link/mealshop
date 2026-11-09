<script setup lang="ts">
import { onMounted, ref } from 'vue'
import { useRouter } from 'vue-router'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextField from '@nextcloud/vue/components/NcTextField'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { z } from 'zod'
import { api } from './api'
import { emptySchema, homesSchema } from './contracts'
import { loadHomes, useTask } from './state'
import { t } from './i18n'
const router = useRouter()
const data = ref<z.infer<typeof homesSchema>>()
const name = ref('')
const { busy, error, run, errorMessage } = useTask()
async function load() {
  await run(async () => {
    data.value = await loadHomes()
  })
}
async function create() {
  if (!name.value.trim()) return
  await run(async () => {
    const home = await api('/api/households', z.object({ id: z.string() }), {
      method: 'POST',
      data: { name: name.value },
    })
    await loadHomes()
    await router.push(`/h/${home.id}/recipes`)
  })
}
async function answer(id: string, accept: boolean) {
  await run(async () => {
    await api(`/api/households/${id}/invitation`, emptySchema, { method: 'PUT', data: { accept } })
    data.value = await loadHomes()
  })
}
async function leave(id: string) {
  if (!data.value || !window.confirm(t('leaveHousehold'))) return
  const user = data.value.user
  await run(async () => {
    await api(`/api/households/${id}/members/${encodeURIComponent(user)}`, emptySchema, {
      method: 'DELETE',
    })
    data.value = await loadHomes()
  })
}
onMounted(load)
</script>
<template>
  <header class="page-header">
    <div>
      <h1 tabindex="-1">{{ t('households') }}</h1>
      <p>{{ t('householdsIntro') }}</p>
    </div>
  </header>
  <NcNoteCard v-if="error" type="error" role="alert"
    >{{ errorMessage(error) }} <NcButton @click="load">{{ t('retry') }}</NcButton></NcNoteCard
  >
  <NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
  <template v-if="data">
    <NcEmptyContent
      v-if="!data.households.length && !data.invitations.length"
      :name="t('noHouseholds')"
      :description="t('noHouseholdsText')"
    />
    <ul v-if="data.households.length" class="record-list">
      <li v-for="home in data.households" :key="home.id">
        <RouterLink :to="`/h/${home.id}/recipes`"
          ><strong>{{ home.name }}</strong
          ><span>{{ t(home.role) }}</span></RouterLink
        ><NcButton v-if="home.role !== 'owner'" :disabled="busy" @click="leave(home.id)">{{
          t('leaveHousehold')
        }}</NcButton>
      </li>
    </ul>
    <section v-if="data.invitations.length">
      <h2>{{ t('invitations') }}</h2>
      <NcNoteCard type="info">{{ t('invitationNotice') }}</NcNoteCard>
      <div v-for="invite in data.invitations" :key="invite.id" class="inline-row">
        <strong>{{ invite.name }}</strong
        ><span>{{ t(invite.role) }}</span
        ><NcButton :disabled="busy" variant="primary" @click="answer(invite.id, true)">{{
          t('accept')
        }}</NcButton
        ><NcButton :disabled="busy" @click="answer(invite.id, false)">{{ t('decline') }}</NcButton>
      </div>
    </section>
  </template>
  <section class="narrow">
    <h2>{{ t('createHousehold') }}</h2>
    <form @submit.prevent="create">
      <NcTextField
        v-model="name"
        :label="t('householdName')"
        required
        maxlength="200"
        :disabled="busy"
      /><NcButton type="submit" variant="primary" :disabled="busy || !name.trim()">{{
        t('createHousehold')
      }}</NcButton>
    </form>
  </section>
</template>
