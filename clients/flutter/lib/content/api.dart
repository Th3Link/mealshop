import 'dart:convert';
import 'dart:math';
import 'dart:typed_data';

import '../connection/nextcloud.dart';
import '../shopping/controller.dart';

Json clone(Json value) => object(jsonDecode(jsonEncode(value)));
List<Json> objects(Object? value) => (value as List).map(object).toList();
String newId(String prefix) {
  final random = Random.secure();
  return '$prefix${List.generate(24, (_) => random.nextInt(256).toRadixString(16).padLeft(2, '0')).join()}';
}

String decimal(String value, String language) {
  if (!RegExp(r'^-?\d+(?:\.\d{1,12})?$').hasMatch(value)) {
    throw const ApiFailure('invalid_response');
  }
  var result = value.contains('.')
      ? value.replaceFirst(RegExp(r'0+$'), '').replaceFirst(RegExp(r'\.$'), '')
      : value;
  if (language == 'de') result = result.replaceAll('.', ',');
  return result;
}

String inputDecimal(String canonical, String language) =>
    language == 'de' ? canonical.replaceAll('.', ',') : canonical;
String exactInput(String value, String language, {bool signed = false}) {
  final text = value.trim();
  final separator = language == 'de' ? ',' : '.';
  final pattern =
      '^${signed ? "-?" : ""}[0-9]{1,12}(?:${RegExp.escape(separator)}[0-9]{1,6})?\$';
  if (!RegExp(pattern).hasMatch(text)) {
    throw const ApiFailure('quantity_invalid');
  }
  return text.replaceAll(separator, '.');
}

/// Captures identity and household once. A disposed route cannot retarget a request.
class HouseholdApi {
  HouseholdApi(this.controller)
    : credential = controller.credential!,
      home = controller.selected;
  final ShoppingController controller;
  final Credential credential;
  final String? home;
  bool get current =>
      identical(controller.credential, credential) &&
      controller.selected == home &&
      !controller.storageBlocked;
  bool get writable => current && controller.home?.writable == true;
  bool get owner => current && controller.home?.role == 'owner';
  String get root {
    if (home == null) throw const ApiFailure('no_home');
    return '/api/households/$home';
  }

  String get domain => '$root/domain/api';
  Future<Json> call(String path, {String method = 'GET', Object? body}) async {
    if (!current) throw const ApiFailure('access_removed', 403);
    final result = await controller.api.api(
      credential,
      path,
      method: method,
      body: body,
    );
    if (!current) throw const ApiFailure('access_removed', 403);
    return result;
  }

  Future<Json> household(String path, {String method = 'GET', Object? body}) =>
      call('$domain/household$path', method: method, body: body);
  Future<Json> recipes([
    String suffix = '',
    String method = 'GET',
    Object? body,
  ]) => call('$domain/recipes$suffix', method: method, body: body);
  Future<Uint8List> image(String path) async {
    if (!current) throw const ApiFailure('access_removed', 403);
    final result = await controller.api.bytes(credential, path);
    if (!current) throw const ApiFailure('access_removed', 403);
    return result;
  }

  Future<String> upload(Uint8List bytes, String mime) async {
    if (!writable) throw const ApiFailure('household_read_only', 403);
    final result = await controller.api.upload(
      credential,
      '$domain/household/images',
      bytes,
      mime,
    );
    if (!current) throw const ApiFailure('access_removed', 403);
    return result['id'] as String;
  }
}

class Catalogue {
  Catalogue(Json value)
    : revision = value['revision'] as int,
      items = objects(value['ingredients']),
      definitions = objects(value['nutrient_definitions']);
  final int revision;
  final List<Json> items, definitions;
  Json? find(String id) => items.where((i) => i['id'] == id).firstOrNull;
  String name(Json item, String language) =>
      (item['translations'] as Json?)?[language]?['name'] as String? ??
      item['name'] as String;
  List<Json> search(String query, String language, {bool archived = false}) {
    final words = query.toLowerCase().trim().split(RegExp(r'\s+'));
    final found = items
        .where(
          (item) =>
              (archived || item['archived'] != true) &&
              words.every(
                (word) =>
                    '${item['name']} ${item['aliases']} ${item['translations']}'
                        .toLowerCase()
                        .contains(word),
              ),
        )
        .toList();
    found.sort((a, b) {
      final favorite =
          (b['favorite'] == true ? 1 : 0) - (a['favorite'] == true ? 1 : 0);
      return favorite != 0
          ? favorite
          : name(a, language).compareTo(name(b, language));
    });
    return found;
  }
}
