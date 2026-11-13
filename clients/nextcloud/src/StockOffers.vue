<script setup lang="ts">
import { computed, ref } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import LabeledField from './LabeledField.vue'
import { t, compactExact, shoppingUnit, calendarDate } from './i18n'
import { stockName, stockToday } from './inventory-format'
import { foldSearch } from './ingredient-search'
import type { Inventory, InventoryCommand } from './inventory-api'
import type { Shopping } from './shopping-api'
const props = defineProps<{
  offers: Inventory['offers']
  shops: Shopping['shops']['entries']
  members: string[]
  writable: boolean
  busy: boolean
}>()
const emit = defineEmits<{ action: [command: InventoryCommand] }>()
const date = ref(stockToday()),
  query = ref(''),
  shop = ref('')
const matching = computed(() =>
  props.offers.filter(
    (o) =>
      !o.offer.archived &&
      (!shop.value || o.offer.shop_id === shop.value) &&
      foldSearch(
        [
          o.offer.binding.name,
          ...Object.values(o.offer.binding.translations),
          o.offer.shop_name,
        ].join(' '),
      ).includes(foldSearch(query.value)),
  ),
)
function edit(offer: Inventory['offers'][number]['offer']) {
  emit('action', {
    operation: 'save_offer',
    offer: {
      id: offer.id,
      shop_id: offer.shop_id,
      target: offer.binding.product
        ? { kind: 'product', barcode: offer.binding.product.barcode }
        : { kind: 'ingredient', ingredient_id: offer.binding.ingredient_id ?? '' },
      quantity: { ...offer.quantity },
      price: offer.price,
      currency: offer.currency,
      valid_from: offer.valid_from,
      valid_until: offer.valid_until,
      source: offer.source,
      source_url: offer.source_url,
    },
  })
}
</script>
<template>
  <p>{{ t('stockOffersHelp') }}</p>
  <div class="form-row">
    <LabeledField v-model="query" :label="t('stockOfferSearch')" type="search" /><label
      class="field"
      >{{ t('stockShop')
      }}<select v-model="shop">
        <option value="">{{ t('shopAll') }}</option>
        <option v-for="entry in shops" :key="entry.id" :value="entry.id">{{ entry.name }}</option>
      </select></label
    ><LabeledField v-model="date" :label="t('stockShoppingDate')" type="date" />
  </div>
  <NcEmptyContent
    v-if="!matching.length"
    :name="offers.length ? t('stockNoMatches') : t('stockOffersEmpty')"
  />
  <ul class="stock-offers">
    <li v-for="item in matching" :key="item.offer.id">
      <div class="stock-lot-name">
        <strong>{{ stockName(item.offer.binding) }}</strong
        ><span
          >{{ item.offer.shop_name }} · {{ compactExact(item.offer.quantity.amount) }}
          {{ shoppingUnit(item.offer.quantity.unit, item.offer.quantity.amount) }}</span
        ><span
          >{{ calendarDate(item.offer.valid_from) }} –
          {{ calendarDate(item.offer.valid_until) }}</span
        ><span
          >{{ item.offer.source }}
          <a
            v-if="item.offer.source_url"
            :href="item.offer.source_url"
            target="_blank"
            rel="noopener noreferrer"
            >{{ t('stockSource') }}</a
          ></span
        ><span :class="{ 'stock-date-passed': item.safety.status === 'blocked' }">{{
          t(item.safety.status === 'blocked' ? 'stockOfferBlocked' : 'stockOfferReview')
        }}</span
        ><span v-if="date < item.offer.valid_from || date > item.offer.valid_until">{{
          t('stockOfferInactive')
        }}</span>
      </div>
      <strong class="stock-offer-price"
        >{{ compactExact(item.offer.price) }} {{ item.offer.currency }}</strong
      >
      <div v-if="writable" class="actions">
        <NcButton
          :disabled="
            busy ||
            !item.shop_available ||
            !date ||
            date < item.offer.valid_from ||
            date > item.offer.valid_until
          "
          @click="
            emit('action', {
              operation: 'add_offer',
              offer_id: item.offer.id,
              packages: 1,
              as_of: date,
              member_ids: [...members],
            })
          "
          >{{ t('stockOfferAdd') }}</NcButton
        ><NcButton variant="tertiary" :disabled="busy" @click="edit(item.offer)">{{
          t('stockOfferEdit')
        }}</NcButton
        ><NcButton
          variant="tertiary"
          :disabled="busy"
          @click="emit('action', { operation: 'archive_offer', offer_id: item.offer.id })"
          >{{ t('stockOfferArchive') }}</NcButton
        >
      </div>
    </li>
  </ul>
</template>
