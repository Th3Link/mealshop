import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../planning/data.dart';
import '../shopping/models.dart' show Shop;
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'ingredient_picker.dart';
import 'plan_common.dart';
import 'shops.dart';
import 'stock_review.dart';
import 'item_details.dart' show originLabel;

class StockEditor extends StatefulWidget {
  const StockEditor(
    this.api,
    this.data, {
    super.key,
    this.offer = false,
    this.lot,
    this.offerRecord,
  });
  final HouseholdApi api;
  final PlanningData data;
  final bool offer;
  final Json? lot, offerRecord;
  @override
  State<StockEditor> createState() => _StockEditorState();
}

class _StockEditorState extends State<StockEditor> {
  final form = GlobalKey<FormState>();
  late final language = widget.api.controller.language;
  String mode = 'update_lot';
  late String kind = widget.offer ? 'ingredient' : 'unmatched',
      ingredient = '',
      barcode = '',
      label = '',
      quantity = '1',
      unit = 'g',
      reason = '',
      price = '1',
      currency = 'EUR',
      source = '',
      url = '';
  late String? shop =
      widget.offerRecord?['shop_id'] as String? ??
      widget.api.controller.shopping?.shops.firstOrNull?.id;
  late String from = isoDay(DateTime.now()), until = from;
  late Json metadata = widget.lot == null
      ? {'location': '', 'batch': '', 'expiry': null}
      : clone(object(widget.lot!['metadata']));
  bool dirty = false, saved = false, busy = false;
  @override
  void initState() {
    super.initState();
    if (widget.lot != null) {
      quantity = inputDecimal(widget.lot!['remaining'] as String, language);
      unit = widget.lot!['unit'] as String;
    }
    final o = widget.offerRecord;
    if (o != null) {
      final binding = object(o['binding']);
      kind = binding['product'] == null ? 'ingredient' : 'product';
      ingredient = binding['ingredient_id'] as String? ?? '';
      barcode = (binding['product'] as Json?)?['barcode'] as String? ?? '';
      quantity = inputDecimal(
        object(o['quantity'])['amount'] as String,
        language,
      );
      unit = object(o['quantity'])['unit'] as String;
      price = inputDecimal(o['price'] as String, language);
      currency = o['currency'] as String;
      from = o['valid_from'] as String;
      until = o['valid_until'] as String;
      source = o['source'] as String;
      url = o['source_url'] as String? ?? '';
    }
  }

  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => dirty && !saved,
    child: Scaffold(
      appBar: AppBar(
        title: Text(
          context.t(
            widget.offer
                ? 'record_offer'
                : widget.lot == null
                ? 'add_stock'
                : 'edit_stock',
          ),
        ),
      ),
      body: Form(
        key: form,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            if (widget.lot != null)
              Choice('stock_action', mode, {
                'update_lot': context.t('change_storage'),
                'adjust_lot': context.t('correct_stock'),
              }, (v) => setState(() => mode = v)),
            if (widget.lot == null) ...[
              Choice(
                'stock_identity',
                kind,
                {
                  'ingredient': context.t('ingredient_name'),
                  'product': context.t('source_product'),
                  if (!widget.offer) 'unmatched': context.t('unmatched_stock'),
                },
                (v) => setState(() {
                  kind = v;
                  dirty = true;
                }),
              ),
              if (kind == 'ingredient')
                OutlinedButton.icon(
                  icon: const Icon(Icons.search),
                  label: Text(
                    widget.data.catalogue.find(ingredient) == null
                        ? context.t('find_ingredient')
                        : widget.data.catalogue.name(
                            widget.data.catalogue.find(ingredient)!,
                            language,
                          ),
                  ),
                  onPressed: () async {
                    final item = await pickIngredient(context, widget.api);
                    if (item != null && mounted) {
                      setState(() {
                        ingredient = item['id'] as String;
                        dirty = true;
                      });
                    }
                  },
                ),
              if (kind == 'product')
                Choice(
                  'source_product',
                  barcode,
                  {
                    for (final e
                        in (widget.api.controller.shopping?.products ??
                                <String, dynamic>{})
                            .entries)
                      e.key: object(e.value)['name'] as String,
                  },
                  (v) => setState(() {
                    barcode = v;
                    dirty = true;
                  }),
                ),
              if (kind == 'unmatched')
                EditText('name', label, (v) {
                  label = v;
                  dirty = true;
                }, required: true),
            ],
            if (widget.lot == null || mode == 'adjust_lot') ...[
              EditText(
                'quantity',
                quantity,
                (v) {
                  quantity = v;
                  dirty = true;
                },
                required: true,
                numeric: true,
              ),
              Choice(
                'unit',
                unit,
                {
                  for (final u in ['g', 'kg', 'ml', 'l', 'piece'])
                    u: u == 'piece' ? context.t('piece') : u,
                },
                (v) => setState(() {
                  unit = v;
                  dirty = true;
                }),
              ),
            ],
            if (!widget.offer &&
                (widget.lot == null || mode == 'update_lot')) ...[
              EditText(
                'storage_location',
                metadata['location'] as String,
                (v) {
                  metadata['location'] = v;
                  dirty = true;
                },
                required: true,
                max: 100,
              ),
              EditText('stock_batch', metadata['batch'] as String, (v) {
                metadata['batch'] = v;
                dirty = true;
              }, max: 100),
              if (metadata['expiry'] == null)
                TextButton.icon(
                  icon: const Icon(Icons.add),
                  label: Text(context.t('declare_use_by')),
                  onPressed: () => setState(() {
                    metadata['expiry'] = from;
                    dirty = true;
                  }),
                )
              else ...[
                DayButton(
                  'declared_use_by',
                  metadata['expiry'] as String,
                  (v) => setState(() {
                    metadata['expiry'] = v;
                    dirty = true;
                  }),
                ),
                TextButton(
                  onPressed: () => setState(() {
                    metadata['expiry'] = null;
                    dirty = true;
                  }),
                  child: Text(context.t('remove_use_by')),
                ),
              ],
              Note(context.t('prepared_declaration_note')),
            ],
            if (widget.lot != null && mode == 'adjust_lot')
              EditText(
                'correction_reason',
                reason,
                (v) {
                  reason = v;
                  dirty = true;
                },
                required: true,
                max: 500,
              ),
            if (widget.offer) ...[
              Choice(
                'shop',
                shop,
                {
                  for (final s
                      in widget.api.controller.shopping?.shops ??
                          const <Shop>[])
                    s.id: s.name,
                },
                (v) => setState(() {
                  shop = v;
                  dirty = true;
                }),
              ),
              TextButton(
                onPressed: () async {
                  await openPage<void>(
                    context,
                    ShopsScreen(widget.api.controller),
                  );
                  if (mounted) setState(() {});
                },
                child: Text(context.t('shops')),
              ),
              EditText(
                'offer_price',
                price,
                (v) {
                  price = v;
                  dirty = true;
                },
                required: true,
                numeric: true,
              ),
              Choice(
                'currency',
                currency,
                {
                  for (final c in ['EUR', 'GBP', 'USD', 'CHF']) c: c,
                },
                (v) => setState(() {
                  currency = v;
                  dirty = true;
                }),
              ),
              DayButton(
                'valid_from',
                from,
                (v) => setState(() {
                  from = v;
                  dirty = true;
                }),
              ),
              DayButton(
                'valid_until',
                until,
                (v) => setState(() {
                  until = v;
                  dirty = true;
                }),
              ),
              EditText(
                'source_label',
                source,
                (v) {
                  source = v;
                  dirty = true;
                },
                required: true,
                max: 1000,
              ),
              EditText('source_url', url, (v) {
                url = v;
                dirty = true;
              }, max: 2000),
            ],
            FilledButton(
              onPressed: busy || !widget.api.writable
                  ? null
                  : () async {
                      if (!form.currentState!.validate()) return;
                      setState(() => busy = true);
                      try {
                        final target = <String, dynamic>{
                          'kind': kind,
                          if (kind == 'ingredient') 'ingredient_id': ingredient,
                          if (kind == 'product') 'barcode': barcode,
                          if (kind == 'unmatched') 'label': label,
                        };
                        final q = <String, dynamic>{
                          'amount': exactInput(quantity, language),
                          'unit': unit,
                        };
                        final command = widget.offer
                            ? <String, dynamic>{
                                'operation': 'save_offer',
                                'offer': {
                                  'id': widget.offerRecord?['id'],
                                  'shop_id': shop,
                                  'target': target,
                                  'quantity': q,
                                  'price': exactInput(price, language),
                                  'currency': currency,
                                  'valid_from': from,
                                  'valid_until': until,
                                  'source': source,
                                  'source_url': url.trim().isEmpty
                                      ? null
                                      : url.trim(),
                                },
                              }
                            : widget.lot == null
                            ? <String, dynamic>{
                                'operation': 'create_lot',
                                'target': target,
                                'quantity': q,
                                'metadata': metadata,
                              }
                            : mode == 'adjust_lot'
                            ? <String, dynamic>{
                                'operation': mode,
                                'lot_id': widget.lot!['id'],
                                'quantity': q,
                                'reason': reason,
                              }
                            : <String, dynamic>{
                                'operation': 'update_lot',
                                'lot_id': widget.lot!['id'],
                                'metadata': metadata,
                              };
                        if (await reviewStock(
                              context,
                              widget.api,
                              widget.data,
                              command,
                            ) &&
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
        ),
      ),
    ),
  );
}

class StockSourceEditor extends StatefulWidget {
  const StockSourceEditor(
    this.api,
    this.data, {
    super.key,
    required this.receive,
  });
  final HouseholdApi api;
  final PlanningData data;
  final bool receive;
  @override
  State<StockSourceEditor> createState() => _StockSourceEditorState();
}

class _StockSourceEditorState extends State<StockSourceEditor> {
  final form = GlobalKey<FormState>();
  final selected = <String>{};
  final metadata = <String, dynamic>{
    'location': '',
    'batch': '',
    'expiry': null,
  };
  String date = isoDay(DateTime.now());
  bool busy = false, saved = false;
  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => selected.isNotEmpty && !saved,
    child: Scaffold(
      appBar: AppBar(
        title: Text(
          context.t(widget.receive ? 'receive_stock' : 'reserve_stock'),
        ),
      ),
      body: Resource<Json>(
        load: () => widget.api.household('/shopping'),
        builder: (context, shopping, reload) => Form(
          key: form,
          child: ListView(
            padding: const EdgeInsets.all(16),
            children: [
              Note(
                context.t(
                  widget.receive ? 'receive_stock_note' : 'reserve_stock_note',
                ),
              ),
              for (final row
                  in widget.receive
                      ? objects(widget.data.stock['purchases'])
                      : objects(shopping['sources'])
                            .where((s) => s['active'] == true))
                CheckboxListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(
                    widget.receive
                        ? object(row['item'])['name'] as String
                        : originLabel(context, object(row['origin'])),
                  ),
                  subtitle: Text(
                    widget.receive
                        ? '${exactNumber(context, row['amount'] as String)} ${object(row['item'])['unit']}'
                        : '${objects(row['requirements']).length} ${context.t('ingredient_requirements')}',
                  ),
                  value: selected.contains(row['id']),
                  onChanged: busy
                      ? null
                      : (v) => setState(
                          () => v == true
                              ? selected.add(row['id'] as String)
                              : selected.remove(row['id']),
                        ),
                ),
              if (widget.receive) ...[
                EditText(
                  'storage_location',
                  metadata['location'] as String,
                  (v) => metadata['location'] = v,
                  required: true,
                  max: 100,
                ),
                EditText(
                  'stock_batch',
                  metadata['batch'] as String,
                  (v) => metadata['batch'] = v,
                  max: 100,
                ),
                if (metadata['expiry'] == null)
                  TextButton(
                    onPressed: () => setState(() => metadata['expiry'] = date),
                    child: Text(context.t('declare_use_by')),
                  )
                else
                  DayButton(
                    'declared_use_by',
                    metadata['expiry'] as String,
                    (v) => setState(() => metadata['expiry'] = v),
                  ),
              ] else
                DayButton('use_on', date, (v) => setState(() => date = v)),
              FilledButton(
                onPressed: busy || selected.isEmpty || selected.length > 100
                    ? null
                    : () async {
                        if (!form.currentState!.validate()) return;
                        setState(() => busy = true);
                        try {
                          if (await reviewStock(
                                context,
                                widget.api,
                                widget.data,
                                widget.receive
                                    ? {
                                        'operation': 'receive',
                                        'purchase_ids': selected.toList(),
                                        'metadata': metadata,
                                      }
                                    : {
                                        'operation': 'reserve',
                                        'source_ids': selected.toList(),
                                        'as_of': date,
                                      },
                              ) &&
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
          ),
        ),
      ),
    ),
  );
}
