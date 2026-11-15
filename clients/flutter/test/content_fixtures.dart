import 'dart:convert';

import 'package:http/http.dart' as http;
import 'package:http/testing.dart';
import 'package:mealshop/connection/nextcloud.dart';
import 'package:mealshop/shopping/controller.dart';

import 'fixtures.dart';

Json ingredient(
  String id,
  String name, {
  String? german,
  bool group = false,
  String? parent,
}) => {
  'id': id,
  'name': name,
  'aliases': ['synthetic alias'],
  'form': 'Synthetic declared form',
  'translations': {
    'de': {'name': german ?? name, 'aliases': <String>[]},
  },
  'organization': {
    'is_group': group,
    'group_id': parent,
    'diet': 'unknown',
    'origin': 'unknown',
    'production': 'unknown',
    'nutrition': {'mode': 'reference'},
  },
  'source': {
    'name': 'Synthetic reference',
    'url': 'https://example.org/source',
    'record_id': 'synthetic',
    'version': '1',
    'license': 'CC0-1.0',
  },
  'tags': <String>[],
  'components': <String>[],
  'declared_allergens': <String>[],
  'archived': false,
  'favorite': false,
  'nutrients': <String, dynamic>{},
  'constituents': <String, dynamic>{},
  'nutrition_estimates': <Json>[],
};
Json catalogue() => {
  'revision': 0,
  'catalogue_version': 'synthetic',
  'language': 'en',
  'ingredients': [
    ingredient('oats', 'Oats', german: 'Haferflocken'),
    ingredient('egg', 'Egg', german: 'Ei'),
  ],
  'nutrient_definitions': [
    {'id': 'protein', 'label': 'Protein', 'unit': 'g', 'group': 'macro'},
  ],
};
Json profiles() => {'schema_version': 1, 'revision': 0, 'members': <Json>[]};
Json nutrition() => {
  'nutrients': [
    {
      'id': 'protein',
      'label': 'Protein',
      'group': 'macro',
      'unit': 'g',
      'status': 'complete',
      'per_serving': '2.500000',
      'total': '5',
      'known_per_serving': '2.5',
      'known_total': '5',
      'missing_ingredients': <String>[],
      'uncertainty': <String>[],
      'range': {
        'status': 'complete',
        'total': {'lower': '4', 'upper': '6'},
        'per_serving': {'lower': '2', 'upper': '3'},
        'estimated_ingredients': ['oats'],
      },
    },
  ],
  'ingredients': <Json>[],
  'constituents': <Json>[],
  'warning_codes': <String>[],
};
Json recipeRecord() => {
  'id': 'recipe-00000001',
  'revision': 1,
  'recipe': {
    'title': 'Synthetic porridge',
    'language': 'en',
    'author': '',
    'servings': 2,
    'preparation': 'as_recorded',
    'parts': <Json>[],
    'lines': [
      {
        'id': 'line-1',
        'ingredient_id': 'oats',
        'grams': '200.000001',
        'quantity': {'amount': '200.000001', 'unit': 'g'},
      },
    ],
    'steps': [
      {
        'instruction': 'Mix carefully',
        'active_seconds': 60,
        'passive_seconds': 0,
        'ingredient_line_ids': ['line-1'],
      },
    ],
  },
  'nutrition': nutrition(),
  'display_lines': [
    {
      'ingredient_id': 'oats',
      'name': 'Oats',
      'translations': {
        'de': {'name': 'Haferflocken'},
      },
      'amount': '200.000001',
      'unit': 'g',
    },
  ],
};

class ContentFixture {
  final vault = MemoryVault();
  final List<http.Request> requests = [];
  Future<http.Response?> Function(http.Request)? respond;
  late final ShoppingController controller;
  Json people = profiles(), foods = catalogue();
  Json shared = {'items': <Json>[], 'total': 0, 'page': 1, 'per_page': 50};
  bool activityDenied = false;
  Json activity = {
    'generation': 1,
    'access_revision': 1,
    'revision': 0,
    'can_edit': true,
    'energy': null,
    'total': 0,
    'entries': <Json>[],
  };
  Future<void> start({String language = 'en', String role = 'owner'}) async {
    controller = ShoppingController(
      Nextcloud(
        client: MockClient((request) async {
          requests.add(request);
          final overridden = await respond?.call(request);
          if (overridden != null) return overridden;
          final path = request.url.path;
          Object value;
          if (path.endsWith('/api/households')) {
            value = membership(role: role);
          } else if (path.endsWith('/shopping/mobile')) {
            value = capability(0, role: role);
          } else if (path.endsWith('/profiles')) {
            if (request.method == 'PUT') {
              people = {
                ...object(jsonDecode(request.body)),
                'schema_version': 1,
                'revision': (people['revision'] as int) + 1,
              }..remove('expected_revision');
            }
            value = people;
          } else if (path.endsWith('/api/shared')) {
            value = shared;
          } else if (path.endsWith('/allergens')) {
            value = {'groups': <Json>[]};
          } else if (path.endsWith('/ingredients')) {
            value = foods;
          } else if (path.endsWith('/conversions')) {
            value = {'entries': <Json>[]};
          } else if (path.contains('/people/') && path.endsWith('/activity')) {
            if (activityDenied) {
              return http.Response(
                jsonEncode({'code': 'person_activity_access_required'}),
                403,
              );
            }
            value = activity;
          } else if (path.contains('/activity/activity-')) {
            value = activity;
          } else if (path.endsWith('/recipes')) {
            value = {'recipes': <Json>[]};
          } else if (path.contains('/recipes/')) {
            value = recipeRecord();
          } else {
            value = capability(0, role: role);
          }
          return http.Response(
            jsonEncode(value),
            200,
            headers: {'content-type': 'application/json; charset=utf-8'},
          );
        }),
      ),
      vault,
    );
    await controller.initialize(deviceLanguage: language);
    await controller.signIn(
      Credential(Server('https://cloud.example.org'), 'alice', 'synthetic'),
    );
  }
}
