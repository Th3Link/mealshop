import { t, locale, servingCount, calendarDate, mealName } from './i18n'
import type { Inventory, InventoryCommand, StockMovement } from './inventory-api'
import type { ShoppingOrigin } from './shopping-api'
import type { inventoryEnglish } from './inventory-messages'
const actions = {
  create_lot: 'stockNew',
  update_lot: 'stockMetadata',
  adjust_lot: 'stockCount',
  archive_lot: 'stockArchive',
  reserve: 'stockReserveTitle',
  release: 'stockReleaseTitle',
  receive: 'stockReceive',
  consume: 'stockConsume',
  reverse: 'stockReverse',
  save_offer: 'stockOfferNew',
  archive_offer: 'stockOfferArchive',
  add_offer: 'stockOfferAdd',
} satisfies Record<InventoryCommand['operation'], keyof typeof inventoryEnglish>
const movements = {
  opening: 'stockOpening',
  adjustment: 'stockAdjustment',
  purchase: 'stockPurchase',
  cooking: 'stockCooking',
  reversal: 'stockReversal',
} satisfies Record<StockMovement['kind']['kind'], keyof typeof inventoryEnglish>
export const stockActionLabel = (command: InventoryCommand) =>
  t(
    command.operation === 'save_offer' && command.offer.id
      ? 'stockOfferEdit'
      : actions[command.operation],
  )
export const stockMovementLabel = (kind: StockMovement['kind']['kind']) => t(movements[kind])
export const stockName = (binding: Inventory['lots'][number]['binding']) =>
  binding.translations[locale.value] ?? binding.name
export function stockSourceName(origin: ShoppingOrigin): string {
  switch (origin.kind) {
    case 'manual':
      return origin.label
    case 'product':
      return `${origin.product.name} · ${origin.packages} ${t('productPackages')}`
    case 'offer':
      return `${stockName(origin.offer.binding)} · ${origin.offer.shop_name}`
    case 'recipe':
      return `${origin.title} · ${servingCount(origin.servings)}`
    case 'meal':
      return `${origin.title} · ${calendarDate(origin.date)} · ${mealName(origin.meal)}`
  }
}
export function stockToday(): string {
  const now = new Date()
  return `${now.getFullYear()}-${String(now.getMonth() + 1).padStart(2, '0')}-${String(now.getDate()).padStart(2, '0')}`
}
