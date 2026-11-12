<script setup lang="ts">
import LabeledField from './LabeledField.vue'
import type { StockMetadata } from './inventory-api'
import { t } from './i18n'
const model = defineModel<StockMetadata>({ required: true })
</script>
<template>
  <LabeledField v-model="model.location" :label="t('stockLocation')" required maxlength="100" />
  <div class="form-row">
    <LabeledField v-model="model.batch" :label="t('stockBatch')" maxlength="100" /><LabeledField
      :model-value="model.expiry ?? ''"
      :label="t('stockExpiry')"
      type="date"
      @update:model-value="model.expiry = String($event) || null"
    />
  </div>
</template>
