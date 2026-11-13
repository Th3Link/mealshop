<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, shallowRef, toRaw, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import { mdiPlus } from '@mdi/js'
import Icon from './Icon.vue'
import { api, domain, ApiError, errorMessage } from './api'
import { homes, loadCatalogue } from './state'
import type { Ingredient } from './contracts'
import { useUnsavedChanges } from './useUnsavedChanges'
import { t, compactExact } from './i18n'
import { localDate } from './cooking-format'
import type { CookingCommand, LocalTime } from './cooking-api'
import {
  brewStateSchema,
  brewDetailSchema,
  brewRecordSchema,
  brewPreviewSchema,
  brewResultSchema,
  templatesSchema,
  readingsPageSchema,
} from './brewing-api'
import type {
  BrewState,
  BrewDetail,
  BrewRecord,
  BrewTemplates,
  BrewPreview,
  BrewCommand,
  BrewRequest,
  BrewReading,
} from './brewing-api'
import BrewRecipeEditor from './BrewRecipeEditor.vue'
import BrewEquipmentEditor from './BrewEquipmentEditor.vue'
import BrewBatchEditor from './BrewBatchEditor.vue'
import BrewMeasurements from './BrewMeasurements.vue'
import BrewOverview from './BrewOverview.vue'
import BrewRecipeSummary from './BrewRecipeSummary.vue'
import BrewReview from './BrewReview.vue'
import CookingSteps from './CookingSteps.vue'
import CookingNotifications from './CookingNotifications.vue'
import CookingLocalTime from './CookingLocalTime.vue'
import { localInput } from './cooking-format'
const route = useRoute(),
  router = useRouter()
const home = computed(() => String(route.params.home)),
  section = computed(() =>
    route.params.section === 'recipes'
      ? 'recipes'
      : route.params.section === 'equipment'
        ? 'equipment'
        : 'batches',
  ),
  id = computed(() => String(route.params.id ?? '')),
  base = computed(() => `/h/${home.value}/brewing`),
  creating = computed(() => id.value === 'new'),
  editing = computed(() => creating.value || route.query.edit === '1')
const writable = computed(() => homes.value.some((h) => h.id === home.value && h.role !== 'reader'))
const data = ref<BrewState>(),
  detail = ref<BrewDetail>(),
  record = ref<BrewRecord>(),
  catalogue = ref<BrewTemplates>(),
  ingredients = ref<Ingredient[]>([])
const busy = ref(false),
  loaded = ref(false),
  offline = ref(false),
  dirty = ref(false),
  error = shallowRef<unknown>(),
  readError = shallowRef<unknown>(),
  announcement = ref(''),
  showArchived = ref(false),
  editorEpoch = ref(0)
const review = ref<BrewPreview>(),
  request = ref<BrewRequest>(),
  reviewed = ref(false),
  uncertain = ref(false),
  reading = ref(false),
  correcting = ref<BrewReading>(),
  earlier = ref<BrewReading[]>([])
const replan = ref<{ stepId: string; kind: 'delay' | 'timer'; at: LocalTime }>()
function accessFailure(value: unknown) {
  return value instanceof ApiError && [401, 403, 404].includes(value.status)
}
const accessError = computed(() => accessFailure(readError.value))
const disabled = computed(
  () =>
    busy.value ||
    offline.value ||
    !writable.value ||
    data.value?.scheduler.writable === false ||
    uncertain.value,
)
const selectedEquipment = computed(() => data.value?.equipment.find((e) => e.id === id.value))
const equipmentNames = computed(() =>
  Object.fromEntries(detail.value?.batch.equipment.map((e) => [e.id, e.details.name]) ?? []),
)
const measurements = computed(() =>
  [...(detail.value?.measurements.readings ?? []), ...earlier.value].filter(
    (r, i, all) => all.findIndex((other) => other.id === r.id) === i,
  ),
)
const nextCursor = ref<string | null>(null)
const confirmDiscard = useUnsavedChanges(() => dirty.value || uncertain.value)
let returnFocus: HTMLElement | undefined
let generation = 0,
  readGeneration = 0,
  polling = false,
  anchor = performance.now(),
  anchorSeconds = 0
