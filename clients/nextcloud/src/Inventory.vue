<script setup lang="ts">
import { computed, nextTick, onBeforeUnmount, onMounted, ref, toRaw, watch } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcLoadingIcon from '@nextcloud/vue/components/NcLoadingIcon'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import Icon from './Icon.vue'
import { mdiPlus } from '@mdi/js'
import { api, domain, ApiError } from './api'
import { homes, loadHomes, loadCatalogue, useTask } from './state'
import { profilesSchema } from './contracts'
import type { Catalogue, Member } from './contracts'
import { shoppingSchema } from './shopping-api'
import type { Shopping } from './shopping-api'
import { inventorySchema, inventoryPreviewSchema, inventoryResultSchema } from './inventory-api'
import type {
  Inventory,
  InventoryCommand,
  InventoryPreview,
  InventoryRequest,
} from './inventory-api'
import { t, compactExact, shoppingUnit } from './i18n'
import { stockActionLabel, stockToday, stockName } from './inventory-format'
import { useUnsavedChanges } from './useUnsavedChanges'
import InventoryForm from './InventoryForm.vue'
import InventoryReview from './InventoryReview.vue'
import StockLots from './StockLots.vue'
import StockOffers from './StockOffers.vue'
import StockHistory from './StockHistory.vue'
const route = useRoute(),
  router = useRouter(),
  home = computed(() => String(route.params.home)),
  base = computed(() => `/h/${home.value}/inventory`)
const section = computed(() =>
  route.params.section === 'offers'
    ? 'offers'
    : route.params.section === 'history'
      ? 'history'
      : 'stock',
)
const writable = computed(() => homes.value.some((h) => h.id === home.value && h.role !== 'reader'))
const data = ref<Inventory>(),
  shopping = ref<Shopping>(),
  catalogue = ref<Catalogue>(),
  members = ref<Member[]>([])
const draft = ref<InventoryCommand>(),
  baseline = ref(''),
  nested = ref(false),
  reviewed = ref(false),
  message = ref<'stockSaved' | 'stockReplay'>()
const prepared = ref<{ request: InventoryRequest; preview: InventoryPreview }>(),
  submission = ref<InventoryRequest>()
