import { describe, it, expect } from 'vitest'
import { mdiBasketOutline } from '@mdi/js'
import { shoppingSchema } from '../src/shopping-api'
import {
  matchingShoppingItems,
  isPurchased,
  isStockCovered,
  shoppingIcon,
} from '../src/shopping-display'
const item = (key: string, name: string, remaining = '1.000001') =>
  shoppingSchema.shape.items.element.parse({
    key,
    name,
    ingredient_id: null,
    translations: { de: key === 'oats' ? 'Haferflocken' : 'Küchenpapier' },
    unit: 'g',
    shop_id: null,
    fallback_shop_id: null,
    order_basis: 'default',
    samples: 0,
    remaining,
    purchased: '0.000000',
    reserved: '0.000000',
    used: '0.000000',
    total: remaining,
    contributions: [],
  })
describe('visual shopping discovery', () => {
  it('finds all words across translated names without changing identity, quantities or order', () => {
    const oats = item('oats', 'Oats'),
      paper = item('paper', 'Kitchen roll')
    const list = [oats, paper]
    expect(matchingShoppingItems(list, 'HAFER')).toEqual([oats])
    expect(matchingShoppingItems(list, 'kuchenpapier')).toEqual([paper])
    expect(matchingShoppingItems(list, 'kitchen roll')).toEqual([paper])
    expect(matchingShoppingItems(list, 'kitchen oats')).toEqual([])
    expect(matchingShoppingItems(list, '  ')).toBe(list)
    expect(list).toEqual([oats, paper])
    expect(oats.remaining).toBe('1.000001')
  })
  it('separates held and consumed stock from purchases, including partial and mixed coverage', () => {
    for (const field of ['reserved', 'used'] as const) {
      const covered = { ...item('oats', 'Oats', '0.000000'), [field]: '100.000000' }
      expect(isStockCovered(covered)).toBe(true)
      expect(isPurchased(covered)).toBe(false)
      const partial = { ...covered, remaining: '0.000001' }
      expect(isStockCovered(partial)).toBe(false)
      expect(isPurchased(partial)).toBe(false)
      const mixed = { ...covered, purchased: '50.000000' }
      expect(isStockCovered(mixed)).toBe(false)
      expect(isPurchased(mixed)).toBe(true)
    }
  })
  it('distinguishes exact zero from a remaining fraction and gives unlinked items a neutral icon', () => {
    expect(isPurchased(item('one', 'One', '0.000000'))).toBe(true)
    expect(isPurchased(item('one', 'One', '0.000001'))).toBe(false)
    expect(shoppingIcon(item('oats', 'Milk'), new Map())).toBe(mdiBasketOutline)
  })
})
