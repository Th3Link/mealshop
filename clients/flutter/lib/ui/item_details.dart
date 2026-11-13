import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../shopping/controller.dart';
import '../shopping/models.dart';
import '../shopping/quantity.dart';
import 'add_item.dart';
import 'common.dart';
import 'messages.dart';

String originLabel(BuildContext context, Json origin) {
  switch (origin['kind']) {
    case 'manual':
      return origin['label'] as String;
    case 'recipe':
      return '${context.t('source_recipe')}: ${origin['title']}';
    case 'meal':
      return '${origin['date']} · ${origin['meal']} · ${origin['title']}';
    case 'product':
      return object(origin['product'])['name'] as String;
    case 'offer':
      return '${context.t('source_offer')}: ${object(origin['offer'])['shop_name']}';
    default:
      return context.t('source');
  }
}

Future<void> showItem(
  BuildContext context,
  ShoppingController c,
  ShoppingItem item,
) async {
  await Navigator.push<void>(
    context,
    MaterialPageRoute(builder: (_) => ItemDetails(c, item.key)),
  );
}

class ItemDetails extends StatelessWidget {
  const ItemDetails(this.c, this.itemKey, {super.key});
  final ShoppingController c;
  final String itemKey;
  Future<void> command(
    BuildContext context,
    Json command, {
    int? expectedRevision,
  }) async {
    try {
      await c.enqueue(command, expectedRevision: expectedRevision);
    } catch (e) {
      if (context.mounted) await showFailure(context, e);
    }
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: c,
    builder: (context, _) {
      final observedRevision = c.shopping?.revision;
      Future<void> apply(Json value) =>
          command(context, value, expectedRevision: observedRevision);
      final item = c.shopping?.items.where((i) => i.key == itemKey).firstOrNull;
      return Scaffold(
        appBar: AppBar(
          title: Text(item?.label(c.language) ?? context.t('shopping')),
        ),
        body: item == null
            ? Center(child: Text(context.t('no_results')))
            : ListView(
                padding: const EdgeInsets.all(20),
                children: [
                  Text(
                    '${Quantity.display(item.remaining, c.language)} ${item.unit == 'piece' ? context.t('piece') : item.unit}',
                    style: Theme.of(context).textTheme.headlineMedium,
                  ),
                  Text(context.t('needed')),
                  if (Quantity.positive(item.purchased))
                    Padding(
                      padding: const EdgeInsets.only(top: 8),
                      child: Text(
                        '${context.t('purchased')}: ${Quantity.display(item.purchased, c.language)} ${item.unit}',
                      ),
                    ),
                  if (c.writable && c.online)
                    Padding(
                      padding: const EdgeInsets.only(top: 24),
                      child: DropdownButtonFormField<String>(
                        key: ValueKey(item.shopId),
                        initialValue: item.shopId ?? '',
                        isExpanded: true,
                        decoration: InputDecoration(
                          labelText: context.t('assign_shop'),
                        ),
                        items: [
                          DropdownMenuItem(
                            value: '',
                            child: Text(context.t('unassigned')),
                          ),
                          for (final s in c.shopping!.shops)
                            DropdownMenuItem(value: s.id, child: Text(s.name)),
                        ],
                        onChanged: c.busy
                            ? null
                            : (v) => apply({
                                'operation': 'assign_shop',
                                'key': item.key,
                                'shop_id': v == '' ? null : v,
                              }),
                      ),
                    ),
                  if (c.writable && c.online && item.ingredientId != null)
                    ShopPreference(c, item.ingredientId!),
                  SectionTitle(context.t('sources')),
                  for (final contribution in item.contributions)
                    Card.outlined(
                      child: Padding(
                        padding: const EdgeInsets.all(16),
                        child: Column(
                          crossAxisAlignment: CrossAxisAlignment.start,
                          children: [
                            Text(
                              originLabel(
                                context,
                                object(contribution['origin']),
                              ),
                              style: Theme.of(context).textTheme.titleMedium,
                            ),
                            const SizedBox(height: 8),
                            Text(
                              '${context.t('required')}: ${Quantity.display(contribution['required'] as String, c.language)} ${item.unit}',
                            ),
                            Text(
                              '${context.t('purchased')}: ${Quantity.display(contribution['purchased'] as String, c.language)} ${item.unit}',
                            ),
                            if (c.writable &&
                                c.online &&
                                c.shopping!.sources.any(
                                  (s) =>
                                      s['id'] == contribution['source_id'] &&
                                      s['active'] == true,
                                ))
                              Wrap(
                                spacing: 8,
                                children: [
                                  if (object(contribution['origin'])['kind'] ==
                                      'manual')
                                    TextButton.icon(
                                      onPressed: c.busy
                                          ? null
                                          : () {
                                              final source = c.shopping!.sources
                                                  .where(
                                                    (s) =>
                                                        s['id'] ==
                                                            contribution['source_id'] &&
                                                        s['active'] == true,
                                                  )
                                                  .firstOrNull;
                                              if (source != null) {
                                                Navigator.push<void>(
                                                  context,
                                                  MaterialPageRoute(
                                                    builder: (_) =>
                                                        AddItemScreen(
                                                          c,
                                                          source: source,
                                                        ),
                                                  ),
                                                );
                                              }
                                            },
                                      icon: const Icon(Icons.edit_outlined),
                                      label: Text(context.t('edit')),
                                    ),
                                  TextButton.icon(
                                    onPressed: c.busy
                                        ? null
                                        : () async {
                                            if (await confirm(
                                                  context,
                                                  'confirm_delete',
                                                ) &&
                                                context.mounted) {
                                              await apply({
                                                'operation': 'remove_source',
                                                'id': contribution['source_id'],
                                              });
                                            }
                                          },
                                    icon: const Icon(Icons.delete_outline),
                                    label: Text(context.t('delete')),
                                  ),
                                ],
                              ),
                          ],
                        ),
                      ),
                    ),
                ],
              ),
      );
    },
  );
}

