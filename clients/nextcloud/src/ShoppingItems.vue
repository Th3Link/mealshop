<script setup lang="ts">
import { computed, ref, watch, nextTick } from 'vue'
import NcButton from '@nextcloud/vue/components/NcButton'
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import { mdiDotsHorizontal } from '@mdi/js'
import Icon from './Icon.vue'
import ShoppingItemDetails from './ShoppingItemDetails.vue'
import { compactExact, shoppingUnit, locale, t } from './i18n'
import { isPurchased, shoppingIcon } from './shopping-display'
import type { Shopping, ShoppingCommand } from './shopping-api'
import type { Ingredient } from './contracts'
const props = defineProps<{
  items: Shopping['items']
  writable: boolean
  busy: boolean
  shops: Shopping['shops']
  selectedShop: string
  ingredients: ReadonlyMap<string, Ingredient>
  orderingDisabled: boolean
}>()
const emit = defineEmits<{
  check: [key: string, checked: boolean]
  command: [command: ShoppingCommand]
}>()
const selectedKey = ref<string>(),
  selected = computed(() => props.items.find((item) => item.key === selectedKey.value)),
  selectedIndex = computed(() => props.items.findIndex((item) => item.key === selectedKey.value))
watch(selected, (item, previous) => {
  if (!item && previous) {
    selectedKey.value = undefined
    void nextTick(() =>
      document.querySelector<HTMLInputElement>('#mealshop-shopping-search input')?.focus(),
    )
  }
})
function move(delta: number) {
  const keys = props.items.map((i) => i.key),
    index = selectedIndex.value,
    other = index + delta,
    key = keys[index],
    adjacent = keys[other]
  if (props.orderingDisabled || index < 0 || key === undefined || adjacent === undefined) return
  keys[index] = adjacent
  keys[other] = key
  emit('command', { operation: 'order', shop_id: props.selectedShop, keys })
}
function label(item: Shopping['items'][number]) {
  const amount = isPurchased(item) ? item.purchased : item.remaining
  return `${item.translations[locale.value] ?? item.name} · ${compactExact(amount)} ${shoppingUnit(item.unit, amount)}`
}
</script>
<template>
  <ul class="shopping-items">
    <li
      v-for="item in items"
      :key="item.key"
      :data-item-key="item.key"
      :class="{ 'shopping-item--purchased': isPurchased(item) }"
    >
      <NcCheckboxRadioSwitch
        class="shopping-tile"
        :model-value="isPurchased(item)"
        :disabled="busy || !writable"
        :aria-label="label(item)"
        @update:model-value="emit('check', item.key, !isPurchased(item))"
        ><span class="shopping-tile__body"
          ><Icon :path="shoppingIcon(item, ingredients)" class="shopping-tile__illustration" /><span
            class="shopping-tile__caption"
            ><span class="shopping-tile__name" :title="item.translations[locale] ?? item.name">{{
              item.translations[locale] ?? item.name
            }}</span
            ><span class="visually-hidden"> · </span>
            <span class="shopping-tile__quantity"
              >{{ compactExact(isPurchased(item) ? item.purchased : item.remaining) }}
              {{
                shoppingUnit(item.unit, isPurchased(item) ? item.purchased : item.remaining)
              }}</span
            ></span
          ></span
        ></NcCheckboxRadioSwitch
      >
      <div class="shopping-tile__footer">
        <span class="shopping-tile__state"
          >{{ t(isPurchased(item) ? 'shoppingBoughtState' : 'shoppingOpenState')
          }}<template v-if="!isPurchased(item) && !/^0(?:\.0+)?$/.test(item.purchased)">
            · {{ t('shoppingBought') }}: {{ compactExact(item.purchased) }}
            {{ shoppingUnit(item.unit, item.purchased) }}
          </template></span
        ><NcButton
          variant="tertiary"
          :aria-label="`${t('shoppingItemDetails')}: ${item.translations[locale] ?? item.name}`"
          @click="selectedKey = item.key"
          ><template #icon><Icon :path="mdiDotsHorizontal" /></template
        ></NcButton>
      </div>
    </li>
  </ul>
  <ShoppingItemDetails
    v-if="selected"
    :item="selected"
    :shops="shops"
    :selected-shop="selectedShop"
    :busy="busy"
    :writable="writable"
    :first="selectedIndex === 0"
    :last="selectedIndex === items.length - 1"
    :ordering-disabled="orderingDisabled"
    @close="selectedKey = undefined"
    @command="emit('command', $event)"
    @move="move"
  />
</template>