const now = ref(0)
function clock(at: number) {
  anchor = performance.now()
  anchorSeconds = at
  now.value = at
}
const path = (suffix = '') => domain(home.value, `/api/household/brewing${suffix}`)
async function refresh(force = false) {
  if (
    (polling && !force) ||
    (!force && (busy.value || dirty.value || uncertain.value || editing.value))
  )
    return
  polling = true
  const epoch = generation,
    token = ++readGeneration,
    target = id.value,
    currentSection = section.value
  try {
    const state = await api(path(), brewStateSchema)
    if (epoch !== generation || token !== readGeneration) return
    let batch: BrewDetail | undefined, recipe: BrewRecord | undefined
    if (target && target !== 'new' && currentSection === 'batches')
      batch = await api(path(`/batches/${encodeURIComponent(target)}`), brewDetailSchema)
    if (target && target !== 'new' && currentSection === 'recipes')
      recipe = await api(
        path(
          `/recipes/${encodeURIComponent(target)}${typeof route.query.revision === 'string' ? `?revision=${encodeURIComponent(route.query.revision)}` : ''}`,
        ),
        brewRecordSchema,
      )
    if (
      epoch !== generation ||
      token !== readGeneration ||
      (!force && (dirty.value || busy.value || editing.value))
    )
      return
    data.value = state
    detail.value = batch
    record.value = recipe
    clock(state.server_now)
    offline.value = false
    readError.value = undefined
    loaded.value = true
    if (!earlier.value.length) nextCursor.value = batch?.measurements.next ?? null
  } catch (e) {
    if (epoch === generation && token === readGeneration) {
      readError.value = e
      offline.value = true
      loaded.value = true
    }
  } finally {
    if (token === readGeneration) polling = false
  }
}
async function load() {
  const epoch = ++generation
  ++readGeneration
  polling = false
  catalogue.value = undefined
  ingredients.value = []
  data.value = undefined
  detail.value = undefined
  record.value = undefined
  review.value = undefined
  request.value = undefined
  reading.value = false
  correcting.value = undefined
  replan.value = undefined
  dirty.value = false
  uncertain.value = false
  busy.value = false
  error.value = undefined
  earlier.value = []
  loaded.value = false
  editorEpoch.value++
  void refresh(true)
  try {
    const [c, food] = await Promise.all([
      api(path('/templates'), templatesSchema),
      loadCatalogue(home.value),
    ])
    if (epoch !== generation) return
    catalogue.value = c
    ingredients.value = food.ingredients
  } catch (e) {
    if (epoch === generation) {
      if (accessFailure(e)) {
        readError.value = e
        error.value = undefined
      } else error.value = e
      offline.value = true
    }
  }
}
async function preview(command: BrewCommand) {
  if (disabled.value || !data.value) return
  busy.value = true
  error.value = undefined
  const epoch = generation
  ++readGeneration
  returnFocus = document.activeElement instanceof HTMLElement ? document.activeElement : undefined
  const input: BrewRequest = {
    expected_revision: data.value.revision,
    idempotency_key: crypto.randomUUID(),
    command: structuredClone(toRaw(command)),
    reviewed: false,
  }
  try {
    const result = await api(path('/preview'), brewPreviewSchema, { method: 'POST', data: input })
    if (epoch !== generation) return
    request.value = input
    review.value = result
    reviewed.value = false
    dirty.value = true
    await nextTick()
    document.getElementById('brew-review-title')?.focus()
  } catch (e) {
    if (epoch === generation) error.value = e
  } finally {
    if (epoch === generation) busy.value = false
  }
}
async function commit(input: BrewRequest) {
  if (busy.value || !writable.value) return
  const epoch = generation
  busy.value = true
  error.value = undefined
  ++readGeneration
  request.value = structuredClone(toRaw(input))
  try {
    const result = await api(path(), brewResultSchema, { method: 'POST', data: input })
    if (epoch !== generation) return
    data.value = result.brewing
    clock(result.brewing.server_now)
    uncertain.value = false
    dirty.value = false
    review.value = undefined
    request.value = undefined
    reading.value = false
    correcting.value = undefined
    replan.value = undefined
    announcement.value = t('brewSaved')
    if (input.command.operation === 'save_recipe')
      await router.push(
        `${base.value}/recipes${result.result_id ? `/${encodeURIComponent(result.result_id)}` : ''}`,
      )
    else if (input.command.operation === 'save_equipment')
      await router.push(`${base.value}/equipment`)
    else if (input.command.operation === 'create_batch')
      await router.push(
        `${base.value}/batches${result.result_id ? `/${encodeURIComponent(result.result_id)}` : ''}`,
      )
    if (epoch === generation) await refresh(true)
  } catch (e) {
    if (epoch !== generation) return
    error.value = e
    uncertain.value = !(e instanceof ApiError) || e.status === 0 || e.status >= 500
    if (uncertain.value) dirty.value = true
    else {
      request.value = undefined
      review.value = undefined
    }
  } finally {
    if (epoch === generation) busy.value = false
  }
}
function retrySave() {
  if (request.value) void commit(structuredClone(toRaw(request.value)))
}

