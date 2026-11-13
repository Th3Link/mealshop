import 'package:material_ui/material_ui.dart';

import '../shopping/controller.dart';
import '../shopping/models.dart';
import '../shopping/quantity.dart';
import 'add_item.dart';
import 'common.dart';
import 'item_details.dart';
import 'messages.dart';
import 'pending.dart';

class ShoppingListScreen extends StatefulWidget {
  const ShoppingListScreen(this.c, {super.key});
  final ShoppingController c;
  @override
  State<ShoppingListScreen> createState() => _ShoppingListScreenState();
}

class _ShoppingListScreenState extends State<ShoppingListScreen> {
  String search = '';
  String? shop;
  String? group;
  Future<void> change(ShoppingItem item, bool checked, int revision) async {
    try {
      await widget.c.enqueue(
        {'operation': 'check', 'key': item.key, 'checked': checked},
        offline: true,
        expectedRevision: revision,
      );
    } catch (e) {
      if (mounted) await showFailure(context, e);
    }
  }

  @override
  Widget build(BuildContext context) {
    final c = widget.c;
    final list = c.shopping;
    if (list == null) {
      return Center(
        child: c.busy
            ? const CircularProgressIndicator()
            : FilledButton(
                onPressed: c.refresh,
                child: Text(context.t('retry')),
              ),
      );
    }
    final ingredientGroups = c.ingredientGroups;
    final groups =
        {for (final value in ingredientGroups.values) value.id: value}.values
            .toList()
          ..sort((a, b) => a.label(c.language).compareTo(b.label(c.language)));
    final selectedGroup = groups.any((g) => g.id == group) ? group : null;
    final matching = list.items
        .where(
          (i) =>
              (shop == null || i.shopId == shop) &&
              (selectedGroup == null ||
                  ingredientGroups[i.ingredientId]?.id == selectedGroup) &&
              i.label(c.language).toLowerCase().contains(search.toLowerCase()),
        )
        .toList();
    final needed = matching
        .where((i) => Quantity.positive(i.remaining))
        .toList();
    final bought = matching
        .where((i) => Quantity.positive(i.purchased))
        .toList();
    return Scaffold(
      backgroundColor: Colors.transparent,
      floatingActionButton: c.writable
          ? FloatingActionButton.extended(
              onPressed: () => Navigator.push<void>(
                context,
                MaterialPageRoute(builder: (_) => AddItemScreen(c)),
              ),
              icon: const Icon(Icons.add),
              label: Text(context.t('add')),
            )
          : null,
      body: RefreshIndicator(
        onRefresh: c.refresh,
        child: CustomScrollView(
          slivers: [
            SliverToBoxAdapter(
              child: Padding(
                padding: const EdgeInsets.fromLTRB(20, 8, 20, 0),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Row(
                      children: [
                        Icon(
                          c.online
                              ? Icons.cloud_done_outlined
                              : Icons.cloud_off_outlined,
                          size: 18,
                        ),
                        const SizedBox(width: 8),
                        Expanded(
                          child: Text(
                            context.t(
                              c.busy
                                  ? 'syncing'
                                  : c.online
                                  ? 'synced'
                                  : 'offline',
                            ),
                            style: Theme.of(context).textTheme.bodySmall,
                          ),
                        ),
                      ],
                    ),
                    if (c.home?.writable == false) Text(context.t('reader')),
                    if (c.syncedAt != null)
                      Text(
                        '${context.t('last_sync')}: ${MaterialLocalizations.of(context).formatTimeOfDay(TimeOfDay.fromDateTime(c.syncedAt!.toLocal()))}',
                        style: Theme.of(context).textTheme.bodySmall,
                      ),
                    const SizedBox(height: 16),
                    TextField(
                      onChanged: (v) => setState(() => search = v),
                      decoration: InputDecoration(
                        labelText: context.t('search'),
                        prefixIcon: const Icon(Icons.search),
                      ),
                    ),
                    if (groups.isNotEmpty)
                      Padding(
                        padding: const EdgeInsets.only(top: 12),
                        child: DropdownButtonFormField<String>(
                          key: ValueKey(selectedGroup),
                          initialValue: selectedGroup ?? '',
                          isExpanded: true,
                          decoration: InputDecoration(
                            labelText: context.t('ingredient_group'),
                          ),
                          items: [
                            DropdownMenuItem(
                              value: '',
                              child: Text(context.t('all_groups')),
                            ),
                            for (final g in groups)
                              DropdownMenuItem(
                                value: g.id,
                                child: Text(
                                  g.label(c.language),
                                  overflow: TextOverflow.ellipsis,
                                ),
                              ),
                          ],
                          onChanged: (value) => setState(
                            () => group = value == '' ? null : value,
                          ),
                        ),
                      ),
                    if (list.shops.isNotEmpty)
                      Padding(
                        padding: const EdgeInsets.only(top: 12),
                        child: Wrap(
                          spacing: 8,
                          runSpacing: 4,
                          children: [
                            ChoiceChip(
                              label: Text(context.t('all_shops')),
                              selected: shop == null,
                              onSelected: (_) => setState(() => shop = null),
                            ),
                            for (final s in list.shops)
                              ChoiceChip(
                                label: Text(s.name),
                                selected: shop == s.id,
                                onSelected: (_) => setState(() => shop = s.id),
                              ),
                          ],
                        ),
                      ),
                  ],
                ),
              ),
            ),
            for (final entry in c.pending.where(
              (p) =>
                  p.command['operation'] == 'add_manual' &&
                  p.command['source_id'] == null,
            ))
              SliverToBoxAdapter(
                child: ListTile(
                  leading: Checkbox(
                    semanticLabel:
                        '${context.t(entry.command['purchased'] == true ? 'undo' : 'check')}: ${entry.command['label']}',
                    value: entry.command['purchased'] == true,
                    onChanged: !c.writable || entry.attempted
                        ? null
                        : (value) async {
                            try {
                              await c.markPending(entry.id, value ?? false);
                            } catch (e) {
                              if (context.mounted) {
                                await showFailure(context, e);
                              }
                            }
                          },
                  ),
                  title: Text(entry.command['label'] as String),
                  subtitle: Text(
                    '${Quantity.display((entry.command['quantity'] as Map<String, dynamic>)['amount'] as String, c.language)} ${(entry.command['quantity'] as Map<String, dynamic>)['unit']} · ${context.t('pending')}',
                  ),
                  trailing: const Icon(Icons.schedule),
                  onTap: () => Navigator.push<void>(
                    context,
                    MaterialPageRoute(builder: (_) => PendingScreen(c)),
                  ),
                ),
              ),
            if (c.pending.isNotEmpty)
              SliverToBoxAdapter(
                child: ListTile(
                  leading: const Icon(Icons.sync),
                  title: Text('${context.t('pending')} (${c.pending.length})'),
                  subtitle: Text(context.t('pending_body')),
                  trailing: const Icon(Icons.chevron_right),
                  onTap: () => Navigator.push<void>(
                    context,
                    MaterialPageRoute(builder: (_) => PendingScreen(c)),
                  ),
                ),
              ),
            if (list.items.isEmpty && c.pending.isEmpty)
              SliverFillRemaining(
                hasScrollBody: false,
                child: Padding(
                  padding: const EdgeInsets.all(32),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Icon(
                        Icons.shopping_basket_outlined,
                        size: 56,
                        color: Theme.of(context).colorScheme.primary,
                      ),
                      const SizedBox(height: 20),
                      Text(
                        context.t('empty'),
                        style: Theme.of(context).textTheme.titleLarge,
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 12),
                      Text(
                        context.t('empty_body'),
                        textAlign: TextAlign.center,
                      ),
                      const SizedBox(height: 96),
                    ],
                  ),
                ),
              ),
            if (needed.isNotEmpty) ...[
              SliverToBoxAdapter(
                child: SectionTitle(
                  '${context.t('needed')} · ${needed.length}',
                ),
              ),
              SliverPadding(
                padding: const EdgeInsets.symmetric(horizontal: 16),
                sliver: SliverLayoutBuilder(
                  builder: (context, constraints) {
                    final scale = MediaQuery.textScalerOf(context).scale(1);
                    final count =
                        constraints.crossAxisExtent < 350 || scale > 1.3
                        ? 1
                        : constraints.crossAxisExtent > 700
                        ? 3
                        : 2;
                    return SliverGrid.builder(
                      gridDelegate: SliverGridDelegateWithFixedCrossAxisCount(
                        crossAxisCount: count,
                        mainAxisExtent: 148 * scale,
                        crossAxisSpacing: 8,
                        mainAxisSpacing: 8,
                      ),
                      itemCount: needed.length,
                      itemBuilder: (context, index) {
                        final item = needed[index];
                        final pending = c.pending.any(
                          (p) => p.command['key'] == item.key,
                        );
                        return Card.filled(
                          margin: EdgeInsets.zero,
                          child: InkWell(
                            borderRadius: BorderRadius.circular(12),
                            onTap: () => showItem(context, c, item),
                            child: Padding(
                              padding: const EdgeInsets.fromLTRB(16, 8, 8, 8),
                              child: Column(
                                crossAxisAlignment: CrossAxisAlignment.start,
                                children: [
                                  Row(
                                    children: [
                                      Expanded(
                                        child: Text(
                                          '${Quantity.display(item.remaining, c.language)} ${item.unit == 'piece' ? context.t('piece') : item.unit}',
                                          style: Theme.of(context)
                                              .textTheme
                                              .titleMedium,
                                        ),
                                      ),
                                      IconButton(
                                        tooltip: context.t('check'),
                                        onPressed: c.writable && !pending
                                            ? () => change(
                                                item,
                                                true,
                                                list.revision,
                                              )
                                            : null,
                                        icon: Icon(
                                          pending
                                              ? Icons.schedule
                                              : Icons
                                                    .add_shopping_cart_outlined,
                                        ),
                                      ),
                                    ],
                                  ),
                                  Expanded(
                                    child: Align(
                                      alignment: Alignment.centerLeft,
                                      child: Text(
                                        item.label(c.language),
                                        style: Theme.of(context)
                                            .textTheme
                                            .titleMedium,
                                        maxLines: 3,
                                        overflow: TextOverflow.ellipsis,
                                      ),
                                    ),
                                  ),
                                  if (ingredientGroups[item.ingredientId] !=
                                      null)
                                    Text(
                                      ingredientGroups[item.ingredientId]!
                                          .label(c.language),
                                      maxLines: 1,
                                      overflow: TextOverflow.ellipsis,
                                      style: Theme.of(context)
                                          .textTheme
                                          .bodySmall,
                                    ),
                                  if (item.shopId != null)
                                    Text(
                                      list.shops
                                              .where((s) => s.id == item.shopId)
                                              .firstOrNull
                                              ?.name ??
                                          '',
                                      style: Theme.of(context)
                                          .textTheme
                                          .bodySmall,
                                    ),
                                ],
                              ),
                            ),
                          ),
                        );
                      },
                    );
                  },
                ),
              ),
            ],
            if (bought.isNotEmpty) ...[
              SliverToBoxAdapter(
                child: SectionTitle(
                  '${context.t('purchased')} · ${bought.length}',
                ),
              ),
              SliverList.builder(
                itemCount: bought.length,
                itemBuilder: (context, index) {
                  final item = bought[index];
                  return ListTile(
                    leading: const Icon(Icons.check_circle_outline),
                    title: Text(item.label(c.language)),
                    subtitle: Text(
                      '${Quantity.display(item.purchased, c.language)} ${item.unit == 'piece' ? context.t('piece') : item.unit}',
                    ),
                    onTap: () => showItem(context, c, item),
                    trailing: c.writable
                        ? IconButton(
                            tooltip: context.t('undo'),
                            onPressed: c.busy
                                ? null
                                : () => change(item, false, list.revision),
                            icon: const Icon(Icons.undo),
                          )
                        : null,
                  );
                },
              ),
            ],
            if (matching.isEmpty && list.items.isNotEmpty)
              SliverToBoxAdapter(
                child: Padding(
                  padding: const EdgeInsets.all(32),
                  child: Text(context.t('no_results')),
                ),
              ),
            if (matching.any(
              (i) =>
                  !Quantity.positive(i.remaining) &&
                  !Quantity.positive(i.purchased) &&
                  (Quantity.positive(i.reserved) || Quantity.positive(i.used)),
            )) ...[
              SliverToBoxAdapter(child: SectionTitle(context.t('covered'))),
              for (final item in matching.where(
                (i) =>
                    !Quantity.positive(i.remaining) &&
                    !Quantity.positive(i.purchased) &&
                    (Quantity.positive(i.reserved) ||
                        Quantity.positive(i.used)),
              ))
                SliverToBoxAdapter(
                  child: ListTile(
                    leading: const Icon(Icons.inventory_2_outlined),
                    title: Text(item.label(c.language)),
                    subtitle: Text(
                      [
                        if (Quantity.positive(item.reserved))
                          '${context.t('reserved')}: ${Quantity.display(item.reserved, c.language)} ${item.unit}',
                        if (Quantity.positive(item.used))
                          '${context.t('used')}: ${Quantity.display(item.used, c.language)} ${item.unit}',
                      ].join(' · '),
                    ),
                    onTap: () => showItem(context, c, item),
                  ),
                ),
            ],
            const SliverToBoxAdapter(child: SizedBox(height: 112)),
          ],
        ),
      ),
    );
  }
}
