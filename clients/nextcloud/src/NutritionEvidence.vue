<script setup lang="ts">
import type { NutritionEvidence } from './contracts'
import { nutrientLabel, t } from './i18n'
import { evidenceLabel, rangeText } from './nutrition-evidence'
defineProps<{
  profiles: NutritionEvidence[]
  definitions: { id: string; label: string; unit: string }[]
}>()
</script>
<template>
  <section v-for="(profile, index) in profiles" :key="index" class="nutrition-evidence">
    <h4>{{ evidenceLabel(profile.method) }}</h4>
    <p v-if="profile.method !== 'source_supplement'">{{ t('estimateHelp') }}</p>
    <p lang="en">{{ profile.rationale }}</p>
    <small>{{ profile.version }} · {{ t('basis') }}</small>
    <ul class="plain-list">
      <li v-for="reference in profile.references" :key="reference.id">
        <a :href="reference.source.url" target="_blank" rel="noopener noreferrer" lang="en">{{
          reference.name
        }}</a>
        <small lang="en"
          >{{ reference.form }} · {{ reference.source.record_id }} ·
          {{ reference.source.version }} · {{ reference.source.license }} ·
          {{ reference.source.retrieved_on }}</small
        >
        <p class="reference-values">
          <template v-for="(value, id) in reference.values" :key="id"
            ><span
              >{{
                nutrientLabel(
                  String(id),
                  definitions.find((n) => n.id === id)?.label ?? String(id),
                )
              }}: {{ rangeText(value).replace(/^≈ /, '') }}
              {{ definitions.find((n) => n.id === id)?.unit }}</span
            ></template
          >
        </p>
      </li>
    </ul>
  </section>
</template>
