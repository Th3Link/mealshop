<script setup lang="ts">
import NcButton from '@nextcloud/vue/components/NcButton'
import NcTextArea from '@nextcloud/vue/components/NcTextArea'
import NcActions from '@nextcloud/vue/components/NcActions'
import NcActionButton from '@nextcloud/vue/components/NcActionButton'
import CookingDuration from './CookingDuration.vue'
import RecipeReferences from './RecipeReferences.vue'
import Icon from './Icon.vue'
import { mdiPlus, mdiTrashCanOutline } from '@mdi/js'
import type { Recipe } from './contracts'
import { t } from './i18n'
const step = defineModel<Recipe['steps'][number]>({ required: true })
defineProps<{
  index: number
  parts: { id: string; label: string }[]
  ingredients: { id: string; label: string }[]
}>()
const emit = defineEmits<{ remove: [] }>()
</script>
<template>
  <fieldset class="step-editor">
    <legend class="visually-hidden">{{ t('instruction') }} {{ index + 1 }}</legend>
    <div class="step-instruction-row">
      <span class="step-number" aria-hidden="true">{{ index + 1 }}</span>
      <NcTextArea
        v-model="step.instruction"
        :label="`${index + 1}. ${t('instruction')}`"
        :aria-label="`${index + 1}. ${t('instruction')}`"
        label-outside
        :rows="1"
        resize="vertical"
        required
        maxlength="4000"
      />
      <NcActions force-menu :aria-label="`${t('stepExtras')} ${index + 1}`">
        <template #icon><Icon :path="mdiPlus" /></template>
        <NcActionButton
          close-after-click
          v-if="step.passive_max_seconds === undefined"
          @click="step.passive_max_seconds = step.passive_seconds"
          >{{ t('waitingRange') }}</NcActionButton
        >
        <NcActionButton close-after-click v-else @click="delete step.passive_max_seconds">{{
          t('removeWaitingRange')
        }}</NcActionButton>
        <NcActionButton close-after-click v-if="step.part_id !== null" @click="step.part_id = null"
          >{{ t('moveToPart') }} · {{ t('mainRecipe') }}</NcActionButton
        >
        <NcActionButton
          close-after-click
          v-for="part in parts.filter((p) => p.id !== step.part_id)"
          :key="part.id"
          :disabled="step.prepared_parts.includes(part.id)"
          :title="step.prepared_parts.includes(part.id) ? t('preparationInUse') : undefined"
          @click="step.part_id = part.id"
          >{{ t('moveToPart') }} · {{ part.label }}</NcActionButton
        >
      </NcActions>
      <NcButton
        variant="tertiary"
        :aria-label="`${t('remove')} ${t('instruction')} ${index + 1}`"
        @click="emit('remove')"
        ><template #icon><Icon :path="mdiTrashCanOutline" /></template
      ></NcButton>
    </div>
    <div class="step-timing form-row">
      <CookingDuration v-model="step.active_seconds" :label="t('cookActiveDuration')" />
      <CookingDuration v-model="step.passive_seconds" :label="t('cookPassiveDuration')" />
      <CookingDuration
        v-if="step.passive_max_seconds !== undefined"
        v-model="step.passive_max_seconds"
        :label="t('maximumWait')"
      />
    </div>
    <div v-if="ingredients.length || parts.length > 1" class="step-associations form-row">
      <RecipeReferences
        v-if="ingredients.length"
        v-model="step.ingredient_line_ids"
        :label="t('stepIngredients')"
        :options="ingredients"
      />
      <RecipeReferences
        v-if="parts.some((p) => p.id !== step.part_id)"
        v-model="step.prepared_parts"
        :label="t('stepParts')"
        :options="parts.filter((p) => p.id !== step.part_id)"
      />
    </div>
  </fieldset>
</template>
