import { createApp, nextTick } from 'vue'
import { createRouter, createWebHistory } from 'vue-router'
import { generateUrl } from '@nextcloud/router'
import App from './App.vue'
import Households from './Households.vue'
import Catalogue from './Catalogue.vue'
import Recipes from './Recipes.vue'
import Household from './Household.vue'
import MealPlan from './MealPlan.vue'
import './style.css'
const router = createRouter({
  history: createWebHistory(generateUrl('/apps/app_api/embedded/mealshop/main')),
  routes: [
    { path: '/', redirect: '/households' },
    { path: '/households', component: Households },
    { path: '/h/:home/ingredients/:id?', component: Catalogue },
    { path: '/h/:home/recipes/:id?', component: Recipes },
    { path: '/h/:home/household', component: Household },
    { path: '/h/:home/plan', component: MealPlan },
    { path: '/:pathMatch(.*)*', redirect: '/households' },
  ],
})
router.afterEach(async () => {
  await nextTick()
  document.querySelector<HTMLElement>('.mealshop h1')?.focus()
})
const mount = document.getElementById('content')
if (mount) createApp(App).use(router).mount(mount)
