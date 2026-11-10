<script setup lang="ts">
import { computed, onMounted, onBeforeUnmount, ref, watch, nextTick } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { mdiPlus, mdiChefHat } from '@mdi/js'
import Icon from './Icon.vue'
import CookingWizard from './CookingWizard.vue'
import CookingSteps from './CookingSteps.vue'
import CookingReplan from './CookingReplan.vue'
import CookingNotifications from './CookingNotifications.vue'
import { api, domain, errorMessage } from './api'
import { homes, useTask } from './state'
import {
  cookingListSchema,
  cookingDetailSchema,
  cookingMutationSchema,
  previewSchema,
} from './cooking-api'
import type {
  CookingList,
  CookingDetail,
  CookingCommand,
  CookingPreview,
  Proposal,
} from './cooking-api'
import { localDate } from './cooking-format'
import { t, exact, locale, servingCount } from './i18n'
import { useUnsavedChanges } from './useUnsavedChanges'
const route = useRoute(),
  router = useRouter(),
  home = computed(() => String(route.params.home)),
  id = computed(() => String(route.params.id ?? '')),
  creating = computed(() => id.value === 'new'),
  base = computed(() => `/h/${home.value}/cooking`)
const writable = computed(() => homes.value.some((h) => h.id === home.value && h.role !== 'reader'))
const list = ref<CookingList>(),
  detail = ref<CookingDetail>(),
  offline = ref(false),
  loaded = ref(false),
  dirty = ref(false),
  announcement = ref(''),
  replan = ref<{ kind: 'target' | 'delay' | 'timer'; stepId?: string }>()
const { busy, error, run, errorMessage: taskError } = useTask()
const disabled = computed(
  () =>
    busy.value || offline.value || !writable.value || detail.value?.scheduler.writable === false,
)
const readError = ref<unknown>(),
  now = ref(Math.floor(Date.now() / 1000))
let anchor = performance.now(),
  anchorSeconds = now.value,
  generation = 0,
  polling = false