function confirmReview() {
  if (!request.value || !reviewed.value || disabled.value) return
  void commit({ ...structuredClone(toRaw(request.value)), reviewed: true })
}
function back() {
  if (uncertain.value) return
  review.value = undefined
  request.value = undefined
  reviewed.value = false
  void nextTick(() => returnFocus?.isConnected && returnFocus.focus())
}
function direct(command: BrewCommand) {
  if (disabled.value || !data.value || review.value) return
  void commit({
    expected_revision: data.value.revision,
    idempotency_key: crypto.randomUUID(),
    command,
    reviewed: true,
  })
}
function processCommand(command: CookingCommand) {
  switch (command.operation) {
    case 'timer':
    case 'subscribe':
    case 'acknowledge':
    case 'retry':
      direct(command)
      break
    case 'cancel':
      direct({ operation: 'cancel', run_id: command.run_id })
      break
    default:
      break
  }
}
function adjust(stepId: string, kind: 'delay' | 'timer') {
  if (!detail.value || disabled.value) return
  const step = detail.value.run.steps.find((s) => s.stage.id === stepId)
  const at =
    kind === 'timer' && step?.timer.state === 'running' ? step.timer.deadline : now.value + 3600
  replan.value = {
    stepId,
    kind,
    at: {
      local: localInput(at, detail.value.run.timezone),
      timezone: detail.value.run.timezone,
      fold: 'reject',
    },
  }
  dirty.value = true
}
function cancelBatch() {
  if (detail.value && window.confirm(t('cookCancelConfirm')))
    void preview({ operation: 'cancel', run_id: detail.value.run.id })
}
function previewReplan() {
  if (!detail.value || !replan.value) return
  const r = replan.value
  void preview({
    operation: 'replan',
    run_id: detail.value.run.id,
    change:
      r.kind === 'delay'
        ? { kind: 'delay', step_id: r.stepId, start_after: r.at }
        : { kind: 'timer', step_id: r.stepId, deadline: r.at },
  })
}
async function latestKeepingDraft() {
  if (busy.value || uncertain.value) return
  if (!dirty.value) {
    await refresh(true)
    error.value = undefined
    return
  }
  const epoch = generation
  busy.value = true
  ++readGeneration
  try {
    const latest = await api(path(), brewStateSchema)
    if (epoch !== generation) return
    data.value = latest
    clock(latest.server_now)
    offline.value = false
    error.value = undefined
    review.value = undefined
    request.value = undefined
  } catch (e) {
    if (epoch === generation) error.value = e
  } finally {
    if (epoch === generation) busy.value = false
  }
}
async function reload() {
  if (!confirmDiscard()) return
  await load()
}
async function earlierReadings() {
  if (!nextCursor.value || !detail.value || busy.value || offline.value) return
  const epoch = generation
  busy.value = true
  try {
    const result = await api(
      path(
        `/batches/${encodeURIComponent(detail.value.run.id)}/readings?before=${encodeURIComponent(nextCursor.value)}`,
      ),
      readingsPageSchema,
    )
    if (epoch !== generation) return
    earlier.value.push(...result.readings)
    nextCursor.value = result.next
  } catch (e) {
    if (epoch === generation) error.value = e
  } finally {
    if (epoch === generation) busy.value = false
  }
}
const heading = computed(() =>
  editing.value
    ? t(
        section.value === 'recipes'
          ? creating.value
            ? 'brewNewRecipe'
            : 'brewEdit'
          : section.value === 'equipment'
            ? creating.value
              ? 'brewNewEquipment'
              : 'brewEditEquipment'
            : 'brewNewBatch',
      )
    : (record.value?.recipe.title ??
      detail.value?.run.name ??
      t(
        section.value === 'recipes'
          ? 'brewRecipes'
          : section.value === 'equipment'
            ? 'brewEquipment'
            : 'brewBatches',
      )),
)
let poll: ReturnType<typeof setInterval> | undefined,
  tick: ReturnType<typeof setInterval> | undefined