const dirty = computed(() =>
  Boolean(
    draft.value &&
    (JSON.stringify(draft.value) !== baseline.value || prepared.value || submission.value),
  ),
)
const { busy, error, run, errorMessage } = useTask()
const disabled = computed(() => busy.value || !writable.value || Boolean(submission.value))
const confirmDiscard = useUnsavedChanges(() => dirty.value && !nested.value)
let generation = 0
onBeforeUnmount(() => {
  generation++
})
watch(
  draft,
  () => {
    if (!submission.value) {
      prepared.value = undefined
      reviewed.value = false
    }
  },
  { deep: true, flush: 'sync' },
)
const lotContext = computed(() =>
  draft.value && 'lot_id' in draft.value
    ? data.value?.lots.find(
        (l) => draft.value && 'lot_id' in draft.value && l.id === draft.value.lot_id,
      )
    : undefined,
)
function metadata() {
  return { location: t('stockPantry'), batch: '', expiry: null }
}
function open(command: InventoryCommand) {
  if (busy.value || !writable.value || !confirmDiscard()) return
  draft.value = structuredClone(toRaw(command))
  baseline.value = JSON.stringify(draft.value)
  prepared.value = undefined
  submission.value = undefined
  message.value = undefined
  void nextTick(() => document.getElementById('stock-editor-title')?.focus())
}
function newLot() {
  open({
    operation: 'create_lot',
    target: { kind: 'ingredient', ingredient_id: '' },
    quantity: { amount: '1', unit: 'g' },
    metadata: metadata(),
  })
}
function reserve() {
  open({ operation: 'reserve', source_ids: [], as_of: stockToday() })
}
function receive() {
  open({ operation: 'receive', purchase_ids: [], metadata: metadata() })
}
function consume(run_id = '') {
  open({
    operation: 'consume',
    run_id,
    source_ids: [],
    include_unreserved: true,
    as_of: stockToday(),
  })
}
function newOffer() {
  open({
    operation: 'save_offer',
    offer: {
      id: null,
      shop_id: shopping.value?.shops.entries[0]?.id ?? '',
      target: { kind: 'ingredient', ingredient_id: '' },
      quantity: { amount: '1', unit: 'g' },
      price: '',
      currency: 'EUR',
      valid_from: stockToday(),
      valid_until: stockToday(),
      source: '',
      source_url: null,
    },
  })
}
async function load(keep = false, runId?: string) {
  const current = ++generation,
    household = home.value,
    selected = runId
  await run(async () => {
    const params = new URLSearchParams({ limit: '500', history_limit: '50' })
    const offset = typeof route.query.history_offset === 'string' ? route.query.history_offset : '0'
    if (/^\d{1,6}$/.test(offset)) params.set('history_offset', offset)
    if (selected) params.set('run_id', selected)
    const [stock, list, foods, profiles] = await Promise.all([
      api(domain(household, `/api/household/inventory?${params}`), inventorySchema),
      api(domain(household, '/api/household/shopping'), shoppingSchema),
      loadCatalogue(household),
      api(domain(household, '/api/household/profiles'), profilesSchema),
      loadHomes(),
    ])
    if (current !== generation) return
    data.value = stock
    shopping.value = list
    catalogue.value = foods
    members.value = profiles.members
    if (!keep) {
      draft.value = undefined
      prepared.value = undefined
      submission.value = undefined
      baseline.value = ''
    }
  })
  if (current === generation && !keep && writable.value && !error.value) {
    switch (route.query.task) {
      case 'reserve':
        reserve()
        break
      case 'receive':
        receive()
        break
      case 'consume':
        consume(typeof route.query.run === 'string' ? route.query.run : '')
        break
    }
  }
}
async function selectRun(id: string) {
  if (
    !draft.value ||
    draft.value.operation !== 'consume' ||
    data.value?.cooking_selection?.run_id === id
  )
    return
  draft.value.source_ids = []
  await load(true, id)
  if (
    draft.value?.operation === 'consume' &&
    draft.value.run_id === id &&
    data.value?.cooking_selection?.run_id === id
  ) {
    const exact =
      shopping.value?.sources.filter(
        (s) =>
          data.value?.cooking_selection?.source_ids.includes(s.id) &&
          (s.origin.kind === 'recipe' || s.origin.kind === 'meal'),
      ) ?? []
    if (exact.length === 1 && exact[0]) draft.value.source_ids = [exact[0].id]
  }
}
async function cancel() {
  if (!confirmDiscard()) return
  draft.value = undefined
  prepared.value = undefined
  submission.value = undefined
  baseline.value = ''
  if (route.query.task) await router.replace({ path: route.path })
  await nextTick()
  document.querySelector<HTMLElement>('.stock-page h1')?.focus()
}
async function preview() {
  if (!draft.value || !data.value || disabled.value) return
  const current = generation,
    request: InventoryRequest = {
      expected_revision: data.value.revision,
      idempotency_key: crypto.randomUUID(),
      command: structuredClone(toRaw(draft.value)),
      reviewed: false,
    }
  await run(async () => {
    const result = await api(
      domain(home.value, '/api/household/inventory/preview'),
      inventoryPreviewSchema,
      { method: 'POST', data: request },
    )
    if (current !== generation) return
    prepared.value = { request, preview: result }
    reviewed.value = false
    await nextTick()
    document.getElementById('stock-review-title')?.focus()
  })
}
async function confirm() {
  if (!submission.value) {
    if (!prepared.value) return
    submission.value = { ...prepared.value.request, reviewed: true }
  }
  const request = submission.value,
    current = generation,
    path = route.fullPath,
    destination = route.path,
    household = home.value
  await run(async () => {
    try {
      const result = await api(
        domain(home.value, '/api/household/inventory'),
        inventoryResultSchema,
        { method: 'POST', data: request },
      )
      if (current !== generation || route.fullPath !== path) return
      data.value = result.inventory
      draft.value = undefined
      prepared.value = undefined
      submission.value = undefined
      baseline.value = ''
      if (route.query.task || route.query.history_offset)
        await router.replace({ path: destination })
      if (home.value !== household || route.path !== destination) return
      message.value = result.replayed ? 'stockReplay' : 'stockSaved'
      const refreshGeneration = generation
      const refreshed = await api(domain(household, '/api/household/shopping'), shoppingSchema)
      if (refreshGeneration === generation && home.value === household) shopping.value = refreshed
    } catch (error) {
      if (current !== generation) return
      if (error instanceof ApiError && [401, 403, 409, 422].includes(error.status)) {
        submission.value = undefined
        prepared.value = undefined
        reviewed.value = false
      }
      throw error
    }
  })
}
const canPreview = computed(() => {
  const command = draft.value
  if (!command) return false
  switch (command.operation) {
    case 'reserve':
      return command.source_ids.length > 0
    case 'release':
      return command.allocation_ids.length > 0
    case 'receive':
      return command.purchase_ids.length > 0
    case 'consume':
      return Boolean(
        command.run_id &&
        data.value?.cooking_selection?.run_id === command.run_id &&
        !data.value.completed_runs.find((r) => r.id === command.run_id)?.booked,
      )
    case 'create_lot':
      return Boolean(
        command.quantity.amount &&
        (command.target.kind === 'ingredient'
          ? command.target.ingredient_id
          : command.target.kind === 'product'
            ? command.target.barcode
            : command.target.label),
      )
    case 'save_offer':
      return Boolean(
        command.offer.price &&
        command.offer.quantity.amount &&
        (command.offer.target.kind === 'ingredient'
          ? command.offer.target.ingredient_id
          : command.offer.target.kind === 'product'
            ? command.offer.target.barcode
            : command.offer.target.label),
      )
    default:
      return true
  }
})
watch(
  () => route.fullPath,
  () => {
    void load()
  },
)
onMounted(() => {
  void load()
})
</script>
<template>
  <div class="stock-page" :aria-busy="busy">
    <header class="page-header">
      <div>
        <h1 tabindex="-1">
          {{
            t(
              section === 'offers'
                ? 'stockOffers'
                : section === 'history'
                  ? 'stockHistory'
                  : 'inventory',
            )
          }}
        </h1>
        <p v-if="section === 'stock'">{{ t('stockIntro') }}</p>
      </div>
      <NcButton
        v-if="writable && !draft && section !== 'history'"
        variant="primary"
        :disabled="busy"
        @click="section === 'offers' ? newOffer() : newLot()"
        ><template #icon><Icon :path="mdiPlus" /></template
        >{{ t(section === 'offers' ? 'stockOfferNew' : 'stockNew') }}</NcButton
      >
    </header>
    <nav class="actions stock-tabs" :aria-label="t('inventory')">
      <NcButton
        :to="base"
        :variant="section === 'stock' ? 'primary' : 'tertiary'"
        :aria-current="section === 'stock' ? 'page' : undefined"
        >{{ t('inventory') }}</NcButton
      ><NcButton
        :to="`${base}/offers`"
        :variant="section === 'offers' ? 'primary' : 'tertiary'"
        :aria-current="section === 'offers' ? 'page' : undefined"
        >{{ t('stockOffers') }}</NcButton
      ><NcButton
        :to="`${base}/history`"
        :variant="section === 'history' ? 'primary' : 'tertiary'"
        :aria-current="section === 'history' ? 'page' : undefined"
        >{{ t('stockHistory') }}</NcButton
      >
    </nav>
    <NcNoteCard v-if="error" type="error" role="alert" tabindex="-1"
      >{{ errorMessage(error)
      }}<NcButton :disabled="busy || Boolean(submission)" @click="load(true)">{{
        t('stockReload')
      }}</NcButton></NcNoteCard
    >
    <p v-if="message" role="status">
      {{ t(message) }} <RouterLink :to="`${base}/history`">{{ t('stockViewHistory') }}</RouterLink>
    </p>
    <NcLoadingIcon v-if="busy" :aria-label="t('loading')" />
    <template v-if="data && shopping && catalogue">
      <section v-if="draft" class="stock-editor narrow">
        <header class="page-header">
          <h2 id="stock-editor-title" tabindex="-1">{{ stockActionLabel(draft) }}</h2>
          <NcButton :disabled="busy" @click="cancel">{{ t('cancel') }}</NcButton>
        </header>
        <p v-if="lotContext">
          <strong>{{ stockName(lotContext.binding) }}</strong> · {{ t('stockOriginal') }}:
          {{ compactExact(lotContext.original_quantity.amount) }}
          {{ shoppingUnit(lotContext.original_quantity.unit, lotContext.original_quantity.amount) }}
        </p>
        <NcNoteCard v-if="submission" type="warning">{{ t('stockFrozen') }}</NcNoteCard>
        <form @submit.prevent="preview">
          <InventoryForm
            v-model="draft"
            :data="data"
            :shopping="shopping"
            :catalogue="catalogue"
            :members="members"
            :home="home"
            :disabled="disabled"
            @nested="nested = $event"
            @run="selectRun"
          /><NcButton
            v-if="!prepared && !submission"
            type="submit"
            variant="primary"
            :disabled="disabled || !canPreview"
            >{{ t('stockPreview') }}</NcButton
          >
        </form>
        <InventoryReview
          v-if="prepared"
          :preview="prepared.preview"
          :data="data"
          :command="draft"
        />
        <NcCheckboxRadioSwitch
          v-if="prepared && draft.operation === 'add_offer'"
          v-model="reviewed"
          :disabled="disabled"
          >{{ t('stockReviewed') }}</NcCheckboxRadioSwitch
        >
        <div v-if="prepared || submission" class="actions">
          <NcButton
            variant="primary"
            :disabled="
              busy || !writable || Boolean(prepared && draft.operation === 'add_offer' && !reviewed)
            "
            @click="confirm"
            >{{ t(submission ? 'stockRetry' : 'stockConfirm') }}</NcButton
          ><NcButton
            v-if="prepared && !submission"
            :disabled="busy"
            @click="prepared = undefined"
            >{{ t('stockBackToEdit') }}</NcButton
          >
        </div>
      </section>
      <template v-else-if="section === 'stock'"
        ><div v-if="writable" class="actions stock-workflows">
          <NcButton :disabled="busy || !shopping.sources.some((s) => s.active)" @click="reserve">{{
            t('stockReserve')
          }}</NcButton
          ><NcButton :disabled="busy || !data.purchases.length" @click="receive">{{
            t('stockReceive')
          }}</NcButton
          ><NcButton
            :disabled="busy || !data.completed_runs.some((r) => !r.booked)"
            @click="consume()"
            >{{ t('stockConsume') }}</NcButton
          >
        </div>
        <StockLots :lots="data.lots" :writable="writable" :busy="busy" @action="open"
      /></template>
      <StockOffers
        v-else-if="section === 'offers'"
        :offers="data.offers"
        :shops="shopping.shops.entries"
        :members="members.map((m) => m.id)"
        :writable="writable"
        :busy="busy"
        @action="open"
      />
      <StockHistory
        v-else
        :data="data"
        :home="home"
        :writable="writable"
        :busy="busy"
        @action="open"
        @page="router.push({ path: route.path, query: { history_offset: String($event) } })"
      />
    </template>
  </div>
