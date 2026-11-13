<script setup lang="ts">
import { computed, watch } from 'vue'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import LabeledField from './LabeledField.vue'
import StockTarget from './StockTarget.vue'
import StockMetadata from './StockMetadata.vue'
import StockSelection from './StockSelection.vue'
import StockPrice from './StockPrice.vue'
import ShoppingQuantityInput from './ShoppingQuantityInput.vue'
import { compactExact, shoppingUnit, locale, t } from './i18n'
import { stockSourceName } from './inventory-format'
import type { Catalogue, Member } from './contracts'
import type { Shopping } from './shopping-api'
import type { Inventory, InventoryCommand } from './inventory-api'
const command = defineModel<InventoryCommand>({ required: true })
const props = defineProps<{
  data: Inventory
  shopping: Shopping
  catalogue: Catalogue
  members: Member[]
  home: string
  disabled: boolean
}>()
const emit = defineEmits<{ nested: [open: boolean]; run: [id: string] }>()
const selectedRun = computed(() =>
  command.value.operation === 'consume' ? command.value.run_id : undefined,
)
watch(
  selectedRun,
  (id) => {
    if (id) emit('run', id)
  },
  { immediate: true },
)
const sources = computed(() =>
  props.shopping.sources
    .filter((s) => s.active)
    .filter(
      (s) =>
        command.value.operation !== 'consume' ||
        (props.data.cooking_selection?.run_id === command.value.run_id &&
          props.data.cooking_selection.source_ids.includes(s.id)),
    )
    .map((s) => ({ id: s.id, label: stockSourceName(s.origin) })),
)
const purchases = computed(() =>
  props.data.purchases.map((p) => ({
    id: p.id,
    label: `${p.item.translations[locale.value] ?? p.item.name} · ${compactExact(p.amount)} ${shoppingUnit(p.item.unit, p.amount)} · ${stockSourceName(p.origin)}`,
  })),
)
const allocations = computed(() =>
  props.data.allocations
    .filter((a) => a.state === 'held')
    .map((a) => {
      const lot = props.data.lots.find((l) => l.id === a.lot_id)
      return {
        id: a.id,
        label: `${lot?.binding.translations[locale.value] ?? lot?.binding.name ?? a.lot_id} · ${compactExact(a.amount)} ${lot?.unit ?? ''} · ${stockSourceName(a.origin)}`,
      }
    }),
)
const members = computed(() => props.members.map((m) => ({ id: m.id, label: m.name })))
</script>
<template>
  <fieldset class="editor-fields stock-fields" :disabled="disabled">
    <template v-if="command.operation === 'create_lot'">
      <StockTarget
        v-model="command.target"
        :catalogue="catalogue"
        :products="shopping.products"
        :home="home"
        :disabled="disabled"
        allow-unmatched
        @nested="emit('nested', $event)"
      />
      <ShoppingQuantityInput v-model="command.quantity" :disabled="disabled" />
      <StockMetadata v-model="command.metadata" />
    </template>
    <template v-else-if="command.operation === 'update_lot'"
      ><StockMetadata v-model="command.metadata"
    /></template>
    <template v-else-if="command.operation === 'adjust_lot'">
      <ShoppingQuantityInput
        v-model="command.quantity"
        :disabled="disabled"
        allow-zero
        :label="t('stockObserved')"
      />
      <p class="muted">{{ t('stockZeroAllowed') }}</p>
      <LabeledField v-model="command.reason" :label="t('stockReason')" required maxlength="500" />
    </template>
    <template v-else-if="command.operation === 'reserve'">
      <p>{{ t('stockReserveHelp') }}</p>
      <LabeledField v-model="command.as_of" :label="t('stockUseDate')" type="date" required />
      <StockSelection
        v-model="command.source_ids"
        :label="t('stockSources')"
        :options="sources"
        :disabled="disabled"
      />
    </template>
    <template v-else-if="command.operation === 'release'"
      ><StockSelection
        v-model="command.allocation_ids"
        :label="t('stockReleaseTitle')"
        :options="allocations"
        :disabled="disabled"
    /></template>
    <template v-else-if="command.operation === 'receive'">
      <p>{{ t('stockReceiveHelp') }}</p>
      <StockSelection
        v-model="command.purchase_ids"
        :label="t('stockPurchases')"
        :options="purchases"
        :disabled="disabled"
      />
      <StockMetadata v-model="command.metadata" />
    </template>
    <template v-else-if="command.operation === 'consume'">
      <p>{{ t('stockConsumeHelp') }}</p>
      <label class="field"
        >{{ t('stockRun')
        }}<select
          :aria-label="t('stockRun')"
          v-model="command.run_id"
          required
          :disabled="disabled"
        >
          <option disabled value="">{{ t('stockRun') }}</option>
          <option
            v-for="run in data.completed_runs"
            :key="run.id"
            :value="run.id"
            :disabled="run.booked"
          >
            {{ run.name }} · {{ run.recipe.title }}
            {{ run.booked ? `· ${t('stockAlreadyBooked')}` : '' }}
          </option>
        </select></label
      >
      <LabeledField v-model="command.as_of" :label="t('stockUseDate')" type="date" required />
      <NcCheckboxRadioSwitch v-model="command.include_unreserved" :disabled="disabled">{{
        t('stockUnreserved')
      }}</NcCheckboxRadioSwitch>
      <p class="muted">{{ t('stockNoSourceNeeded') }}</p>
      <StockSelection
        v-model="command.source_ids"
        :label="t('stockMatchingSources')"
        :options="sources"
        :disabled="
          disabled || !data.cooking_selection || data.cooking_selection.run_id !== command.run_id
        "
      />
    </template>
    <template v-else-if="command.operation === 'reverse'"
      ><p>{{ t('stockReverseHelp') }}</p>
      <LabeledField v-model="command.reason" :label="t('stockReason')" required maxlength="500"
    /></template>
    <template v-else-if="command.operation === 'save_offer'">
      <label class="field"
        >{{ t('stockShop')
        }}<select :aria-label="t('stockShop')" v-model="command.offer.shop_id" required>
          <option disabled value="">{{ t('stockShop') }}</option>
          <option v-for="shop in shopping.shops.entries" :key="shop.id" :value="shop.id">
            {{ shop.name }}
          </option>
        </select></label
      >
      <RouterLink v-if="!shopping.shops.entries.length" :to="`/h/${home}/shopping`">{{
        t('stockManageShops')
      }}</RouterLink>
      <StockTarget
        v-model="command.offer.target"
        :catalogue="catalogue"
        :products="shopping.products"
        :home="home"
        :disabled="disabled"
        @nested="emit('nested', $event)"
      />
      <ShoppingQuantityInput
        v-model="command.offer.quantity"
        :disabled="disabled"
        :label="t('stockPackage')"
      />
      <div class="form-row">
        <StockPrice v-model="command.offer.price" :disabled="disabled" /><label class="field"
          >{{ t('stockCurrency')
          }}<select :aria-label="t('stockCurrency')" v-model="command.offer.currency">
            <option v-for="currency in ['EUR', 'GBP', 'USD', 'CHF']" :key="currency">
              {{ currency }}
            </option>
          </select></label
        >
      </div>
      <div class="form-row">
        <LabeledField
          v-model="command.offer.valid_from"
          :label="t('stockValidFrom')"
          type="date"
          required
        /><LabeledField
          v-model="command.offer.valid_until"
          :label="t('stockValidUntil')"
          type="date"
          required
          :min="command.offer.valid_from"
        />
      </div>
      <LabeledField
        v-model="command.offer.source"
        :label="t('stockSource')"
        required
        maxlength="1000"
      />
      <LabeledField
        :model-value="command.offer.source_url ?? ''"
        :label="t('stockSourceUrl')"
        type="url"
        maxlength="2048"
        @update:model-value="command.offer.source_url = String($event) || null"
      />
    </template>
    <template v-else-if="command.operation === 'add_offer'">
      <div class="form-row">
        <LabeledField
          v-model.number="command.packages"
          :label="t('stockPackages')"
          type="number"
          required
          min="1"
          max="1000000"
          step="1"
        /><LabeledField
          v-model="command.as_of"
          :label="t('stockShoppingDate')"
          type="date"
          required
        />
      </div>
      <NcNoteCard type="info">{{ t('stockDietaryHelp') }}</NcNoteCard>
      <StockSelection
        v-if="members.length"
        v-model="command.member_ids"
        :label="t('stockIntendedProfiles')"
        :options="members"
        :disabled="disabled"
      />
    </template>
  </fieldset>
</template>
