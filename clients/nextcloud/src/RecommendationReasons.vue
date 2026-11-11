<script setup lang="ts">
import type { z } from 'zod'
import { recommendationSchema } from './planning-api'
import { t, locale } from './i18n'
defineProps<{ value: z.infer<typeof recommendationSchema> }>()
const confidence = {
  cold_start: 'suggestNoHistory',
  limited_history: 'suggestLimited',
  repeated_history: 'suggestRepeated',
} as const
function number(value: number) {
  return new Intl.NumberFormat(locale.value).format(value)
}
</script>
<template>
  <div class="recommendation-reasons">
    <p v-if="value.reasons.includes('declared_dislikes')">{{ t('suggestDislikes') }}</p>
    <strong v-if="value.discovery">{{ t('suggestDiscovery') }}</strong>
    <p>
      {{ t(confidence[value.confidence])
      }}<template v-if="value.evidence.cooks">
        · {{ number(value.evidence.cooks) }}
        {{ t(value.evidence.cooks === 1 ? 'suggestCookOne' : 'suggestCookMany') }}</template
      >
    </p>
    <p v-if="value.evidence.household_ratings || value.evidence.personal_ratings">
      {{ t('suggestHouseholdSamples') }}: {{ number(value.evidence.household_ratings) }} ·
      {{ t('suggestPersonalSamples') }}: {{ number(value.evidence.personal_ratings) }}
    </p>
    <p v-if="value.evidence.recent_cooks">
      {{ t('suggestRecent') }}: {{ number(value.evidence.recent_cooks) }}
    </p>
    <p v-if="value.evidence.earlier_revision_cooks">
      {{ t('suggestEarlier') }}: {{ number(value.evidence.earlier_revision_cooks) }}
    </p>
  </div>
</template>
