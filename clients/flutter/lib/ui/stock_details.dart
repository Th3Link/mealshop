import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../planning/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'plan_common.dart';
import 'stock_editor.dart';
import 'stock_review.dart';

class LotDetail extends StatelessWidget {
  const LotDetail(this.api, this.id, {super.key});
  final HouseholdApi api;
  final String id;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('edit_stock'))),
    body: Resource<PlanningData>(
      sensitive: true,
      pollInterval: null,
      load: () => PlanningData.load(api),
      builder: (context, data, reload) {
        final lot = objects(data.stock['lots'])
            .where((l) => l['id'] == id)
            .firstOrNull;
        if (lot == null) {
          return const EmptyContent(
            icon: Icons.kitchen_outlined,
            title: 'inventory_lot_not_found',
          );
        }
        Future<void> change(Json c) => runAction(context, () async {
          await reviewStock(context, api, data, c);
          await reload();
        });
        return ListView(
          padding: const EdgeInsets.all(16),
          children: [
            Text(
              object(lot['binding'])['name'] as String,
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            Text(
              '${context.t('available')}: ${exactNumber(context, lot['available'] as String)} ${lot['unit']}\n${context.t('reserved')}: ${exactNumber(context, lot['reserved'] as String)} ${lot['unit']}',
            ),
            Text(
              '${object(lot['metadata'])['location']}\n${context.t('stock_batch')}: ${object(lot['metadata'])['batch']}\n${context.t('declared_use_by')}: ${object(lot['metadata'])['expiry'] ?? context.t('no_declared_date')}',
            ),
            Note(context.t('stock_original_note')),
            Text(
              '${exactNumber(context, object(lot['original_quantity'])['amount'] as String)} ${object(lot['original_quantity'])['unit']}',
            ),
            for (final a in objects(
              data.stock['allocations'],
            ).where((a) => a['lot_id'] == id && a['state'] == 'held'))
              ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(
                  '${exactNumber(context, a['amount'] as String)} ${lot['unit']} · ${a['use_on']}',
                ),
                subtitle: Text(a['source_id'] as String),
                trailing: api.writable
                    ? IconButton(
                        tooltip: context.t('release_stock'),
                        icon: const Icon(Icons.link_off),
                        onPressed: () => change({
                          'operation': 'release',
                          'allocation_ids': [a['id']],
                        }),
                      )
                    : null,
              ),
            if (api.writable) ...[
              FilledButton(
                onPressed: lot['archived'] == true
                    ? null
                    : () async {
                        await openPage<void>(
                          context,
                          StockEditor(api, data, lot: lot),
                        );
                        await reload();
                      },
                child: Text(context.t('edit_stock')),
              ),
              TextButton(
                onPressed: () =>
                    change({'operation': 'archive_lot', 'lot_id': id}),
                child: Text(context.t('archive')),
              ),
            ],
          ],
        );
      },
    ),
  );
}

class OfferDetail extends StatefulWidget {
  const OfferDetail(this.api, this.id, {super.key});
  final HouseholdApi api;
  final String id;
  @override
  State<OfferDetail> createState() => _OfferDetailState();
}

class _OfferDetailState extends State<OfferDetail> {
  String packages = '1', date = isoDay(DateTime.now());
  List<String>? people;
  final form = GlobalKey<FormState>();
  bool busy = false;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('offer'))),
    body: Resource<PlanningData>(
      load: () => PlanningData.load(widget.api),
      builder: (context, data, reload) {
        final row = objects(data.stock['offers'])
            .where((r) => object(r['offer'])['id'] == widget.id)
            .firstOrNull;
        if (row == null) {
          return const EmptyContent(
            icon: Icons.local_offer_outlined,
            title: 'inventory_offer_not_found',
          );
        }
        final offer = object(row['offer']);
        people ??= data.people.map((p) => p['id'] as String).toList();
        Future<void> change(Json c) async {
          setState(() => busy = true);
          try {
            await reviewStock(context, widget.api, data, c);
            await reload();
          } catch (e) {
            if (context.mounted) await showFailure(context, e);
          } finally {
            if (mounted) setState(() => busy = false);
          }
        }

        return Form(
          key: form,
          child: ListView(
            padding: const EdgeInsets.all(16),
            children: [
              Text(
                object(offer['binding'])['name'] as String,
                style: Theme.of(context).textTheme.headlineSmall,
              ),
              Text(
                '${exactNumber(context, offer['price'] as String)} ${offer['currency']} · ${exactNumber(context, object(offer['quantity'])['amount'] as String)} ${object(offer['quantity'])['unit']}\n${offer['shop_name']} · ${offer['valid_from']} – ${offer['valid_until']}',
              ),
              SelectableText(
                '${offer['source']}\n${offer['source_url'] ?? ''}',
              ),
              Note(context.t('dietary_review_note'), warning: true),
              if (widget.api.writable && offer['archived'] != true) ...[
                EditText(
                  'packages',
                  packages,
                  (v) => packages = v,
                  required: true,
                  integer: true,
                ),
                DayButton(
                  'shopping_date',
                  date,
                  (v) => setState(() => date = v),
                ),
                AttendeeChoices(
                  data,
                  people!,
                  (v) => setState(() => people = v),
                ),
                FilledButton(
                  onPressed: busy
                      ? null
                      : () async {
                          if (form.currentState!.validate()) {
                            await change({
                              'operation': 'add_offer',
                              'offer_id': widget.id,
                              'packages': int.parse(packages),
                              'as_of': date,
                              'member_ids': people,
                            });
                          }
                        },
                  child: Text(context.t('add_offer_shopping')),
                ),
                OutlinedButton(
                  onPressed: busy
                      ? null
                      : () async {
                          await openPage<void>(
                            context,
                            StockEditor(
                              widget.api,
                              data,
                              offer: true,
                              offerRecord: offer,
                            ),
                          );
                          await reload();
                        },
                  child: Text(context.t('edit')),
                ),
                TextButton(
                  onPressed: busy
                      ? null
                      : () => change({
                          'operation': 'archive_offer',
                          'offer_id': widget.id,
                        }),
                  child: Text(context.t('archive')),
                ),
              ],
            ],
          ),
        );
      },
    ),
  );
}

