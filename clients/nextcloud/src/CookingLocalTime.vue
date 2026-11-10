<script setup lang="ts">
import { useId } from 'vue'
import type { LocalTime } from './cooking-api'
import { t } from './i18n'
const model = defineModel<LocalTime>({ required: true })
defineProps<{ label: string; zoneEditable?: boolean }>()
const id = useId()
</script>
<template>
  <fieldset class="cooking-local-time">
    <legend>{{ label }}</legend>
    <div class="form-row">
      <label class="field" :for="id"
        >{{ t('cookLocalTime')
        }}<input
          :id="id"
          v-model="model.local"
          type="datetime-local"
          step="1"
          min="2020-01-01T00:00"
          max="2100-12-31T23:59:59"
          required /></label
      ><label v-if="zoneEditable" class="field"
        >{{ t('cookTimezone')
        }}<input
          v-model="model.timezone"
          type="text"
          required
          maxlength="100"
          :aria-label="t('cookTimezone')"
          spellcheck="false"
      /></label>
    </div>
    <p class="muted">{{ model.timezone }} · {{ t('cookTimeZoneHelp') }}</p>
    <label class="field"
      >{{ t('cookFoldChoice')
      }}<select v-model="model.fold" :aria-label="t('cookFoldChoice')">
        <option value="reject">{{ t('cookFoldReject') }}</option>
        <option value="earlier">{{ t('cookFoldEarlier') }}</option>
        <option value="later">{{ t('cookFoldLater') }}</option>
      </select></label
    >
  </fieldset>
</template>
