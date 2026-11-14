import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../planning/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'prepared_editor.dart';
import 'stock_editor.dart';
import 'stock_details.dart';
import 'stock_cooking.dart';
import 'stock_review.dart';

class StockScreen extends StatefulWidget {
  const StockScreen(this.api, {super.key});
  final HouseholdApi api;
  @override
  State<StockScreen> createState() => _StockScreenState();
}

class _StockScreenState extends State<StockScreen> {
  String view = 'prepared';
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('stock'))),
    body: Resource<PlanningData>(
      sensitive: true,
      pollInterval: null,
      load: () => PlanningData.load(widget.api),
      builder: (context, data, reload) => ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Choice('stock_view', view, {
            for (final v in ['prepared', 'lots', 'offers'])
              v: context.t('stock_$v'),
          }, (v) => setState(() => view = v)),
          if (!widget.api.writable) Note(context.t('household_read_only')),
          if (view == 'prepared') ...[
            Note(context.t('prepared_declaration_note')),
            for (final row in data.prepared)
              ListTile(
                contentPadding: EdgeInsets.zero,
                leading: const Icon(Icons.inventory_2_outlined),
                title: Text(object(row['batch'])['title'] as String),
                subtitle: Text(
                  '${context.t('available')}: ${row['available']} · ${context.t('reserved')}: ${row['held']} · ${context.t('consumed_portions')}: ${row['used']}',
                ),
                trailing: const Icon(Icons.chevron_right),
                onTap: () async {
                  await openPage<void>(
                    context,
                    PreparedDetail(
                      widget.api,
                      object(row['batch'])['id'] as String,
                    ),
                  );
                  await reload();
                },
              ),
            if (data.prepared.isEmpty) Note(context.t('no_prepared_stock')),
            if (widget.api.writable)
              FilledButton.icon(
                icon: const Icon(Icons.add),
                label: Text(context.t('record_prepared')),
                onPressed: () async {
                  await openPage<void>(
                    context,
                    PreparedEditor(widget.api, data),
                  );
                  await reload();
                },
              ),
          ],
          if (view == 'lots') ...[
            for (final lot in objects(data.stock['lots']))
              ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(object(lot['binding'])['name'] as String),
                subtitle: Text(
                  '${context.t('available')}: ${exactNumber(context, lot['available'] as String)} ${lot['unit']} · ${context.t('reserved')}: ${exactNumber(context, lot['reserved'] as String)}\n${object(lot['metadata'])['location']} · ${object(lot['metadata'])['expiry'] ?? context.t('no_declared_date')}',
                ),
                trailing: const Icon(Icons.chevron_right),
                onTap: () async {
                  await openPage<void>(
                    context,
                    LotDetail(widget.api, lot['id'] as String),
                  );
                  await reload();
                },
              ),
            if (objects(data.stock['lots']).isEmpty) Note(context.t('no_lots')),
            if (widget.api.writable) ...[
              for (final run in objects(
                data.stock['completed_runs'] ?? [],
              ).where((r) => r['booked'] != true))
                OutlinedButton(
                  onPressed: () async {
                    await openPage<void>(
                      context,
                      CookingStockEditor(widget.api, run),
                    );
                    await reload();
                  },
                  child: Text(
                    '${context.t('record_cooking_stock')} · ${run['name']}',
                  ),
                ),
              FilledButton.icon(
                icon: const Icon(Icons.add),
                label: Text(context.t('add_stock')),
                onPressed: () async {
                  await openPage<void>(context, StockEditor(widget.api, data));
                  await reload();
                },
              ),
              OutlinedButton(
                onPressed: () async {
                  await openPage<void>(
                    context,
                    StockSourceEditor(widget.api, data, receive: false),
                  );
                  await reload();
                },
                child: Text(context.t('reserve_stock')),
              ),
              OutlinedButton(
                onPressed: () async {
                  await openPage<void>(
                    context,
                    StockSourceEditor(widget.api, data, receive: true),
                  );
                  await reload();
                },
                child: Text(context.t('receive_stock')),
              ),
            ],
          ],
          if (view == 'offers') ...[
            for (final row in objects(data.stock['offers']))
              Builder(
                builder: (context) {
                  final offer = object(row['offer']);
                  return ListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(object(offer['binding'])['name'] as String),
                    subtitle: Text(
                      '${exactNumber(context, offer['price'] as String)} ${offer['currency']} · ${offer['shop_name']}\n${offer['valid_from']} – ${offer['valid_until']} · ${offer['source']}',
                    ),
                    onTap: () async {
                      await openPage<void>(
                        context,
                        OfferDetail(widget.api, offer['id'] as String),
                      );
                      await reload();
                    },
                  );
                },
              ),
            if (objects(data.stock['offers']).isEmpty)
              Note(context.t('no_offers')),
            if (widget.api.writable)
              FilledButton.icon(
                icon: const Icon(Icons.add),
                label: Text(context.t('record_offer')),
                onPressed: () async {
                  await openPage<void>(
                    context,
                    StockEditor(widget.api, data, offer: true),
                  );
                  await reload();
                },
              ),
          ],
          const SizedBox(height: 16),
          OutlinedButton.icon(
            icon: const Icon(Icons.history),
            label: Text(context.t('stock_history')),
            onPressed: () => openPage<void>(context, StockHistory(widget.api)),
          ),
        ],
      ),
    ),
  );
}

