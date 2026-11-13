<script setup lang="ts">
import BrewNumber from './BrewNumber.vue'
import type { z } from 'zod'
import type { brewQuantity } from './brewing-api'
import { t, shoppingUnit } from './i18n'
const value = defineModel<z.infer<typeof brewQuantity>>({ required: true })
defineProps<{ label: string; volume?: boolean }>()
</script>
<template>
  <div class="brew-quantity">
    <BrewNumber v-model="value.amount" :label="label" required /><label class="field"
      ><span class="visually-hidden">{{ label }} · {{ t('unit') }}</span
      ><select v-model="value.unit" :aria-label="`${label} · ${t('unit')}`">
        <option
          v-for="u in volume ? (['ml', 'l'] as const) : (['g', 'kg', 'ml', 'l', 'piece'] as const)"
          :key="u"
          :value="u"
        >
          {{ shoppingUnit(u, '2') }}
        </option>
      </select></label
    >
  </div>
</template>
