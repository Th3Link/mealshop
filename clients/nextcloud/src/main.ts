import { createApp, nextTick } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { generateUrl } from '@nextcloud/router'
import App from './App.vue'
import Households from './Households.vue'
import Catalogue from './Catalogue.vue'
import Recipes from './Recipes.vue'
import Household from './Household.vue'
import MealPlan from './MealPlan.vue'
import Shopping from './Shopping.vue'
import Cooking from './Cooking.vue'
import Collections from './Collections.vue'
import SharedWithMe from './SharedWithMe.vue'
import SharedContent from './SharedContent.vue'
import { loadHomes, lastHousehold, rememberHousehold } from './state'
import './style.css'
const generatedBase = generateUrl('/apps/app_api/embedded/mealshop/main')
const prettyBase = generatedBase.replace('/index.php/apps/app_api/', '/apps/app_api/')
const indexedBase = prettyBase.replace('/apps/app_api/', '/index.php/apps/app_api/')
const routingBase =
  [indexedBase, prettyBase].find(
    (base) => location.pathname === base || location.pathname.startsWith(`${base}/`),
  ) ?? generatedBase
const router = createRouter({
  history: createWebHistory(routingBase),
  routes: [
    { path: '/', component: Households },
    { path: '/households', component: Households },
    { path: '/shared', component: SharedWithMe },
    { path: '/shared/:sourceHome/:share', component: SharedContent },
    { path: '/shared/:sourceHome/:share/recipes/:recipe', component: SharedContent },
    { path: '/h/:home/ingredients/:id?', component: Catalogue },
    { path: '/h/:home/recipes/:id?', component: Recipes },
    { path: '/h/:home/collections/:id?', component: Collections },
    { path: '/h/:home/household', component: Household },
    { path: '/h/:home/plan', component: MealPlan },
    { path: '/h/:home/shopping', component: Shopping },
    { path: '/h/:home/cooking/:id?', component: Cooking },
    { path: '/:pathMatch(.*)*', redirect: '/households' },
  ],
})
router.beforeEach(async (to) => {
  if (to.path !== '/') return
  try {
    await loadHomes()
    return lastHousehold.value ? `/h/${lastHousehold.value}/recipes` : '/households'
  } catch {
    return '/households'
  }
})
router.afterEach(async (to, from, failure) => {
  if (failure) return
  if (typeof to.params.home === 'string') rememberHousehold(to.params.home)
  if (to.path === from.path && to.query.edit === from.query.edit) return
  await nextTick()
  document.querySelector<HTMLElement>('.mealshop h1')?.focus()
})
const mount = document.getElementById('content')
if (mount) createApp(App).use(router).mount(mount)
