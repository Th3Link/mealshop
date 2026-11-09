<script setup lang="ts">
import { computed, watch, useId } from 'vue'
import { useRoute, useRouter } from 'vue-router'
import NcSelect from '@nextcloud/vue/components/NcSelect'
import { z } from 'zod'
import { homes, currentUser, rememberHousehold, lastHousehold } from './state'
import { t } from './i18n'
const id = useId()
const route = useRoute(),
  router = useRouter()
const selected = computed(
  () => homes.value.find((home) => home.id === (route.params.home ?? lastHousehold.value)) ?? null,
)
watch(
  () => [route.params.home, currentUser.value],
  () => {
    if (typeof route.params.home === 'string') rememberHousehold(route.params.home)
  },
  { immediate: true },
)
function switchHome(value: unknown) {
  const result = z.object({ id: z.string() }).safeParse(value)
  if (!result.success || !homes.value.some((home) => home.id === result.data.id)) return
  const section = route.path.split('/')[3]
  void router.push(
    `/h/${result.data.id}/${['recipes', 'ingredients', 'plan', 'household'].includes(section ?? '') ? section : 'recipes'}`,
  )
}
</script>
<template>
  <div v-if="homes.length" class="household-switcher labeled-field">
    <label :for="id">{{ t('households') }}</label>
    <NcSelect
      :input-id="id"
      label-outside
      :input-label="t('households')"
      :model-value="selected"
      :options="homes"
      label="name"
      :clearable="false"
      @update:model-value="switchHome"
      ><template #option="home"
        ><span class="mealshop-selection-name">{{ home.name }}</span></template
      ><template #selected-option="home"
        ><span class="mealshop-selection-name" :title="home.name">{{ home.name }}</span></template
      ><template #no-options>{{ t('noSearchResults') }}</template></NcSelect
    >
  </div>
</template>
