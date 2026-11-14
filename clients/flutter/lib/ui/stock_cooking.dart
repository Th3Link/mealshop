import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../planning/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'plan_common.dart';
import 'stock_review.dart';
import 'item_details.dart' show originLabel;

class CookingStockEditor extends StatefulWidget {
  const CookingStockEditor(this.api, this.run, {super.key});
  final HouseholdApi api;
  final Json run;
  @override
  State<CookingStockEditor> createState() => _CookingStockEditorState();
}

class _CookingStockEditorState extends State<CookingStockEditor> {
  final selected = <String>{};
  String date = isoDay(DateTime.now());
  bool unreserved = false, busy = false, saved = false;
  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => !saved && (selected.isNotEmpty || unreserved),
    child: Scaffold(
      appBar: AppBar(title: Text(context.t('record_cooking_stock'))),
      body: Resource<Json>(
        load: () async {
          final inventory = await widget.api.household(
            '/inventory?limit=500&run_id=${Uri.encodeComponent(widget.run['id'] as String)}',
          );
          final shopping = await widget.api.household('/shopping');
          if (inventory['revision'] != shopping['revision']) {
            throw const ApiFailure('shopping_revision_conflict', 409);
          }
          inventory['sources'] = shopping['sources'];
          return inventory;
        },
        builder: (context, inventory, reload) {
          final selection = object(inventory['cooking_selection']);
          return ListView(
            padding: const EdgeInsets.all(16),
            children: [
              Text(
                widget.run['name'] as String,
                style: Theme.of(context).textTheme.titleLarge,
              ),
              Note(context.t('cooking_stock_note')),
              for (final r in objects(selection['requirements']))
                ListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(object(r['item'])['name'] as String),
                  subtitle: Text(
                    '${exactNumber(context, r['amount'] as String)} ${object(r['item'])['unit']}',
                  ),
                ),
              for (final id in (selection['source_ids'] as List).cast<String>())
                CheckboxListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(
                    originLabel(
                      context,
                      object(
                        objects(inventory['sources'])
                            .firstWhere((s) => s['id'] == id)['origin'],
                      ),
                    ),
                  ),
                  value: selected.contains(id),
                  onChanged: busy
                      ? null
                      : (v) => setState(
                          () => v == true
                              ? selected.add(id)
                              : selected.remove(id),
                        ),
                ),
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(context.t('include_unreserved_stock')),
                value: unreserved,
                onChanged: busy ? null : (v) => setState(() => unreserved = v),
              ),
              DayButton('use_on', date, (v) => setState(() => date = v)),
              FilledButton(
                onPressed: busy || !widget.api.writable
                    ? null
                    : () async {
                        setState(() => busy = true);
                        try {
                          final data = await PlanningData.load(widget.api);
                          if (!context.mounted) return;
                          if (data.stock['revision'] != inventory['revision']) {
                            throw const ApiFailure(
                              'shopping_revision_conflict',
                              409,
                            );
                          }
                          if (await reviewStock(context, widget.api, data, {
                                'operation': 'consume',
                                'run_id': widget.run['id'],
                                'source_ids': selected.toList(),
                                'include_unreserved': unreserved,
                                'as_of': date,
                              }) &&
                              context.mounted) {
                            saved = true;
                            Navigator.pop(context);
                          }
                        } catch (e) {
                          if (context.mounted) await showFailure(context, e);
                        } finally {
                          if (mounted) setState(() => busy = false);
                        }
                      },
                child: Text(context.t('review_stock')),
              ),
            ],
          );
        },
      ),
    ),
  );
}
