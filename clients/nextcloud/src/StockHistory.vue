<script setup lang="ts">
import NcButton from '@nextcloud/vue/components/NcButton'
import NcEmptyContent from '@nextcloud/vue/components/NcEmptyContent'
import ShoppingOrigin from './ShoppingOrigin.vue'
import { t, compactExact, shoppingUnit, timestamp, calendarDate } from './i18n'
import { stockName, stockMovementLabel } from './inventory-format'
import type { Inventory, InventoryCommand } from './inventory-api'
const props = defineProps<{ data: Inventory; writable: boolean; busy: boolean; home: string }>()
const emit = defineEmits<{ action: [command: InventoryCommand]; page: [offset: number] }>()
function lotLabel(id: string) {
  const lot = props.data.lots.find((l) => l.id === id)
  return lot ? stockName(lot.binding) : t('unknown')
}
function quantity(id: string, amount: string) {
  const lot = props.data.lots.find((l) => l.id === id)
  return `${compactExact(amount)} ${lot ? shoppingUnit(lot.unit, amount) : t('unknown')}`
}
</script>
<template>
  <section>
    <h2>{{ t('stockReserved') }}</h2>
    <p v-if="!data.allocations.some((a) => a.state === 'held')" class="muted">
      {{ t('stockNoHolds') }}
    </p>
    <ul class="stock-change-list">
      <li
        v-for="allocation in data.allocations.filter((a) => a.state === 'held')"
        :key="allocation.id"
      >
        <strong>{{ lotLabel(allocation.lot_id) }}</strong>
        <p>
          {{ quantity(allocation.lot_id, allocation.amount) }} ·
          {{ calendarDate(allocation.use_on) }}
        </p>
        <ShoppingOrigin :origin="allocation.origin" /><NcButton
          v-if="writable"
          variant="tertiary"
          :disabled="busy"
          @click="emit('action', { operation: 'release', allocation_ids: [allocation.id] })"
          >{{ t('stockRelease') }}</NcButton
        >
      </li>
    </ul>
  </section>
  <h2>{{ t('stockMovements') }}</h2>
  <NcEmptyContent v-if="!data.movements.length" :name="t('stockMovementsEmpty')" />
  <ol class="stock-history">
    <li v-for="movement in data.movements" :key="movement.id" :id="`stock-${movement.id}`">
      <header>
        <h3>{{ stockMovementLabel(movement.kind.kind) }}</h3>
        <span class="muted"
          >{{ movement.recorded_at ? timestamp(movement.recorded_at) : t('unknown') }} ·
          {{ movement.actor }}</span
        >
      </header>
      <p v-if="movement.reversed">{{ t('stockReversed') }}</p>
      <p v-if="movement.kind.kind === 'adjustment' || movement.kind.kind === 'reversal'">
        {{ movement.kind.reason }}
      </p>
      <ul>
        <li v-for="(entry, index) in movement.entries" :key="index">
          {{ lotLabel(entry.lot_id) }} · {{ entry.direction === 'in' ? '+' : '−' }}
          {{ quantity(entry.lot_id, entry.amount) }}
        </li>
      </ul>
      <template v-if="movement.kind.kind === 'purchase'"
        ><div v-for="purchase in movement.kind.purchases" :key="purchase.id">
          <ShoppingOrigin :origin="purchase.origin" /></div></template
      ><template v-if="movement.kind.kind === 'cooking'"
        ><p>
          <RouterLink :to="`/h/${home}/cooking/${movement.kind.run_id}`">{{
            t('stockCooking')
          }}</RouterLink>
          · {{ calendarDate(movement.kind.use_on) }}
        </p>
        <p v-for="requirement in movement.kind.unbooked" :key="requirement.item.key">
          {{ t('stockUnbooked') }}: {{ requirement.item.name }} ·
          {{ compactExact(requirement.amount) }}
          {{ shoppingUnit(requirement.item.unit, requirement.amount) }}
        </p></template
      ><NcButton
        v-if="writable && !movement.reversed && movement.kind.kind !== 'reversal'"
        :disabled="busy"
        variant="tertiary"
        @click="emit('action', { operation: 'reverse', movement_id: movement.id, reason: '' })"
        >{{ t('stockReverse') }}</NcButton
      >
    </li>
  </ol>
  <div v-if="data.movement_total > data.movement_limit" class="actions">
    <NcButton
      :disabled="busy || data.movement_offset === 0"
      @click="emit('page', Math.max(0, data.movement_offset - data.movement_limit))"
      >{{ t('stockPagesPrevious') }}</NcButton
    ><span
      >{{ data.movement_offset + 1 }}–{{
        Math.min(data.movement_total, data.movement_offset + data.movement_limit)
      }}
      / {{ data.movement_total }}</span
    ><NcButton
      :disabled="busy || data.movement_offset + data.movement_limit >= data.movement_total"
      @click="emit('page', data.movement_offset + data.movement_limit)"
      >{{ t('stockPagesNext') }}</NcButton
    >
  </div>
</template>
