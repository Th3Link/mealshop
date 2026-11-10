<script setup lang="ts">
import {
  servingCount,
  calendarDate,
  mealName,
  compactExact,
  shoppingUnit,
  timestamp,
  t,
  locale,
} from './i18n'
import type { ShoppingOrigin } from './shopping-api'
defineProps<{ origin: ShoppingOrigin }>()
</script>
<template>
  <span v-if="origin.kind === 'manual'"
    >{{ origin.label }} · {{ compactExact(origin.quantity.amount) }}
    {{ shoppingUnit(origin.quantity.unit, origin.quantity.amount) }}</span
  >
  <span v-else-if="origin.kind === 'product'"
    >{{ origin.product.name }} · {{ origin.product.barcode }} · {{ origin.packages }}
    {{ t(origin.packages === 1 ? 'productPackage' : 'productPackages') }} ×
    {{ compactExact(origin.product.quantity.amount) }}
    {{ shoppingUnit(origin.product.quantity.unit, origin.product.quantity.amount)
    }}<template v-if="origin.product.evidence">
      ·
      <a :href="origin.product.evidence.source_url" target="_blank" rel="noreferrer"
        >Open Food Facts</a
      >
      · {{ origin.product.evidence.licence }} · {{ t('productRetrieved') }}:
      {{ timestamp(origin.product.evidence.retrieved_at) }} · {{ t('productModified') }}:
      {{
        origin.product.evidence.modified_at
          ? timestamp(origin.product.evidence.modified_at)
          : t('unknown')
      }}</template
    ></span
  >
  <span v-else
    ><span :lang="origin.language">{{ origin.title }}</span
    ><template v-if="origin.language !== locale">
      · {{ t('originalLanguage') }}:
      {{ origin.language === 'de' ? 'Deutsch' : 'English' }}</template
    >
    · {{ t('revision') }} {{ origin.revision }} · {{ servingCount(origin.servings)
    }}<template v-if="origin.kind === 'meal'">
      · {{ calendarDate(origin.date) }} · {{ mealName(origin.meal) }}</template
    ></span
  >
</template>
