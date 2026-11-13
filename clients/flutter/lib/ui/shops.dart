import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../shopping/controller.dart';
import 'common.dart';
import 'messages.dart';

class ShopsScreen extends StatelessWidget {
  const ShopsScreen(this.c, {super.key});
  final ShoppingController c;
  Future<void> command(
    BuildContext context,
    Json value, {
    int? expectedRevision,
  }) async {
    try {
      await c.enqueue(value, expectedRevision: expectedRevision);
    } catch (e) {
      if (context.mounted) await showFailure(context, e);
    }
  }

  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: c,
    builder: (context, _) {
      final data = c.shopping;
      Future<void> apply(Json value) =>
          command(context, value, expectedRevision: data?.revision);
      final shops = data?.shops ?? [];
      final sessions =
          (data?.json['shops'] as Json?)?['sessions'] as List? ?? [];
      final learning = (data?.json['shops'] as Json?)?['learning'] == true;
      final activeRound = sessions.any((s) => object(s)['completed'] == false);
      final can = c.writable && c.online && !c.busy && c.pending.isEmpty;
      return Scaffold(
        appBar: AppBar(title: Text(context.t('shops'))),
        body: ListView(
          children: [
            if (c.writable)
              Padding(
                padding: const EdgeInsets.all(20),
                child: FilledButton.icon(
                  onPressed: !can
                      ? null
                      : () async {
                          final name = await askName(context, 'new_shop');
                          if (name != null && context.mounted) {
                            await apply({
                              'operation': 'save_shop',
                              'name': name,
                            });
                          }
                        },
                  icon: const Icon(Icons.add),
                  label: Text(context.t('new_shop')),
                ),
              ),
            if (shops.isEmpty)
              Padding(
                padding: const EdgeInsets.all(24),
                child: Text(context.t('no_results')),
              ),
            for (final shop in shops) ...[
              ListTile(
                leading: const Icon(Icons.storefront_outlined),
                title: Text(shop.name),
                trailing: can
                    ? PopupMenuButton<String>(
                        onSelected: (value) async {
                          if (value == 'edit') {
                            final name = await askName(
                              context,
                              'edit',
                              initial: shop.name,
                            );
                            if (name != null && context.mounted) {
                              await apply({
                                'operation': 'save_shop',
                                'id': shop.id,
                                'name': name,
                              });
                            }
                          }
                          if (value == 'delete' &&
                              context.mounted &&
                              await confirm(context, 'confirm_delete') &&
                              context.mounted) {
                            await apply({
                              'operation': 'remove_shop',
                              'id': shop.id,
                            });
                          }
                          if (value == 'round' && context.mounted) {
                            await apply({
                              'operation': 'start_session',
                              'shop_id': shop.id,
                            });
                          }
                        },
                        itemBuilder: (_) => [
                          PopupMenuItem(
                            value: 'edit',
                            child: Text(context.t('edit')),
                          ),
                          PopupMenuItem(
                            value: 'round',
                            enabled: learning && !activeRound,
                            child: Text(context.t('start_round')),
                          ),
                          PopupMenuItem(
                            value: 'delete',
                            enabled: !sessions.any(
                              (s) =>
                                  object(s)['completed'] == false &&
                                  object(s)['shop_id'] == shop.id,
                            ),
                            child: Text(context.t('delete')),
                          ),
                        ],
                      )
                    : null,
              ),
              for (final item in data!.items.where((i) => i.shopId == shop.id))
                ListTile(
                  contentPadding: const EdgeInsets.only(left: 32, right: 12),
                  title: Text(item.label(c.language)),
                  trailing: can
                      ? Wrap(
                          children: [
                            IconButton(
                              tooltip: context.t('move_up'),
                              onPressed: () async {
                                final keys = data.items
                                    .where((i) => i.shopId == shop.id)
                                    .map((i) => i.key)
                                    .toList();
                                final index = keys.indexOf(item.key);
                                if (index > 0) {
                                  keys.removeAt(index);
                                  keys.insert(index - 1, item.key);
                                  await apply({
                                    'operation': 'order',
                                    'shop_id': shop.id,
                                    'keys': keys,
                                  });
                                }
                              },
                              icon: const Icon(Icons.arrow_upward),
                            ),
                            IconButton(
                              tooltip: context.t('move_down'),
                              onPressed: () async {
                                final keys = data.items
                                    .where((i) => i.shopId == shop.id)
                                    .map((i) => i.key)
                                    .toList();
                                final index = keys.indexOf(item.key);
                                if (index < keys.length - 1) {
                                  keys.removeAt(index);
                                  keys.insert(index + 1, item.key);
                                  await apply({
                                    'operation': 'order',
                                    'shop_id': shop.id,
                                    'keys': keys,
                                  });
                                }
                              },
                              icon: const Icon(Icons.arrow_downward),
                            ),
                          ],
                        )
                      : null,
                ),
            ],
            SectionTitle(context.t('learning')),
            SwitchListTile(
              title: Text(context.t('learning')),
              subtitle: Text(context.t('learning_body')),
              value: learning,
              onChanged: can
                  ? (v) => apply({'operation': 'learning', 'enabled': v})
                  : null,
            ),
            if (can)
              TextButton(
                onPressed: () async {
                  if (await confirm(context, 'reset_history') &&
                      context.mounted) {
                    await apply({'operation': 'reset_history'});
                  }
                },
                child: Text(context.t('reset_history')),
              ),
            for (final raw in sessions.where(
              (s) => object(s)['completed'] == false,
            ))
              Builder(
                builder: (context) {
                  final session = object(raw);
                  return ListTile(
                    title: Text(
                      '${context.t('round')} · ${shops.where((s) => s.id == session['shop_id']).firstOrNull?.name ?? ''}',
                    ),
                    subtitle: can
                        ? Wrap(
                            children: [
                              TextButton(
                                onPressed: () => apply({
                                  'operation': 'finish_session',
                                  'id': session['id'],
                                  'discard': false,
                                }),
                                child: Text(context.t('finish_round')),
                              ),
                              TextButton(
                                onPressed: () => apply({
                                  'operation': 'finish_session',
                                  'id': session['id'],
                                  'discard': true,
                                }),
                                child: Text(context.t('discard_round')),
                              ),
                            ],
                          )
                        : null,
                  );
                },
              ),
            const SizedBox(height: 24),
          ],
        ),
      );
    },
  );
}