function visible() {
  if (document.visibilityState === 'visible') void refresh()
}
watch(
  () => route.fullPath,
  () => {
    void load()
  },
)
onMounted(() => {
  void load()
  poll = setInterval(() => {
    if (document.visibilityState === 'visible') void refresh()
  }, 10000)
  tick = setInterval(() => {
    now.value = anchorSeconds + Math.floor((performance.now() - anchor) / 1000)
  }, 1000)
  document.addEventListener('visibilitychange', visible)
})
onBeforeUnmount(() => {
  generation++
  readGeneration++
  clearInterval(poll)
  clearInterval(tick)
  document.removeEventListener('visibilitychange', visible)
})
</script>
<template>
  <section class="page brewing-page" aria-labelledby="mealshop-brewing-title">
    <div class="section-heading">
      <div>
        <h1 id="mealshop-brewing-title" tabindex="-1" :lang="record?.recipe.language">
          {{ heading }}
        </h1>
        <p v-if="!id" class="muted">{{ t('brewIntro') }}</p>
      </div>
      <NcButton v-if="!id && writable" :to="`${base}/${section}/new`" :disabled="disabled"
        ><template #icon><Icon :path="mdiPlus" /></template
        >{{
          t(
            section === 'recipes'
              ? 'brewNewRecipe'
              : section === 'equipment'
                ? 'brewNewEquipment'
                : 'brewNewBatch',
          )
        }}</NcButton
      ><NcButton v-if="id" :to="`${base}/${section}`">{{ t('back') }}</NcButton>
    </div>
    <nav class="brew-tabs" :aria-label="t('brewing')">
      <RouterLink
        v-for="tab in ['batches', 'recipes', 'equipment'] as const"
        :key="tab"
        :to="`${base}/${tab}`"
        :aria-current="section === tab ? 'page' : undefined"
        >{{
          t(
            (
              {
                batches: 'brewBatches',
                recipes: 'brewRecipes',
                equipment: 'brewEquipment',
              } as const
            )[tab],
          )
        }}</RouterLink
      >
    </nav>
    <p class="visually-hidden" role="status">{{ announcement }}</p>
    <NcNoteCard v-if="offline" :type="accessError ? 'error' : 'warning'"
      >{{ accessError ? errorMessage(readError) : t('cookOffline') }}
      <p v-if="readError && !accessError">{{ errorMessage(readError) }}</p></NcNoteCard
    ><NcNoteCard v-if="error" type="error"
      >{{ errorMessage(error)
      }}<NcButton v-if="!uncertain" :disabled="busy" @click="latestKeepingDraft">{{
        t('brewKeepInputs')
      }}</NcButton
      ><NcButton v-if="!uncertain && dirty" :disabled="busy" variant="tertiary" @click="reload">{{
        t('brewReload')
      }}</NcButton></NcNoteCard
    ><NcNoteCard v-if="uncertain" type="warning"
      >{{ t('brewRetryHelp')
      }}<NcButton :disabled="busy || !request || !writable" @click="retrySave()">{{
        t('brewRetry')
      }}</NcButton></NcNoteCard
    ><NcLoadingIcon v-if="!loaded" :aria-label="t('loading')" /><NcButton
      v-if="offline"
      :disabled="busy"
      @click="reload"
      >{{ t('brewReload') }}</NcButton
    >
    <BrewReview
      v-if="review && request"
      v-model:reviewed="reviewed"
      :preview="review"
      :command="request.command"
      :disabled="disabled"
      @apply="confirmReview"
      @back="back"
    />
    <div v-if="data" v-show="!review">
      <template v-if="editing && writable"
        ><BrewRecipeEditor
          v-if="section === 'recipes' && catalogue && (creating || record)"
          :key="`${editorEpoch}-${id}`"
          :catalogue="catalogue"
          :record="record"
          :ingredients="ingredients"
          :disabled="disabled"
          @dirty="dirty = $event"
          @review="preview" /><BrewEquipmentEditor
          v-else-if="section === 'equipment' && (creating || selectedEquipment)"
          :key="`${editorEpoch}-${id}`"
          :equipment="selectedEquipment"
          :disabled="disabled"
          @dirty="dirty = $event"
          @review="preview" /><BrewBatchEditor
          v-else-if="section === 'batches' && creating"
          :key="editorEpoch"
          :data="data"
          :base="base"
          :recipe-id="typeof route.query.recipe === 'string' ? route.query.recipe : undefined"
          :disabled="disabled"
          @dirty="dirty = $event"
          @review="preview"
      /></template>
      <template v-else-if="section === 'recipes' && record"
        ><div class="actions">
          <NcButton v-if="writable" :to="`${base}/recipes/${id}?edit=1`" :disabled="disabled">{{
            t('edit')
          }}</NcButton
          ><NcButton
            v-if="writable"
            :to="`${base}/batches/new?recipe=${id}`"
            :disabled="disabled"
            >{{ t('brewNewBatch') }}</NcButton
          >
        </div>
        <p class="muted">
          {{ t('revision') }} {{ record.revision }} · {{ record.actor }} ·
          {{ localDate(record.recorded_at, 'UTC') }}
        </p>
        <BrewRecipeSummary :recipe="record.recipe"
      /></template>
      <template v-else-if="detail"
        ><p class="muted">
          {{ t('brewPinned') }}:
          <RouterLink :to="`${base}/recipes/${detail.recipe.id}?revision=${detail.recipe.revision}`"
            >{{ detail.recipe.recipe.title }} · {{ detail.recipe.revision }}</RouterLink
          >
          · {{ compactExact(detail.recipe.recipe.volume.amount) }}
          {{ detail.recipe.recipe.volume.unit }}
        </p>
        <NcNoteCard type="info">{{ t('brewManualHelp') }}</NcNoteCard>
        <form v-if="replan" class="brew-editor" @submit.prevent="previewReplan">
          <h2>{{ t(replan.kind === 'delay' ? 'cookDelayStep' : 'cookChangeTimer') }}</h2>
          <CookingLocalTime
            zone-editable
            v-model="replan.at"
            :label="t(replan.kind === 'delay' ? 'cookPlannedStart' : 'cookChangeTimer')"
          />
          <div class="actions">
            <NcButton type="submit" :disabled="disabled">{{ t('brewReview') }}</NcButton
            ><NcButton
              :disabled="disabled"
              @click="
                () => {
                  replan = undefined
                  dirty = false
                }
              "
              >{{ t('cancel') }}</NcButton
            >
          </div>
        </form>
        <CookingSteps
          v-else
          :run="detail.run"
          :language="detail.recipe.recipe.language"
          :now="now"
          :disabled="disabled || reading"
          brewing
          :equipment-names="equipmentNames"
          @command="processCommand"
          @replan="adjust"
          @announce="announcement = $event"
        />
        <BrewMeasurements
          v-model:reading="reading"
          v-model:correcting="correcting"
          :detail="detail"
          :measurements="measurements"
          :writable="writable"
          :disabled="disabled"
          :busy="busy"
          :offline="offline"
          :replan="!!replan"
          :now="now"
          :next-cursor="nextCursor"
          @review="preview"
          @dirty="dirty = $event"
          @earlier="earlierReadings"
        />
        <CookingNotifications
          :detail="detail"
          :now="now"
          :disabled="disabled || reading || !!replan"
          :writable="writable"
          open
          @command="processCommand"
        />
        <div class="actions">
          <NcButton
            v-if="writable && detail.run.status === 'active'"
            :disabled="disabled || reading || !!replan"
            @click="cancelBatch()"
            >{{ t('cookCancelRun') }}</NcButton
          ><NcButton
            v-if="writable && detail.run.status !== 'active'"
            :disabled="disabled"
            @click="
              preview({
                operation: 'archive_batch',
                run_id: detail.run.id,
                archived: !detail.batch.archived,
              })
            "
            >{{ t(detail.batch.archived ? 'brewRestore' : 'brewArchive') }}</NcButton
          >
        </div></template
      >
      <BrewOverview
        v-else-if="!id"
        v-model:show-archived="showArchived"
        :data="data"
        :section="section"
        :base="base"
        :writable="writable"
        :disabled="disabled"
        :catalogue="catalogue"
        @review="preview"
      />
    </div>
  </section>
</template>
