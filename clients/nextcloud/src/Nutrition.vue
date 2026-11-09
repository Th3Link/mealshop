<script setup lang="ts">
import { z } from 'zod'
import { nutritionSchema } from './contracts'
import { exact, compactExact, t } from './i18n'
defineProps<{ value: z.infer<typeof nutritionSchema> }>()
</script>
<template>
  <section>
    <h2>{{ t('nutrients') }}</h2>
    <p>{{ t('nutritionLimits') }}</p>
    <p class="muted">{{ t('original') }}</p>
    <div class="table-scroll">
      <table>
        <thead>
          <tr>
            <th scope="col">{{ t('nutrient') }}</th>
            <th scope="col" class="numeric">{{ t('perServing') }}</th>
            <th scope="col" class="numeric">{{ t('total') }}</th>
          </tr>
        </thead>
        <tbody>
          <tr v-for="n in value.nutrients" :key="n.id">
            <th scope="row" lang="en">
              {{ n.label }} <small>{{ n.unit }}</small>
            </th>
            <td class="numeric">
              {{ exact(n.per_serving)
              }}<small v-if="n.per_serving === null"
                >{{ t('knownSubtotal') }}: {{ exact(n.known_per_serving) }}</small
              >
            </td>
            <td class="numeric">
              {{ exact(n.total)
              }}<small v-if="n.total === null"
                >{{ t('knownSubtotal') }}: {{ exact(n.known_total) }}</small
              >
            </td>
          </tr>
        </tbody>
      </table>
    </div>
    <details>
      <summary>{{ t('source') }}</summary>
      <div v-for="item in value.ingredients" :key="item.ingredient_id">
        <h3>{{ item.name }}</h3>
        <p>
          {{ compactExact(item.grams) }} g · {{ item.source.name }} · {{ item.source.record_id }} ·
          {{ item.source.version }} · {{ item.source.license }}
        </p>
        <p lang="en">{{ item.uncertainty }}</p>
        <a
          v-if="item.source.url"
          :href="item.source.url"
          target="_blank"
          rel="noopener noreferrer"
          >{{ t('openSource') }}</a
        >
      </div>
    </details>
  </section>
</template>
