<script setup lang="ts">
import type { Product } from './shopping-api'
import { timestamp, t } from './i18n'
defineProps<{ evidence: NonNullable<Product['evidence']> }>()
</script>
<template>
  <div class="source-evidence">
    <p>
      <a :href="evidence.source_url" target="_blank" rel="noreferrer">Open Food Facts</a> ·
      <a href="https://opendatacommons.org/licenses/odbl/1-0/" target="_blank" rel="noreferrer">{{
        evidence.licence
      }}</a>
    </p>
    <p>
      {{ t('productRetrieved') }}: {{ timestamp(evidence.retrieved_at) }} ·
      {{ t('productModified') }}:
      {{ evidence.modified_at ? timestamp(evidence.modified_at) : t('unknown') }}
    </p>
    <details>
      <summary>{{ t('productOriginal') }}</summary>
      <p>
        {{ evidence.original_name }} · {{ evidence.original_brand }} ·
        {{ evidence.original_quantity }}
      </p>
    </details>
  </div>
</template>
