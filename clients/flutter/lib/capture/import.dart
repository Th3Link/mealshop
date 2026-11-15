import 'dart:convert';

import '../connection/nextcloud.dart';
import '../content/api.dart';

/// No fractions, ranges, grouping separators or cup-to-gram guesses.
Json? metricSuggestion(String text, String? language) {
  final match = RegExp(
    r'^\s*(\d+(?:[.,]\d{1,6})?)\s*(kg|g)\b',
    caseSensitive: false,
  ).firstMatch(text);
  if (match == null) return null;
  final amount = match[1]!;
  final lang = language?.toLowerCase().split(RegExp('[-_]')).first;
  if (amount.contains(',') && lang != 'de' ||
      amount.contains('.') && lang != 'en') {
    return null;
  }
  return {
    'amount': amount.replaceAll(',', '.'),
    'unit': match[2]!.toLowerCase(),
  };
}

Json blankCandidate() => {
  'title': null,
  'author': null,
  'language': null,
  'yield_text': null,
  'ingredients': <String>[],
  'steps': <Json>[],
  'prep_time': null,
  'cook_time': null,
  'total_time': null,
};

/// Captured API/scope, immutable submission and protected draft survive route/process loss.
class ImportSession {
  ImportSession(this.api, this.preview, this.candidate, {Json? restored}) {
    envelope = restored?['envelope'] == null
        ? null
        : clone(object(restored!['envelope']));
    draft = restored?['draft'] == null
        ? null
        : clone(object(restored!['draft']));
  }
  final HouseholdApi api;
  final Json preview;
  final int candidate;
  Json? envelope, draft;
  String? error;
  Json get original => objects(preview['candidates'])[candidate];
  bool get frozen => envelope != null;
  static const key = 'capture.v1';
  static ImportSession? restore(HouseholdApi api) {
    final saved = api.controller.draft(key);
    if (saved == null) return null;
    try {
      if (saved['version'] != 1) throw const FormatException();
      final preview = object(saved['preview']),
          candidate = saved['candidate'] as int;
      final candidates = objects(preview['candidates']);
      if (candidate < 0 ||
          candidate >= candidates.length ||
          preview['preview_id'] is! String) {
        throw const FormatException();
      }
      final result = ImportSession(api, preview, candidate, restored: saved);
      if (result.envelope != null &&
          (result.envelope!['preview_id'] != preview['preview_id'] ||
              result.envelope!['candidate'] != candidate ||
              result.envelope!['reviewed'] != true ||
              result.envelope!['idempotency_key'] is! String ||
              result.envelope!['recipe'] is! Map)) {
        throw const FormatException();
      }
      return result;
    } catch (_) {
      throw const ApiFailure('capture_draft_invalid');
    }
  }

  Future<void> retain(Json recipe) async {
    if (!api.current) throw const ApiFailure('access_removed', 403);
    draft = clone(recipe);
    final value = {
      'version': 1,
      'preview': preview,
      'candidate': candidate,
      'draft': draft,
      if (envelope != null) 'envelope': envelope,
    };
    if (utf8.encode(jsonEncode(value)).length > 98304) {
      throw const ApiFailure('cache_capacity');
    }
    await api.controller.saveDraft(key, value);
  }

  Future<String> submit(Json recipe) async {
    if (!api.writable) throw const ApiFailure('household_read_only', 403);
    if (envelope == null) {
      envelope = {
        'preview_id': preview['preview_id'],
        'candidate': candidate,
        'idempotency_key': newId('mobile-import-'),
        'reviewed': true,
        'recipe': clone(recipe),
      };
      try {
        await retain(recipe);
      } catch (_) {
        envelope = null;
        rethrow;
      }
    }
    // A timeout/5xx keeps the original key and body. Only a definite validation response unlocks.
    try {
      final result = await api.household(
        '/imports',
        method: 'POST',
        body: clone(envelope!),
      );
      await discard();
      return result['id'] as String;
    } on ApiFailure catch (e) {
      error = e.code;
      if (e.status == 422) {
        envelope = null;
        await retain(recipe);
      }
      rethrow;
    }
  }

  Future<void> discard() async {
    if (!api.current) throw const ApiFailure('access_removed', 403);
    await api.controller.saveDraft(key, null);
  }

  Json seed() =>
      draft ??
      {
        'title': original['title'] ?? '',
        'author': original['author'] ?? '',
        'language': ['de', 'en'].contains(original['language'])
            ? original['language']
            : api.controller.language,
        'servings': null,
        'parts': <Json>[],
        'lines': <Json>[],
        'steps': [
          for (final s in objects(original['steps']))
            {
              'instruction': s['text'],
              'active_seconds': 0,
              'passive_seconds': 0,
            },
        ],
        'preparation': 'as_recorded',
        'origin': preview['capture'] == null
            ? {'kind': 'imported', 'source_url': preview['requested_url']}
            : {
                'kind': 'captured',
                'source_image_id': object(preview['capture'])['image_id'],
              },
      };
}
