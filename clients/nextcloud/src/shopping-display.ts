import {
  mdiBarley,
  mdiBasketOutline,
  mdiBottleSodaClassicOutline,
  mdiCarrot,
  mdiCheese,
  mdiFoodAppleOutline,
  mdiFoodVariant,
  mdiLeaf,
  mdiPasta,
  mdiPeanutOutline,
  mdiSproutOutline,
} from '@mdi/js'
import { foldSearch } from './ingredient-search'
import type { Ingredient } from './contracts'
import type { Shopping } from './shopping-api'
export type ShoppingItem = Shopping['items'][number]
const categoryIcons: Record<string, string> = {
  grains_flour: mdiBarley,
  legumes_protein: mdiSproutOutline,
  dairy: mdiBottleSodaClassicOutline,
  cheese: mdiCheese,
  plant_drinks: mdiBottleSodaClassicOutline,
  vegetables: mdiCarrot,
  herbs_spices: mdiLeaf,
  fruit: mdiFoodAppleOutline,
  pasta: mdiPasta,
  nuts_seeds: mdiPeanutOutline,
  pantry: mdiFoodVariant,
  eggs_meat: mdiFoodVariant,
}
const zero = (amount: string) => /^0(?:\.0+)?$/.test(amount)
export const isStockCovered = (item: ShoppingItem) =>
  zero(item.remaining) && zero(item.purchased) && (!zero(item.reserved) || !zero(item.used))
export const isPurchased = (item: ShoppingItem) => zero(item.remaining) && !isStockCovered(item)
export function shoppingIcon(
  item: ShoppingItem,
  ingredients: ReadonlyMap<string, Ingredient>,
): string {
  const ingredient = item.ingredient_id ? ingredients.get(item.ingredient_id) : undefined
  return ingredient ? (categoryIcons[ingredient.category] ?? mdiBasketOutline) : mdiBasketOutline
}
export function matchingShoppingItems(items: Shopping['items'], query: string): Shopping['items'] {
  const terms = foldSearch(query).split(/\s+/).filter(Boolean)
  if (!terms.length) return items
  return items.filter((item) => {
    const text = foldSearch([item.name, ...Object.values(item.translations)].join(' '))
    return terms.every((term) => text.includes(term))
  })
}
