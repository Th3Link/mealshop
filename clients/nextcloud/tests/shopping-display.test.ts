import { describe, it, expect } from 'vitest'
import { mdiBasketOutline } from '@mdi/js'
import { shoppingSchema } from '../src/shopping-api'
import { matchingShoppingItems, isPurchased, shoppingIcon } from '../src/shopping-display'
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
  it('distinguishes exact zero from a remaining fraction and gives unlinked items a neutral icon', () => {
    expect(isPurchased(item('one', 'One', '0.000000'))).toBe(true)
    expect(isPurchased(item('one', 'One', '0.000001'))).toBe(false)
    expect(shoppingIcon(item('oats', 'Milk'), new Map())).toBe(mdiBasketOutline)
  })
})
