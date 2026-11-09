<script setup lang="ts">
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import ShoppingOrigin from './ShoppingOrigin.vue'
import { compactExact, shoppingUnit, locale, t } from './i18n'
import type { Shopping, ShoppingCommand } from './shopping-api'
const props = defineProps<{
  items: Shopping['items']
  writable: boolean
  busy: boolean
  shops: Shopping['shops']
  selectedShop: string
}>()
const emit = defineEmits<{
  check: [key: string, checked: boolean]
  command: [command: ShoppingCommand]
}>()
function assign(key: string, event: Event) {
  if (event.target instanceof HTMLSelectElement)
    emit('command', { operation: 'assign_shop', key, shop_id: event.target.value || null })
}
function move(index: number, delta: number) {
  const keys = props.items.map((i) => i.key),
    other = index + delta
  const key = keys[index],
    adjacent = keys[other]
  if (!props.selectedShop || key === undefined || adjacent === undefined) return
  keys[index] = adjacent
  keys[other] = key
  emit('command', { operation: 'order', shop_id: props.selectedShop, keys })
}
const zero = (value: string) => /^0(?:\.0+)?$/.test(value)
</script>
<template>
  <ul class="shopping-items">
    <li v-for="(item, index) in items" :key="item.key" :data-item-key="item.key">
      <NcCheckboxRadioSwitch
        :model-value="zero(item.remaining)"
        :disabled="busy || !writable"
        @update:model-value="emit('check', item.key, zero(item.remaining) ? false : true)"
        >{{ item.translations[locale] ?? item.name }} ·
        {{ compactExact(zero(item.remaining) ? item.purchased : item.remaining) }}
        {{
          shoppingUnit(item.unit, zero(item.remaining) ? item.purchased : item.remaining)
        }}</NcCheckboxRadioSwitch
      >
      <p v-if="!zero(item.remaining) && !zero(item.purchased)" class="muted">
        {{ t('shoppingBought') }}: {{ compactExact(item.purchased) }}
        {{ shoppingUnit(item.unit, item.purchased) }}
      </p>
      <details v-if="shops.entries.length">
        <summary>
          {{ shops.entries.find((s) => s.id === item.shop_id)?.name ?? t('shopUnassigned') }}
        </summary>
        <p>
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
            @change="assign(item.key, $event)"
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
          ><template v-if="selectedShop && selectedShop !== 'all' && selectedShop !== 'none'"
            ><NcButton :disabled="busy || index === 0" @click="move(index, -1)">{{
              t('shopMoveUp')
            }}</NcButton
            ><NcButton :disabled="busy || index === items.length - 1" @click="move(index, 1)">{{
              t('shopMoveDown')
            }}</NcButton></template
          >
        </div>
      </details>
      <details>
        <summary>{{ t('shoppingContributions') }}</summary>
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
      </details>
    </li>
  </ul>
</template>
