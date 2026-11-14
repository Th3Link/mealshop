import '../connection/nextcloud.dart';
import '../content/api.dart';

const brewBeverages = {
  'beer': 'brewBeer',
  'cider': 'brewCider',
  'wine': 'brewWine',
  'mead': 'brewMead',
};
const brewMethods = <String, Map<String, String>>{
  'strategy': {
    'extract': 'brewExtract',
    'extract_steep': 'brewExtractSteep',
    'partial_mash': 'brewPartialMash',
    'all_grain': 'brewAllGrain',
  },
  'layout': {
    'kettle': 'brewKettle',
    'bag': 'brewBag',
    'top_return': 'brewTopReturn',
    'malt_pipe': 'brewMaltPipe',
    'separate_tun': 'brewSeparateTun',
    'two_vessel': 'brewTwoVessel',
    'three_vessel': 'brewThreeVessel',
  },
  'heat': {
    'manual': 'brewManualHeat',
    'direct': 'brewDirect',
    'herms': 'brewHerms',
    'rims': 'brewRims',
  },
  'sparge': {
    'none': 'brewNoneSparge',
    'basket_rinse': 'brewBasketRinse',
    'batch': 'brewBatch',
    'fly': 'brewFly',
  },
  'mash': {
    'none': 'brewNoneMash',
    'infusion': 'brewInfusion',
    'step': 'brewStep',
    'decoction': 'brewDecoction',
  },
};
const brewProcesses = {
  'cider_juice': 'brewCider',
  'white_wine': 'brewWhiteWine',
  'red_wine': 'brewRedWine',
  'traditional_mead': 'brewMead',
};
const brewEquipmentKinds = {
  'kettle': 'brewKettle',
  'mash_tun': 'brewMashTun',
  'hot_liquor_tank': 'brewHotLiquorTank',
  'pump': 'brewPump',
  'chiller': 'brewChiller',
  'fermenter': 'brewFermenter',
  'bag': 'brewBag',
  'other': 'brewOther',
};
const brewReadingUnits = {
  'celsius': 'brewCelsius',
  'fahrenheit': 'brewFahrenheit',
  'sg': 'brewSg',
  'plato': 'brewPlato',
  'brix': 'brewBrix',
  'ml': 'brewMl',
  'l': 'brewL',
};

Object? brewEquipmentIdentity(Json? value) => value == null
    ? null
    : (
        value['name'],
        value['kind'],
        value['model'],
        value['notes'],
        (value['capacity'] as Json?)?['amount'],
        (value['capacity'] as Json?)?['unit'],
      );

String brewLocalized(Json value, String language) =>
    object(value['translations'])[language] as String? ??
    value['name'] as String;
Json brewStage(String id, String note) => {
  'id': id,
  'origin': {'kind': 'custom', 'note': note},
  'name': '',
  'instruction': '',
  'dependencies': <String>[],
  'active_seconds': 0,
  'passive_seconds': 0,
  'passive_min_seconds': 0,
  'passive_max_seconds': 0,
  'equipment': <String>[],
  'lead_seconds': 0,
  'repeat': 1,
  'wait_between_seconds': 0,
  'uncertainty': '',
  'start_after_work': false,
};
Json brewPhase(String id, String note) => {
  'kind': 'custom',
  'timing_known': false,
  'temperature': null,
  'volume': null,
  'stage': brewStage(id, note),
};
Json seedBrew(Json catalogue, Json template, String language, String note) {
  final phases = <Json>[];
  for (final key in (template['phases'] as List).cast<String>()) {
    final source = object(object(catalogue['phases'])[key]);
    final phase = brewPhase('phase-${phases.length + 1}', note),
        stage = object(phase['stage']);
    phase['kind'] = source['kind'];
    stage['name'] = brewLocalized(source, language);
    stage['instruction'] =
        object(source['instruction_translations'])[language] ??
        source['instruction'];
    stage['dependencies'] = phases.isEmpty
        ? <String>[]
        : [object(phases.last['stage'])['id'] as String];
    phases.add(phase);
  }
  final beer = template['beverage'] == 'beer';
  return {
    'beverage': template['beverage'],
    'title': '',
    'language': language,
    'volume': {'amount': '', 'unit': 'l'},
    'template': {
      'id': template['id'],
      'version': template['version'],
      'sources': objects(catalogue['sources'])
          .where((s) => (template['sources'] as List).contains(s['id']))
          .map(clone)
          .toList(),
    },
    'method': beer
        ? {
            'strategy': template['strategy'],
            'layout': template['layout'],
            'heat': template['heat'],
            'sparge': template['sparge'],
            'mash': ['all_grain', 'partial_mash'].contains(template['strategy'])
                ? 'infusion'
                : 'none',
          }
        : {'process': template['process']},
    'phases': phases,
    'ingredients': <Json>[],
    'notes': '',
  };
}
