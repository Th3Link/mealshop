import 'dart:convert';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../planning/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'plan_shopping.dart';

Future<bool> reviewStock(
  BuildContext context,
  HouseholdApi api,
  PlanningData data,
  Json command,
) async {
  final input = <String, dynamic>{
    'expected_revision': data.stock['revision'],
    'idempotency_key': newId('stock-'),
    'reviewed': false,
    'command': clone(command),
  };
  final preview = await api.household(
    '/inventory/preview',
    method: 'POST',
    body: input,
  );
  if (!context.mounted) return false;
  return await openPage<bool>(context, StockReview(api, input, preview)) ==
      true;
}

class StockReview extends StatefulWidget {
  const StockReview(this.api, this.input, this.preview, {super.key});
  final HouseholdApi api;
  final Json input, preview;
  @override
  State<StockReview> createState() => _StockReviewState();
}

class _StockReviewState extends State<StockReview> {
  bool checked = false,
      busy = false,
      attempted = false,
      saved = false,
      uncertain = false;
  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => attempted && !saved && uncertain,
    child: Scaffold(
      appBar: AppBar(title: Text(context.t('review_stock'))),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Note(context.t('stock_review_note')),
          for (final change in objects(widget.preview['changes']))
            Builder(
              builder: (context) {
                final row = object(change['after']);
                final before = change['before'] as Json?;
                return ListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(object(row['binding'])['name'] as String),
                  subtitle: Text(
                    '${context.t('available')}: ${exactNumber(context, before?['available'] as String? ?? '0')} → ${exactNumber(context, row['available'] as String)} ${row['unit']}\n${context.t('reserved')}: ${exactNumber(context, row['reserved'] as String)}',
                  ),
                );
              },
            ),
          ..._prepared(context),
          for (final change in objects(widget.preview['shopping']))
            ShoppingChange(change),
          for (final offer in objects(widget.preview['offers']))
            ListTile(
              contentPadding: EdgeInsets.zero,
              title: Text(object(offer['binding'])['name'] as String),
              subtitle: Text(
                '${exactNumber(context, offer['price'] as String)} ${offer['currency']} · ${offer['valid_from']} – ${offer['valid_until']}',
              ),
            ),
          for (final movement in objects(widget.preview['movements'])) ...[
            Text(context.t(object(movement['kind'])['kind'] as String)),
            for (final missing in objects(
              object(movement['kind'])['unbooked'] ?? [],
            ))
              Note(
                '${context.t('unbooked_stock')}: ${object(missing['item'])['name']} · ${exactNumber(context, missing['amount'] as String)} ${object(missing['item'])['unit']}',
                warning: true,
              ),
          ],
          if (uncertain) Note(context.t('stock_uncertain'), warning: true),
          CheckboxListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(context.t('reviewed_stock')),
            value: checked,
            onChanged: busy || attempted
                ? null
                : (v) => setState(() => checked = v == true),
          ),
          FilledButton(
            onPressed: busy || !checked || !widget.api.writable
                ? null
                : () async {
                    setState(() {
                      busy = true;
                      attempted = true;
                      uncertain = true;
                    });
                    try {
                      await widget.api.household(
                        '/inventory',
                        method: 'POST',
                        body: {...widget.input, 'reviewed': true},
                      );
                      saved = true;
                      if (context.mounted) Navigator.pop(context, true);
                    } catch (e) {
                      if (e is ApiFailure &&
                          (e.status == 403 ||
                              e.status == 409 ||
                              e.status == 422)) {
                        uncertain = false;
                      }
                      if (context.mounted) await showFailure(context, e);
                    } finally {
                      if (mounted) setState(() => busy = false);
                    }
                  },
            child: Text(
              context.t(attempted ? 'retry_confirmed_stock' : 'apply_reviewed'),
            ),
          ),
          if (busy) const LinearProgressIndicator(),
        ],
      ),
    ),
  );
  List<Widget> _prepared(BuildContext context) {
    final before = objects(widget.preview['prepared_before'] ?? []);
    return [
      for (final r in objects(widget.preview['prepared_after'] ?? []))
        if (!before.any((old) => jsonEncode(old) == jsonEncode(r)))
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(object(r['batch'])['title'] as String),
            subtitle: Text(
              '${context.t('actual_yield')}: ${object(r['batch'])['original_portions']} ${context.t('servings')}\n'
              '${context.t('reserved')}: ${r['held']} · ${context.t('consumed_portions')}: ${r['used']} · ${context.t('available')}: ${r['available']}\n'
              '${object(object(r['batch'])['storage'])['location']} · ${context.t('storage_${object(object(r['batch'])['storage'])['condition']}')}',
            ),
          ),
    ];
  }
}
