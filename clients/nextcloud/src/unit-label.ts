import { t } from './i18n'
import type { Unit } from './quantity-api'
const keys = {
  g: 'unitG',
  kg: 'unitKg',
  ml: 'unitMl',
  l: 'unitL',
  tsp: 'unitTsp',
  tbsp: 'unitTbsp',
  piece: 'unitPiece',
} as const satisfies Record<Unit, string>
export function unitLabel(unit: Unit): string {
  return t(keys[unit])
}