class ShopPreference extends StatelessWidget {
  const ShopPreference(this.c, this.ingredient, {super.key});
  final ShoppingController c;
  final String ingredient;
  @override
  Widget build(BuildContext context) {
    final shops = c.shopping!.shops;
    final observedRevision = c.shopping!.revision;
    final preferences = object(
      object(c.shopping!.json['shops'])['preferences'],
    );
    final entry =
        preferences[ingredient] as Json? ??
        <String, dynamic>{'first': null, 'second': null};
    return Column(
      children: [
        for (final field in ['first', 'second'])
          Padding(
            padding: const EdgeInsets.only(top: 16),
            child: DropdownButtonFormField<String>(
              key: ValueKey('$field:${entry[field]}'),
              initialValue: entry[field] as String? ?? '',
              isExpanded: true,
              decoration: InputDecoration(
                labelText: context.t(
                  field == 'first' ? 'preferred_shop' : 'fallback_shop',
                ),
              ),
              items: [
                DropdownMenuItem(
                  value: '',
                  child: Text(context.t('unassigned')),
                ),
                for (final shop in shops.where(
                  (s) => s.id != entry[field == 'first' ? 'second' : 'first'],
                ))
                  DropdownMenuItem(value: shop.id, child: Text(shop.name)),
              ],
              onChanged: c.busy || c.pending.isNotEmpty
                  ? null
                  : (value) async {
                      try {
                        await c.enqueue({
                          'operation': 'preference',
                          'ingredient_id': ingredient,
                          'first': field == 'first'
                              ? (value == '' ? null : value)
                              : entry['first'],
                          'second': field == 'second'
                              ? (value == '' ? null : value)
                              : entry['second'],
                        }, expectedRevision: observedRevision);
                      } catch (e) {
                        if (context.mounted) await showFailure(context, e);
                      }
                    },
            ),
          ),
      ],
    );
  }
}
