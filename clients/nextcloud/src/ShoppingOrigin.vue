<script setup lang="ts">
import { compactExact, t } from './i18n'
import type { ShoppingOrigin } from './shopping-api'
defineProps<{ origin: ShoppingOrigin }>()
</script>
<template>
  <span v-if="origin.kind === 'manual'"
    >{{ origin.label }} · {{ compactExact(origin.quantity.amount) }}
    {{ origin.quantity.unit === 'piece' ? t('shoppingPieces') : origin.quantity.unit }}</span
  >
  <span v-else
    >{{ origin.title }} · {{ t('revision') }} {{ origin.revision }} · {{ origin.servings }}
    {{ t('servings')
    }}<template v-if="origin.kind === 'meal'">
      · {{ origin.date }} · {{ origin.meal }}</template
    ></span
  >
</template>
