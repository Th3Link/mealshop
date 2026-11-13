import '../connection/nextcloud.dart';

class Home {
  Home(Json value)
    : id = value['id'] as String,
      name = value['name'] as String,
      role = value['role'] as String {
    if (!RegExp(r'^home-[1-9][0-9]*$').hasMatch(id) ||
        !['owner', 'editor', 'reader'].contains(role)) {
      throw const ApiFailure('invalid_response');
    }
  }
  final String id, name, role;
  bool get writable => role != 'reader';
  Json toJson() => {'id': id, 'name': name, 'role': role};
}

class Shop {
  Shop(Json value) : id = value['id'] as String, name = value['name'] as String;
  final String id, name;
}

class IngredientGroup {
  IngredientGroup(Json value)
    : id = value['id'] as String,
      name = value['name'] as String,
      translations = Map<String, String>.from(object(value['translations']));
  final String id, name;
  final Map<String, String> translations;
  String label(String language) => translations[language] ?? name;
}

class ShoppingItem {
  ShoppingItem(Json value)
    : key = value['key'] as String,
      name = value['name'] as String,
      translations = Map<String, String>.from(object(value['translations'])),
      unit = value['unit'] as String,
      remaining = value['remaining'] as String,
      purchased = value['purchased'] as String,
      reserved = value['reserved'] as String? ?? '0',
      used = value['used'] as String? ?? '0',
      shopId = value['shop_id'] as String?,
      ingredientId = value['ingredient_id'] as String?,
      contributions = (value['contributions'] as List).map(object).toList();
  final String key, name, unit, remaining, purchased, reserved, used;
  final String? shopId, ingredientId;
  final Map<String, String> translations;
  final List<Json> contributions;
  String label(String language) => translations[language] ?? name;
}

class Shopping {
  Shopping(this.json)
    : revision = json['revision'] as int,
      items = (json['items'] as List)
          .map((v) => ShoppingItem(object(v)))
          .toList(),
      shops = (object(json['shops'])['entries'] as List)
          .map((v) => Shop(object(v)))
          .toList(),
      products = object(json['products']),
      sources = (json['sources'] as List).map(object).toList() {
    if (revision < 0 || items.length > 10000) {
      throw const ApiFailure('invalid_response');
    }
  }
  final Json json, products;
  final int revision;
  final List<ShoppingItem> items;
  final List<Shop> shops;
  final List<Json> sources;
}

class Pending {
  Pending(
    this.id,
    this.issuedAt,
    this.command, {
    this.expected,
    this.error,
    this.attempted = false,
    this.original,
  });
  factory Pending.parse(Json value) => Pending(
    value['id'] as String,
    value['issued_at'] as int,
    object(value['command']),
    expected: value['expected'] as int?,
    error: value['error'] as String?,
    attempted: value['attempted'] as bool,
    original: value['original'] as String?,
  );
  final String id;
  final int issuedAt;
  final Json command;
  final String? original;
  int? expected;
  bool attempted;
  String? error;
  bool get uncertain =>
      attempted && (error == null || error == 'connection_failed');
  Json toJson() => {
    'id': id,
    'issued_at': issuedAt,
    'command': command,
    'expected': expected,
    'error': error,
    'attempted': attempted,
    'original': original,
  };
}
