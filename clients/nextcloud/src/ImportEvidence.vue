<script setup lang="ts">
import { t, locale } from './i18n'
import type { ImportSource } from './recipe-import'
defineProps<{ source: ImportSource; withText?: boolean }>()
</script>
<template>
  <section class="import-evidence">
    <h2>{{ t('importOriginal') }}</h2>
    <p class="muted">{{ t('importSourceNote') }}</p>
    <p>
      <strong>{{ t('importSource') }}:</strong>
      <a :href="source.requested_url" target="_blank" rel="noopener noreferrer">{{
        source.requested_url
      }}</a>
    </p>
    <p v-if="source.final_url && source.final_url !== source.requested_url">
      <strong>{{ t('importFinal') }}:</strong>
      <a :href="source.final_url" target="_blank" rel="noopener noreferrer">{{
        source.final_url
      }}</a>
    </p>
    <dl class="import-facts">
      <div
        v-for="(value, key) in {
          importSourceTitle: source.original.title,
          importSourceAuthor: source.original.author,
          importSourceLanguage: source.original.language,
          importSourceYield: source.original.yield_text,
          importSourcePrep: source.original.prep_time,
          importSourceCook: source.original.cook_time,
          importSourceTotal: source.original.total_time,
        }"
        :key="key"
      >
        <dt>{{ t(key) }}</dt>
        <dd>{{ value || t('importUnknown') }}</dd>
      </div>
    </dl>
    <p v-if="source.retrieved_at" class="muted">
      {{
        new Intl.DateTimeFormat(locale, { dateStyle: 'medium', timeStyle: 'short' }).format(
          new Date(source.retrieved_at * 1000),
        )
      }}
    </p>
    <template v-if="withText">
      <template v-if="source.original.ingredients.length"
        ><h3>{{ t('importSourceIngredients') }}</h3>
        <p v-if="!source.original.ingredients.length">{{ t('importUnknown') }}</p>
        <ol v-else>
          <li v-for="(line, index) in source.original.ingredients" :key="index">{{ line }}</li>
        </ol></template
      >
      <template v-if="source.original.steps.length"
        ><h3>{{ t('importSourceInstructions') }}</h3>
        <p v-if="!source.original.steps.length">{{ t('importUnknown') }}</p>
        <ol v-else>
          <li v-for="(step, index) in source.original.steps" :key="index">
            <strong v-if="step.section">{{ step.section }}: </strong>{{ step.text }}
          </li>
        </ol></template
      >
    </template>
  </section>
</template>
