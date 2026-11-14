import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../planning/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';

class PlanShoppingScreen extends StatefulWidget {
  const PlanShoppingScreen(this.api, this.data, {super.key, this.start});
  final HouseholdApi api;
  final PlanningData data;
  final String? start;
  @override
  State<PlanShoppingScreen> createState() => _PlanShoppingScreenState();
}

class _PlanShoppingScreenState extends State<PlanShoppingScreen> {
  late final selected = widget.data.meals
      .where((p) {
        final current =
            widget.api.controller.shopping?.sources.any((s) {
              final origin = object(s['origin']);
              return s['active'] == true &&
                  origin['kind'] == 'meal' &&
                  origin['date'] == p['date'] &&
                  origin['meal'] == p['meal'];
            }) ??
            false;
        final start = widget.start ?? isoDay(weekStart(DateTime.now()));
        final end = isoDay(addDays(civilDay(start), 6));
        final prepared = widget.data.prepared.any((r) {
          final b = object(r['batch']),
              source = object(
                object(r['batch'])['source'] ?? <String, dynamic>{},
              );
          return b['reversed'] != true &&
              source['date'] == p['date'] &&
              source['meal'] == p['meal'];
        });
        return current ||
            !prepared &&
                eatingSlots(p).any(
                  (a) =>
                      (a['date'] as String).compareTo(start) >= 0 &&
                      (a['date'] as String).compareTo(end) <= 0,
                );
      })
      .map((p) => '${p['date']}/${p['meal']}')
      .toSet();
  Json? preview;
  bool busy = false;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('update_plan_shopping'))),
    body: ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Note(context.t('shopping_plan_selection_note')),
        for (final p in widget.data.meals)
          CheckboxListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(widget.data.title(object(p['recipe']))),
            subtitle: Text(
              '${p['date']} · ${context.t(p['meal'] as String)} · ${p['servings']} ${context.t('servings')}',
            ),
            value: selected.contains('${p['date']}/${p['meal']}'),
            onChanged: busy
                ? null
                : (v) => setState(() {
                    preview = null;
                    v == true
                        ? selected.add('${p['date']}/${p['meal']}')
                        : selected.remove('${p['date']}/${p['meal']}');
                  }),
          ),
        if (preview == null)
          FilledButton(
            onPressed: busy || !widget.api.writable
                ? null
                : () async {
                    setState(() => busy = true);
                    try {
                      final shopping = await widget.api.household('/shopping');
                      final result = await widget.api.household(
                        '/shopping/plan-preview',
                        method: 'POST',
                        body: {
                          'expected_revision': shopping['revision'],
                          'plan_revision': widget.data.plan['revision'],
                          'slots': [
                            for (final p in widget.data.meals)
                              if (selected.contains(
                                '${p['date']}/${p['meal']}',
                              ))
                                {'date': p['date'], 'meal': p['meal']},
                          ],
                        },
                      );
                      if (mounted) setState(() => preview = result);
                    } catch (e) {
                      if (context.mounted) await showFailure(context, e);
                    } finally {
                      if (mounted) setState(() => busy = false);
                    }
                  },
            child: Text(context.t('review_shopping')),
          )
        else ...[
          for (final change in objects(preview!['changes']))
            ShoppingChange(change),
          Note(context.t('manual_purchases_preserved')),
          FilledButton(
            onPressed: busy
                ? null
                : () async {
                    setState(() => busy = true);
                    try {
                      await widget.api.household(
                        '/shopping',
                        method: 'POST',
                        body: {
                          'expected_revision': preview!['expected_revision'],
                          'command': preview!['command'],
                        },
                      );
                      await widget.api.controller.refresh();
                      if (context.mounted) Navigator.pop(context);
                    } catch (e) {
                      if (context.mounted) await showFailure(context, e);
                      if (mounted) setState(() => preview = null);
                    } finally {
                      if (mounted) setState(() => busy = false);
                    }
                  },
            child: Text(context.t('apply_reviewed')),
          ),
        ],
        if (busy) const LinearProgressIndicator(),
      ],
    ),
  );
}

class ShoppingChange extends StatelessWidget {
  const ShoppingChange(this.change, {super.key});
  final Json change;
  @override
  Widget build(BuildContext context) {
    final before = change['before'] as Json?, after = change['after'] as Json?;
    final row = after ?? before!;
    final language = Localizations.localeOf(context).languageCode;
    final name =
        object(row['translations'] ?? <String, dynamic>{})[language]
            as String? ??
        row['name'] as String;
    return ListTile(
      contentPadding: EdgeInsets.zero,
      title: Text(name),
      subtitle: Text(
        '${context.t('to_buy')}: ${decimal(before?['remaining'] as String? ?? '0', language)} → ${decimal(after?['remaining'] as String? ?? '0', language)} ${row['unit']}\n'
        '${context.t('purchased')}: ${decimal(after?['purchased'] as String? ?? before?['purchased'] as String? ?? '0', language)} ${row['unit']}',
      ),
    );
  }
}