class PreparedDetail extends StatelessWidget {
  const PreparedDetail(this.api, this.id, {super.key});
  final HouseholdApi api;
  final String id;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('physical_prepared'))),
    body: Resource<PlanningData>(
      sensitive: true,
      pollInterval: null,
      load: () => PlanningData.load(api),
      builder: (context, data, reload) {
        final row = data.prepared
            .where((r) => object(r['batch'])['id'] == id)
            .firstOrNull;
        if (row == null) {
          return const EmptyContent(
            icon: Icons.inventory_2_outlined,
            title: 'prepared_unavailable',
          );
        }
        final b = object(row['batch']), storage = object(b['storage']);
        Future<void> command(Json value) async {
          await runAction(context, () async {
            await reviewStock(context, api, data, value);
            await reload();
          });
        }

        return ListView(
          padding: const EdgeInsets.all(16),
          children: [
            Text(
              b['title'] as String,
              style: Theme.of(context).textTheme.headlineSmall,
            ),
            Text(
              '${context.t('revision')} ${b['revision']} · ${context.t('produced_on')} ${b['produced_on']}',
            ),
            Note(
              '${context.t('actual_yield')}: ${b['original_portions']}\n${context.t('reserved')}: ${row['held']} · ${context.t('consumed_portions')}: ${row['used']} · ${context.t('available')}: ${row['available']}',
            ),
            Text(
              '${storage['location']} · ${context.t('storage_${storage['condition']}')}\n${context.t('storage_declared_on')}: ${storage['declared_on']}\n${context.t('declared_use_by')}: ${storage['use_by'] ?? context.t('no_declared_date')}',
            ),
            Note(context.t('prepared_declaration_note')),
            for (final a in objects(b['allocations']))
              Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(
                      '${a['date']} · ${context.t(a['meal'] as String)} · ${a['portions']} ${context.t('servings')}',
                    ),
                    subtitle: Text(
                      '${(a['member_ids'] as List).map(data.person).join(', ')} · ${context.t(a['state'] as String)}',
                    ),
                  ),
                  if (api.writable && a['state'] == 'held')
                    Wrap(
                      spacing: 8,
                      children: [
                        OutlinedButton(
                          onPressed: () => command({
                            'operation': 'use_prepared',
                            'batch_id': id,
                            'allocation_id': a['id'],
                          }),
                          child: Text(context.t('record_portion_use')),
                        ),
                        TextButton(
                          onPressed: () => command({
                            'operation': 'release_prepared',
                            'batch_id': id,
                            'allocation_id': a['id'],
                          }),
                          child: Text(context.t('release_portions')),
                        ),
                      ],
                    ),
                  if (api.writable && a['state'] == 'consumed')
                    TextButton(
                      onPressed: () async {
                        final reason = await askName(
                          context,
                          'correction_reason',
                        );
                        if (reason != null) {
                          await command({
                            'operation': 'undo_prepared_use',
                            'batch_id': id,
                            'allocation_id': a['id'],
                            'reason': reason,
                          });
                        }
                      },
                      child: Text(context.t('undo_portion_use')),
                    ),
                ],
              ),
            if (api.writable && b['reversed'] != true) ...[
              FilledButton(
                onPressed: (row['available'] as int) == 0
                    ? null
                    : () async {
                        await openPage<void>(
                          context,
                          PreparedAllocationEditor(api, data, b),
                        );
                        await reload();
                      },
                child: Text(context.t('allocate_portions')),
              ),
              OutlinedButton(
                onPressed: () async {
                  await openPage<void>(
                    context,
                    PreparedEditor(api, data, batch: b),
                  );
                  await reload();
                },
                child: Text(context.t('change_storage')),
              ),
              TextButton(
                onPressed: (row['held'] as int) + (row['used'] as int) > 0
                    ? null
                    : () async {
                        final reason = await askName(
                          context,
                          'correction_reason',
                        );
                        if (reason != null) {
                          await command({
                            'operation': 'reverse_prepared',
                            'batch_id': id,
                            'reason': reason,
                          });
                        }
                      },
                child: Text(context.t('reverse_prepared')),
              ),
            ],
          ],
        );
      },
    ),
  );
}
