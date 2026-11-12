<script setup lang="ts">
import { computed } from 'vue'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import ShoppingOrigin from './ShoppingOrigin.vue'
import { compactExact, shoppingUnit, calendarDate, t } from './i18n'
import { stockName, stockMovementLabel } from './inventory-format'
import type { Inventory, InventoryCommand, InventoryPreview } from './inventory-api'
const props = defineProps<{
  preview: InventoryPreview
  data: Inventory
  command: InventoryCommand
}>()
const lots = computed(
  () =>
    new Map(
      [...props.data.lots, ...props.preview.changes.map((c) => c.after)].map((l) => [l.id, l]),
    ),
)
function lotLabel(id: string) {
  const lot = lots.value.get(id)
  return lot ? stockName(lot.binding) : t('unknown')
}
const selectedOffer = computed(() =>
  props.command.operation === 'add_offer'
    ? props.data.offers.find(
        (o) => props.command.operation === 'add_offer' && o.offer.id === props.command.offer_id,
      )?.offer
    : undefined,
)
const offers = computed(() => (selectedOffer.value ? [selectedOffer.value] : props.preview.offers))
</script>
<template>
  <section class="stock-review" aria-labelledby="stock-review-title">
    <h2 id="stock-review-title" tabindex="-1">{{ t('stockPreviewTitle') }}</h2>
    <ul v-if="preview.changes.length" class="stock-change-list">
      <li v-for="change in preview.changes" :key="change.after.id">
        <strong>{{ stockName(change.after.binding) }}</strong
        ><span class="muted"
          >{{ change.after.metadata.location
          }}<template v-if="change.after.metadata.batch">
            · {{ change.after.metadata.batch }}</template
          >
          ·
          {{
            change.after.metadata.expiry
              ? calendarDate(change.after.metadata.expiry)
              : t('stockDateUnknown')
          }}</span
        >
        <div class="stock-comparison">
          <div>
            <span>{{ t('stockBefore') }}</span
            ><strong
              >{{ change.before ? compactExact(change.before.remaining) : t('stockNewLot') }}
              {{
                change.before ? shoppingUnit(change.before.unit, change.before.remaining) : ''
              }}</strong
            ><span v-if="change.before"
              >{{ t('stockReserved') }}: {{ compactExact(change.before.reserved) }}</span
            >
          </div>
          <div>
            <span>{{ t('stockAfter') }}</span
            ><strong
              >{{ compactExact(change.after.remaining) }}
              {{ shoppingUnit(change.after.unit, change.after.remaining) }}</strong
            ><span>{{ t('stockReserved') }}: {{ compactExact(change.after.reserved) }}</span
            ><span v-if="change.after.archived">{{ t('stockArchived') }}</span>
          </div>
        </div>
      </li>
    </ul>
    <section v-if="preview.shopping.length">
      <h3>{{ t('stockShoppingChanges') }}</h3>
      <ul class="stock-change-list">
        <li v-for="change in preview.shopping" :key="change.after.key">
          <strong>{{ change.after.name }}</strong>
          <p>
            {{ t('stockBefore') }}:
            {{ change.before ? compactExact(change.before.remaining) : '—' }} →
            {{ t('stockAfter') }}: {{ compactExact(change.after.remaining) }}
            {{ shoppingUnit(change.after.unit, change.after.remaining) }}
          </p>
          <p>
            {{ t('stockReserved') }}: {{ compactExact(change.after.reserved) }} ·
            {{ t('stockUsed') }}: {{ compactExact(change.after.used) }}
          </p>
        </li>
      </ul>
    </section>
    <section v-if="preview.allocations.length">
      <h3>{{ t('stockSources') }}</h3>
      <ul class="stock-change-list">
        <li v-for="allocation in preview.allocations" :key="allocation.id">
          <strong>{{ lotLabel(allocation.lot_id) }}</strong>
          <p>
            {{ compactExact(allocation.amount) }} {{ lots.get(allocation.lot_id)?.unit }} ·
            {{
              t(
                allocation.state === 'held'
                  ? 'stockHeld'
                  : allocation.state === 'consumed'
                    ? 'stockConsumed'
                    : 'stockReleased',
              )
            }}
            · {{ calendarDate(allocation.use_on) }}
          </p>
          <ShoppingOrigin :origin="allocation.origin" />
        </li>
      </ul>
    </section>
    <template v-for="movement in preview.movements" :key="movement.id"
      ><h3>{{ stockMovementLabel(movement.kind.kind) }}</h3>
      <NcNoteCard
        v-if="movement.kind.kind === 'cooking' && movement.kind.unbooked.length"
        type="warning"
        ><strong>{{ t('stockUnbooked') }}</strong>
        <ul>
          <li v-for="requirement in movement.kind.unbooked" :key="requirement.item.key">
            {{ requirement.item.name }} · {{ compactExact(requirement.amount) }}
            {{ shoppingUnit(requirement.item.unit, requirement.amount) }}
          </li>
        </ul></NcNoteCard
      ></template
    >
    <article v-for="offer in offers" :key="offer.id" class="stock-offer-evidence">
      <h3>{{ stockName(offer.binding) }}</h3>
      <p>
        {{ offer.shop_name }} · {{ compactExact(offer.quantity.amount) }}
        {{ shoppingUnit(offer.quantity.unit, offer.quantity.amount) }} ·
        {{ compactExact(offer.price) }} {{ offer.currency }}
      </p>
      <p>{{ calendarDate(offer.valid_from) }} – {{ calendarDate(offer.valid_until) }}</p>
      <p>
        {{ offer.source }}
        <a
          v-if="offer.source_url"
          :href="offer.source_url"
          target="_blank"
          rel="noopener noreferrer"
          >{{ t('stockSource') }}</a
        >
      </p>
      <p v-if="offer.archived">{{ t('stockArchived') }}</p>
    </article>
  </section>
</template>
