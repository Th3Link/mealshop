<script setup lang="ts">
import NcDialog from '@nextcloud/vue/components/NcDialog'
import NcButton from '@nextcloud/vue/components/NcButton'
import ShoppingOrigin from './ShoppingOrigin.vue'
import { compactExact, locale, t } from './i18n'
import type { Shopping, ShoppingCommand } from './shopping-api'
import type { ShoppingItem } from './shopping-display'
const props = defineProps<{
  item: ShoppingItem
  shops: Shopping['shops']
  selectedShop: string
  busy: boolean
  writable: boolean
  first: boolean
  last: boolean
  orderingDisabled: boolean
}>()
const emit = defineEmits<{
  close: []
  command: [command: ShoppingCommand]
  move: [delta: number]
}>()
function assign(event: Event) {
  if (event.target instanceof HTMLSelectElement)
    emit('command', {
      operation: 'assign_shop',
      key: props.item.key,
      shop_id: event.target.value || null,
    })
}
</script>
<template>
  <NcDialog
    :name="item.translations[locale] ?? item.name"
    class="mealshop shopping-detail-dialog"
    size="normal"
    :buttons="[{ label: t('shoppingClose') }]"
    @update:open="!$event && emit('close')"
    ><section class="shopping-detail-section">
      <h3>{{ t('shoppingContributions') }}</h3>
      <div v-for="part in item.contributions" :key="part.source_id">
        <ShoppingOrigin :origin="part.origin" />
        <p>
          {{ t('shoppingRequired') }}: {{ compactExact(part.required) }} ·
          {{ t('shoppingBought') }}: {{ compactExact(part.purchased) }}
          {{ item.unit === 'piece' ? t('shoppingPieces') : item.unit }}
        </p>
        <p v-for="(receipt, index) in part.receipts" :key="index">
          {{ t('shoppingBought') }}: {{ compactExact(receipt.amount) }}
          {{ item.unit === 'piece' ? t('shoppingPieces') : item.unit }} ·
          <ShoppingOrigin :origin="receipt.origin" />
        </p>
      </div>
    </section>
    <section v-if="shops.entries.length" class="shopping-detail-section">
      <h3>{{ t('shopFilter') }}</h3>
      <p>
        {{ shops.entries.find((s) => s.id === item.shop_id)?.name ?? t('shopUnassigned') }} ·
        {{
          t(
            (
              {
                manual: 'shopManual',
                learned: 'shopLearned',
                uncertain: 'shopUncertain',
                default: 'shopDefault',
              } as const
            )[item.order_basis],
          )
        }}
        · {{ t('shopSamples') }}: {{ item.samples }}
      </p>
      <label class="field"
        >{{ t('shopAssign')
        }}<select
          :aria-label="t('shopAssign')"
          :value="item.shop_id ?? ''"
          :disabled="busy || !writable"
          @change="assign"
        >
          <option value="">{{ t('shopUnassigned') }}</option>
          <option v-for="shop in shops.entries" :key="shop.id" :value="shop.id">
            {{ shop.name }}
          </option>
        </select></label
      >
      <div v-if="writable" class="actions">
        <NcButton
          v-if="item.fallback_shop_id"
          :disabled="busy"
          @click="
            emit('command', {
              operation: 'assign_shop',
              key: item.key,
              shop_id: item.fallback_shop_id,
            })
          "
          >{{ t('shopFallback') }}:
          {{ shops.entries.find((s) => s.id === item.fallback_shop_id)?.name }}</NcButton
        ><template v-if="selectedShop !== 'all' && selectedShop !== 'none' && !orderingDisabled"
          ><NcButton :disabled="busy || first" @click="emit('move', -1)">{{
            t('shopMoveUp')
          }}</NcButton
          ><NcButton :disabled="busy || last" @click="emit('move', 1)">{{
            t('shopMoveDown')
          }}</NcButton></template
        >
        <p v-if="orderingDisabled" class="muted">{{ t('shoppingClearToOrder') }}</p>
      </div>
    </section>
  </NcDialog>
</template>