const confirmDiscard = useUnsavedChanges(() => dirty.value)
function anchorTime(seconds: number) {
  anchor = performance.now()
  anchorSeconds = seconds
  now.value = seconds
}
async function refresh(force = false) {
  if (creating.value || (polling && !force)) return
  polling = true
  const request = ++generation,
    target = id.value
  try {
    if (target) {
      const result = await api(
        domain(home.value, `/api/household/cooking/${encodeURIComponent(target)}`),
        cookingDetailSchema,
      )
      if (request !== generation) return
      detail.value = result
      anchorTime(result.server_now)
    } else {
      const result = await api(domain(home.value, '/api/household/cooking'), cookingListSchema)
      if (request !== generation) return
      list.value = result
      anchorTime(result.server_now)
    }
    offline.value = false
    readError.value = undefined
    loaded.value = true
  } catch (e) {
    if (request === generation) {
      readError.value = e
      offline.value = true
      loaded.value = true
    }
  } finally {
    if (request === generation) polling = false
  }
}
async function command(value: CookingCommand) {
  if (disabled.value) return false
  const revision = detail.value?.revision ?? list.value?.revision
  if (revision === undefined) return false
  const result = await run(async () => {
    ++generation
    polling = false
    const result = await api(domain(home.value, '/api/household/cooking'), cookingMutationSchema, {
      method: 'POST',
      data: { expected_revision: revision, command: value },
    })
    if (value.operation === 'remove') {
      detail.value = undefined
      dirty.value = false
      await router.push(base.value)
    }
    await refresh(true)
    announcement.value = t('changesSaved')
    return result
  })
  if (!result) await refresh(true)
  return !!result
}
async function preview(proposal: Proposal) {
  return run(async () => {
    const revision = (await api(domain(home.value, '/api/household/cooking'), cookingListSchema))
      .revision
    return api(domain(home.value, '/api/household/cooking/preview'), previewSchema, {
      method: 'POST',
      data: { expected_revision: revision, proposal },
    })
  })
}
async function apply(value: CookingPreview) {
  if (disabled.value) return false
  const result = await run(async () => {
    ++generation
    polling = false
    return api(domain(home.value, '/api/household/cooking'), cookingMutationSchema, {
      method: 'POST',
      data: {
        expected_revision: value.revision,
        command: { operation: 'apply', preview_id: value.preview_id, reviewed: true },
      },
    })
  })
  if (!result) {
    await refresh(true)
    return false
  }
  dirty.value = false
  replan.value = undefined
  if (result.run_id && id.value !== result.run_id)
    await router.push(`${base.value}/${result.run_id}`)
  else await refresh(true)
  announcement.value = t('changesSaved')
  await nextTick()
  document.getElementById('cooking-title')?.focus()
  return true
}
function openReplan(stepId: string, kind: 'delay' | 'timer') {
  if (confirmDiscard()) {
    dirty.value = false
    replan.value = { kind, stepId }
    void nextTick(() => document.querySelector<HTMLElement>('.cooking-replan h2')?.focus())
  }
}
function closeReplan() {
  if (confirmDiscard()) {
    dirty.value = false
    replan.value = undefined
  }
}
async function cancelRun() {
  if (detail.value && window.confirm(t('cookCancelConfirm')))
    await command({ operation: 'cancel', run_id: detail.value.run.id })
}
async function removeRun() {
  if (detail.value && window.confirm(t('cookRemoveConfirm')))
    await command({ operation: 'remove', run_id: detail.value.run.id })
}
watch(id, () => {
  polling = false
  ++generation
  detail.value = undefined
  list.value = undefined
  replan.value = undefined
  dirty.value = false
  loaded.value = false
  offline.value = false
  readError.value = undefined
  void refresh()
})
let clock: ReturnType<typeof setInterval>, poll: ReturnType<typeof setInterval>
function visible() {
  if (!document.hidden && !busy.value) void refresh()
}
onMounted(() => {
  void refresh()
  clock = setInterval(() => {
    now.value = anchorSeconds + Math.floor((performance.now() - anchor) / 1000)
  }, 1000)
  poll = setInterval(visible, 5000)
  document.addEventListener('visibilitychange', visible)
  window.addEventListener('online', visible)
})
onBeforeUnmount(() => {
  ++generation
  clearInterval(clock)
  clearInterval(poll)
  document.removeEventListener('visibilitychange', visible)
  window.removeEventListener('online', visible)
})
const stateKeys = {
  active: 'cookRunActive',
  completed: 'cookCompleted',
  cancelled: 'cookCancelled',
} as const
</script>
<template>
  <section class="cooking-view" aria-labelledby="cooking-title">
    <header class="page-header">
      <div>
        <RouterLink v-if="id" :to="base">{{ t('cookBack') }}</RouterLink>
        <h1 id="cooking-title" tabindex="-1">
          {{ creating ? t('cookNew') : (detail?.run.name ?? t('cookingSessions')) }}
        </h1>
        <p v-if="!id" class="muted">{{ t('cookOverviewHelp') }}</p>
      </div>
      <NcButton v-if="!creating && writable" variant="primary" :to="`${base}/new`"
        ><template #icon><Icon :path="mdiPlus" /></template>{{ t('cookNew') }}</NcButton
      >
    </header>
    <p class="visually-hidden" aria-live="polite">{{ announcement }}</p>
    <NcNoteCard v-if="error" type="error" role="alert">{{ taskError(error) }}</NcNoteCard
    ><NcNoteCard v-if="readError" type="error" role="alert"
      >{{ errorMessage(readError) }}
      <NcButton :disabled="busy" @click="refresh(true)">{{ t('retry') }}</NcButton></NcNoteCard
    ><NcNoteCard v-if="offline && detail" type="warning">{{ t('cookOffline') }}</NcNoteCard>
    <CookingWizard
      v-if="creating && writable"
      :home="home"
      :disabled="disabled"
      :preview="preview"
      :apply="apply"
      @dirty="dirty = $event"
    />
    <NcLoadingIcon v-else-if="!loaded" :name="t('loading')" />
    <template v-else-if="detail"
      ><p class="context">
        {{ t(stateKeys[detail.run.status]) }} · {{ detail.run.recipe.title }} · {{ t('revision') }}
        {{ detail.run.recipe.revision }} · {{ servingCount(detail.run.servings) }}
      </p>
      <p class="muted">
        {{ detail.run.timezone
        }}<template v-if="detail.run.target">
          · {{ t('cookTarget') }}: {{ detail.run.target.local }}</template
        >
      </p>
      <NcNoteCard v-if="!detail.scheduler.writable" type="error">{{ t('storage') }}</NcNoteCard>
      <CookingReplan
        v-if="replan"
        :key="`${replan.kind}:${replan.stepId}`"
        :run="detail.run"
        :step-id="replan.stepId"
        :kind="replan.kind"
        :disabled="disabled"
        :now="now"
        :preview="preview"
        :apply="apply"
        @dirty="dirty = $event"
        @close="closeReplan"
      />
      <template v-else
        ><details class="cooking-ingredients">
          <summary>{{ t('ingredients') }} · {{ servingCount(detail.run.servings) }}</summary>
          <ul class="plain-list">
            <li v-for="(line, index) in detail.ingredients" :key="index">
              <span>{{ line.translations[locale]?.name ?? line.name }}</span
              ><strong>{{ exact(line.grams) }} g</strong>
            </li>
          </ul>
          <p class="muted">{{ t('cookPinnedIngredients') }}</p>
        </details>
        <CookingSteps
          :run="detail.run"
          :now="now"
          :disabled="disabled"
          @command="command"
          @replan="openReplan"
          @announce="announcement = $event"
        /><CookingNotifications
          :detail="detail"
          :disabled="disabled"
          :writable="writable"
          :now="now"
          @command="command"
        />
        <div v-if="writable" class="actions cooking-management">
          <NcButton
            v-if="detail.run.status === 'active' && detail.run.target"
            :disabled="disabled"
            @click="replan = { kind: 'target' }"
            >{{ t('cookChangeTarget') }}</NcButton
          ><NcButton
            v-if="detail.run.status === 'active'"
            :disabled="disabled"
            variant="tertiary"
            @click="cancelRun"
            >{{ t('cookCancelRun') }}</NcButton
          ><NcButton v-else :disabled="disabled" variant="tertiary" @click="removeRun">{{
            t('cookRemoveRun')
          }}</NcButton>
        </div></template
      ></template
    >
    <template v-else-if="list"
      ><NcEmptyContent
        v-if="!list.runs.length"
        :name="t('cookEmpty')"
        :description="t('cookEmptyHelp')"
        ><template #icon><Icon :path="mdiChefHat" /></template
      ></NcEmptyContent>
      <ul v-else class="plain-list cooking-runs">
        <li v-for="item in list.runs" :key="item.id">
          <RouterLink :to="`${base}/${item.id}`"
            ><strong>{{ item.name }}</strong
            ><span class="muted">{{ item.recipe.title }} · {{ servingCount(item.servings) }}</span
            ><span
              >{{ t(stateKeys[item.status])
              }}<template v-if="item.running">
                · {{ item.running }} {{ t('cookRunningTimers') }}</template
              ></span
            ><strong v-if="item.overdue">{{ item.overdue }} {{ t('cookElapsed') }}</strong
            ><span v-if="item.next"
              >{{ item.next.name }} · {{ localDate(item.next.at, item.timezone) }}</span
            ></RouterLink
          >
        </li>
      </ul></template
    >
  </section>
</template>
