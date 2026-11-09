<script setup lang="ts">
import { z } from 'zod'
import { nutritionSchema } from './contracts'
import { exact, compactExact, t, locale, nutrientLabel } from './i18n'
defineProps<{ value: z.infer<typeof nutritionSchema> }>()
</script>
<template>
  <section>
    <h2>{{ t('nutrients') }}</h2>
    <p>{{ t('nutritionLimits') }}</p>
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
          <tr v-for="n in value.nutrients.filter((n) => n.group !== 'component')" :key="n.id">
            <th scope="row">
              {{ nutrientLabel(n.id, n.label) }} <small>{{ n.unit }}</small>
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
    <div v-if="value.constituents.length" class="constituent-summary">
      <h3>{{ t('foodConstituents') }}</h3>
      <p class="muted">{{ t('constituentsSummaryHelp') }}</p>
      <div class="table-scroll">
        <table>
          <thead>
            <tr>
              <th scope="col">{{ t('constituentName') }}</th>
              <th scope="col" class="numeric">{{ t('perServing') }}</th>
              <th scope="col" class="numeric">{{ t('total') }}</th>
            </tr>
          </thead>
          <tbody>
            <tr v-for="c in value.constituents" :key="c.id">
              <th scope="row">
                {{ c.translations[locale] ?? nutrientLabel(c.id, c.label) }}
                <small>{{ c.unit }}</small>
              </th>
              <td class="numeric">
                {{ exact(c.per_serving)
                }}<small v-if="c.per_serving === null"
                  >{{ t('knownSubtotal') }}: {{ exact(c.known_per_serving) }}</small
                >
              </td>
              <td class="numeric">
                {{ exact(c.total)
                }}<small v-if="c.total === null"
                  >{{ t('knownSubtotal') }}: {{ exact(c.known_total) }}</small
                >
              </td>
            </tr>
          </tbody>
        </table>
      </div>
    </div>
    <details>
      <summary>{{ t('source') }}</summary>
      <div v-for="item in value.ingredients" :key="item.ingredient_id">
        <h3>{{ item.translations[locale]?.name ?? item.name }}</h3>
        <p>
          {{ compactExact(item.grams) }} g · {{ item.source.name }} · {{ item.source.record_id }} ·
          {{ item.source.version }} · {{ item.source.license }}
        </p>
        <p :lang="item.source.kind === 'primary' ? 'en' : undefined">{{ item.uncertainty }}</p>
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
