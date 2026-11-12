<script setup lang="ts">
import { computed, onMounted } from 'vue'
import { useRoute } from 'vue-router'
import NcContent from '@nextcloud/vue/components/NcContent'
import NcAppNavigation from '@nextcloud/vue/components/NcAppNavigation'
import NcAppNavigationItem from '@nextcloud/vue/components/NcAppNavigationItem'
import NcAppContent from '@nextcloud/vue/components/NcAppContent'
import NcNoteCard from '@nextcloud/vue/components/NcNoteCard'
import {
  mdiBookOpenPageVariantOutline,
  mdiCalendarMonthOutline,
  mdiCarrot,
  mdiAccountGroupOutline,
  mdiHomeOutline,
  mdiBasketOutline,
  mdiChefHat,
  mdiFolderMultipleOutline,
  mdiShareVariantOutline,
  mdiHistory,
  mdiRobotOutline,
  mdiArchiveOutline,
} from '@mdi/js'
import Icon from './Icon.vue'
import HouseholdSwitcher from './HouseholdSwitcher.vue'
import { homes, loadHomes } from './state'
import { locale, t } from './i18n'
const route = useRoute()
const home = computed(() => homes.value.find((h) => h.id === route.params.home))
const links = [
  { key: 'recipes', icon: mdiBookOpenPageVariantOutline },
  { key: 'collections', icon: mdiFolderMultipleOutline },
  { key: 'history', icon: mdiHistory },
  { key: 'plan', icon: mdiCalendarMonthOutline },
  { key: 'shoppingList', icon: mdiBasketOutline },
  { key: 'inventory', icon: mdiArchiveOutline },
  { key: 'cookingSessions', icon: mdiChefHat },
  { key: 'ingredients', icon: mdiCarrot },
  { key: 'household', icon: mdiAccountGroupOutline },
  { key: 'assistants', icon: mdiRobotOutline },
] satisfies {
  key:
    | 'recipes'
    | 'collections'
    | 'history'
    | 'plan'
    | 'ingredients'
    | 'household'
    | 'shoppingList'
    | 'inventory'
    | 'cookingSessions'
    | 'assistants'
  icon: string
}[]
onMounted(() => {
  void loadHomes().catch(() => {})
})
</script>
<template>
  <NcContent app-name="mealshop" class="mealshop">
    <NcAppNavigation :aria-label="t('households')">
      <template #search><HouseholdSwitcher /></template>
      <template #list>
        <NcAppNavigationItem :name="t('households')" to="/households"
          ><template #icon><Icon :path="mdiHomeOutline" /></template
        ></NcAppNavigationItem>
        <NcAppNavigationItem :name="t('sharedWithMe')" to="/shared"
          ><template #icon><Icon :path="mdiShareVariantOutline" /></template
        ></NcAppNavigationItem>
        <NcAppNavigationItem
          v-for="item in home ? links : []"
          :key="item.key"
          :name="t(item.key)"
          :to="`/h/${home?.id}/${item.key === 'shoppingList' ? 'shopping' : item.key === 'cookingSessions' ? 'cooking' : item.key}`"
          ><template #icon><Icon :path="item.icon" /></template
        ></NcAppNavigationItem>
      </template>
      <template #footer
        ><div class="language">
          <label for="mealshop-language">{{ t('language') }}</label
          ><select id="mealshop-language" v-model="locale">
            <option value="en">English</option>
            <option value="de">Deutsch</option>
          </select>
        </div></template
      >
    </NcAppNavigation>
    <NcAppContent
      ><div class="workspace">
        <p v-if="home" class="context">
          {{ home.name }} <span>· {{ t(home.role) }}</span>
        </p>
        <NcNoteCard v-if="home?.role === 'reader'" type="info">{{ t('readOnly') }}</NcNoteCard
        ><RouterView :key="String(route.params.home ?? '')" /></div
    ></NcAppContent>
  </NcContent>
</template>