</template>
<style>
.stock-page > .page-header > div {
  flex: 1;
  min-width: 0;
}
.stock-page > .page-header > .button-vue {
  flex-shrink: 0;
}
.stock-tabs,
.stock-workflows {
  margin-block: 12px 20px;
}
.stock-editor {
  margin-block: 20px;
}
.mealshop .stock-fields .form-row {
  align-items: start;
}
.stock-fields,
.stock-selection {
  min-width: 0;
  display: grid;
  gap: 12px;
}
.stock-selection legend {
  font-weight: 600;
  margin-block: 12px;
}
.stock-lots,
.stock-offers,
.stock-history,
.stock-change-list {
  list-style: none;
  padding: 0;
  margin: 0;
}
.stock-lots > li,
.stock-offers > li {
  display: flex;
  align-items: center;
  gap: 20px;
  padding-block: 16px;
  border-bottom: 1px solid var(--color-border);
  min-width: 0;
}
.stock-lot-name {
  display: grid;
  gap: 3px;
  min-width: 0;
  flex: 1;
  overflow-wrap: anywhere;
}
.stock-lot-name > strong {
  font-size: 16px;
}
.stock-lot-name > span {
  color: var(--color-text-maxcontrast);
}
.stock-lot-name > .stock-date-passed {
  color: var(--color-warning-text, var(--color-main-text));
}
.mealshop .stock-lot-amounts {
  display: flex;
  gap: 20px;
  margin: 0;
  flex-wrap: wrap;
}
.mealshop .stock-lot-amounts dt {
  font-size: 13px;
  color: var(--color-text-maxcontrast);
}
.mealshop .stock-lot-amounts dd {
  margin: 2px 0 0;
  font-weight: 600;
  font-variant-numeric: tabular-nums;
  white-space: nowrap;
}
.stock-offer-price {
  font-size: 18px;
  white-space: nowrap;
  font-variant-numeric: tabular-nums;
}
.stock-offers > li > .actions {
  max-width: 320px;
}
.stock-history > li,
.stock-change-list > li,
.stock-offer-evidence {
  padding-block: 16px;
  border-bottom: 1px solid var(--color-border);
  overflow-wrap: anywhere;
}
.stock-history > li > header,
.stock-change-list > li {
  display: grid;
  gap: 6px;
}
.stock-comparison {
  display: grid;
  grid-template-columns: 1fr 1fr;
  gap: 16px;
}
.stock-comparison > div {
  display: grid;
  gap: 4px;
}
.stock-comparison > div > span {
  color: var(--color-text-maxcontrast);
}
.stock-review {
  margin-block: 24px;
}
.stock-review h3 {
  margin-top: 16px;
}
.stock-review ul {
  padding-inline-start: 0;
}
.stock-history ul {
  padding-inline-start: 20px;
}
.stock-editor > .actions {
  margin-block: 16px;
}
.stock-offers a {
  overflow-wrap: anywhere;
}
.stock-covered {
  margin-block: 20px;
}
@media (max-width: 900px) {
  .stock-lots > li,
  .stock-offers > li {
    align-items: start;
    flex-wrap: wrap;
  }
  .stock-lot-name {
    flex-basis: 100%;
  }
  .mealshop .stock-lot-amounts {
    flex: 1;
  }
  .stock-offers > li > .actions {
    max-width: none;
  }
}
@media (prefers-reduced-motion: reduce) {
  .stock-page {
    scroll-behavior: auto;
  }
}
</style>