class StockHistory extends StatefulWidget {
  const StockHistory(this.api, {super.key});
  final HouseholdApi api;
  @override
  State<StockHistory> createState() => _StockHistoryState();
}

class _StockHistoryState extends State<StockHistory> {
  int offset = 0, version = 0;
  bool prepared = false;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('stock_history'))),
    body: Resource<Json>(
      key: ValueKey((offset, version, prepared)),
      sensitive: true,
      pollInterval: null,
      load: () => widget.api.household(
        '/inventory?limit=1&history_limit=50&history_offset=$offset',
      ),
      builder: (context, data, reload) {
        final rows = objects(
          data[prepared ? 'prepared_history' : 'movements'] ?? [],
        );
        final total =
            data[prepared ? 'prepared_event_total' : 'movement_total']
                as int? ??
            rows.length;
        return ListView(
          padding: const EdgeInsets.all(16),
          children: [
            Choice(
              'stock_view',
              prepared ? 'prepared' : 'lots',
              {
                'prepared': context.t('stock_prepared'),
                'lots': context.t('stock_lots'),
              },
              (v) => setState(() {
                prepared = v == 'prepared';
                offset = 0;
              }),
            ),
            if (rows.isEmpty) Note(context.t('nothing_yet')),
            for (final row in rows)
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(
                      context.t(
                        prepared
                            ? object(row['command'])['operation'] as String
                            : object(row['kind'])['kind'] as String,
                      ),
                    ),
                    subtitle: Text(
                      '${row['id']} · ${row['actor']}\n${DateTime.fromMillisecondsSinceEpoch((row['recorded_at'] as int) * 1000).toLocal()}',
                    ),
                  ),
                  if (prepared)
                    Text(_description(context, object(row['command'])))
                  else ...[
                    for (final e in objects(row['entries']))
                      Text(
                        '${e['direction'] == 'in' ? '+' : '−'}${exactNumber(context, e['amount'] as String)} · ${e['lot_id']}',
                      ),
                    if (widget.api.writable &&
                        row['reversed'] != true &&
                        object(row['kind'])['kind'] != 'reversal')
                      TextButton(
                        onPressed: () => runAction(context, () async {
                          final reason = await askName(
                            context,
                            'correction_reason',
                          );
                          if (reason == null) return;
                          final current = await PlanningData.load(widget.api);
                          if (!context.mounted) return;
                          await reviewStock(context, widget.api, current, {
                            'operation': 'reverse',
                            'movement_id': row['id'],
                            'reason': reason,
                          });
                          await reload();
                        }),
                        child: Text(context.t('reverse_booking')),
                      ),
                  ],
                  const Divider(),
                ],
              ),
            Text('${offset + 1}–${offset + rows.length} / $total'),
            Wrap(
              spacing: 8,
              children: [
                OutlinedButton(
                  onPressed: offset == 0
                      ? null
                      : () => setState(
                          () => offset = (offset - 50).clamp(0, 100000),
                        ),
                  child: Text(context.t('previous')),
                ),
                OutlinedButton(
                  onPressed: offset + rows.length >= total
                      ? null
                      : () => setState(() => offset += 50),
                  child: Text(context.t('next')),
                ),
              ],
            ),
          ],
        );
      },
    ),
  );
  String _description(BuildContext context, Json c) => [
    if (c['portions'] != null) '${c['portions']} ${context.t('servings')}',
    if (c['date'] != null) '${c['date']} · ${context.t(c['meal'] as String)}',
    if (c['storage'] != null)
      '${object(c['storage'])['location']} · ${context.t('storage_${object(c['storage'])['condition']}')}',
    if (c['input'] != null)
      '${object(c['input'])['portions']} ${context.t('servings')} · ${object(c['input'])['produced_on']}',
    if (c['reason'] != null) c['reason'] as String,
  ].join('\n');
}
