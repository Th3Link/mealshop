import type { BrewRecipe, BrewTemplates, BrewEquipmentInput } from './brewing-api'
import { locale, t } from './i18n'
export const fermentationProcesses = {
  cider_juice: 'brewCider',
  white_wine: 'brewWhiteWine',
  red_wine: 'brewRedWine',
  traditional_mead: 'brewMead',
} as const
export const beverages = {
  beer: 'brewBeer',
  cider: 'brewCider',
  wine: 'brewWine',
  mead: 'brewMead',
} as const
export const methodOptions = {
  strategy: {
    extract: 'brewExtract',
    extract_steep: 'brewExtractSteep',
    partial_mash: 'brewPartialMash',
    all_grain: 'brewAllGrain',
  },
  layout: {
    kettle: 'brewKettle',
    bag: 'brewBag',
    top_return: 'brewTopReturn',
    malt_pipe: 'brewMaltPipe',
    separate_tun: 'brewSeparateTun',
    two_vessel: 'brewTwoVessel',
    three_vessel: 'brewThreeVessel',
  },
  heat: { manual: 'brewManualHeat', direct: 'brewDirect', herms: 'brewHerms', rims: 'brewRims' },
  sparge: {
    none: 'brewNoneSparge',
    basket_rinse: 'brewBasketRinse',
    batch: 'brewBatch',
    fly: 'brewFly',
  },
  mash: {
    none: 'brewNoneMash',
    infusion: 'brewInfusion',
    step: 'brewStep',
    decoction: 'brewDecoction',
  },
} as const
export const equipmentKinds = {
  kettle: 'brewKettle',
  mash_tun: 'brewMashTun',
  hot_liquor_tank: 'brewHotLiquorTank',
  pump: 'brewPump',
  chiller: 'brewChiller',
  fermenter: 'brewFermenter',
  bag: 'brewBag',
  other: 'brewOther',
} as const
export const readingUnits = {
  celsius: 'brewCelsius',
  fahrenheit: 'brewFahrenheit',
  sg: 'brewSg',
  plato: 'brewPlato',
  brix: 'brewBrix',
  ml: 'brewMl',
  l: 'brewL',
} as const
export function localized(value: { name: string; translations: Record<string, string> }) {
  return value.translations[locale.value] ?? value.name
}
export function newPhase(): BrewRecipe['phases'][number] {
  return {
    kind: 'custom',
    timing_known: false,
    temperature: null,
    volume: null,
    stage: {
      id: `phase-${crypto.randomUUID()}`,
      origin: { kind: 'custom', note: t('brewPhaseSource') },
      name: '',
      instruction: '',
      dependencies: [],
      active_seconds: 0,
      passive_seconds: 0,
      passive_min_seconds: 0,
      passive_max_seconds: 0,
      equipment: [],
      lead_seconds: 0,
      repeat: 1,
      wait_between_seconds: 0,
      uncertainty: '',
      start_after_work: false,
    },
  }
}
export function seedRecipe(c: BrewTemplates, id: string): BrewRecipe | undefined {
  const template = c.templates.find((v) => v.id === id)
  if (!template) return
  const phases = template.phases.flatMap((key, index) => {
    const p = c.phases[key]
    if (!p) return []
    const phase = newPhase()
    phase.kind = p.kind
    phase.stage.id = `phase-${index + 1}`
    phase.stage.name = localized(p)
    phase.stage.instruction = p.instruction_translations[locale.value] ?? p.instruction
    phase.stage.dependencies = index ? [`phase-${index}`] : []
    return [phase]
  })
  let method: BrewRecipe['method']
  if (template.beverage === 'beer') {
    if (!template.strategy || !template.layout || !template.heat || !template.sparge) return
    method = {
      strategy: template.strategy,
      layout: template.layout,
      heat: template.heat,
      sparge: template.sparge,
      mash:
        template.strategy === 'all_grain' || template.strategy === 'partial_mash'
          ? 'infusion'
          : 'none',
    }
  } else {
    if (!template.process) return
    method = { process: template.process }
  }
  return {
    beverage: template.beverage,
    title: '',
    language: locale.value,
    volume: { amount: '', unit: 'l' },
    template: {
      id: template.id,
      version: template.version,
      sources: c.sources.filter((s) => template.sources.includes(s.id)).map((s) => ({ ...s })),
    },
    method,
    phases,
    ingredients: [],
    notes: '',
  }
}
export function newEquipment(): BrewEquipmentInput {
  return { name: '', kind: 'kettle', model: '', capacity: null, notes: '' }
}
