<script setup lang="ts">
import NcCheckboxRadioSwitch from '@nextcloud/vue/components/NcCheckboxRadioSwitch'
import ShoppingOrigin from './ShoppingOrigin.vue'
import { compactExact, locale, t } from './i18n'
import type { Shopping } from './shopping-api'
defineProps<{ items: Shopping['items']; writable: boolean; busy: boolean }>()
const emit = defineEmits<{ check: [key: string, checked: boolean] }>()
const zero = (value: string) => /^0(?:\.0+)?$/.test(value)
</script>
<template>
  <ul class="shopping-items">
    <li v-for="item in items" :key="item.key">
      <NcCheckboxRadioSwitch
        :model-value="zero(item.remaining)"
        :disabled="busy || !writable"
        @update:model-value="emit('check', item.key, zero(item.remaining) ? false : true)"
        >{{ item.translations[locale] ?? item.name }} ·
        {{ compactExact(zero(item.remaining) ? item.purchased : item.remaining) }}
        {{ item.unit === 'piece' ? t('shoppingPieces') : item.unit }}</NcCheckboxRadioSwitch
      >
      <p v-if="!zero(item.remaining) && !zero(item.purchased)" class="muted">
        {{ t('shoppingBought') }}: {{ compactExact(item.purchased) }}
        {{ item.unit === 'piece' ? t('shoppingPieces') : item.unit }}
      </p>
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
