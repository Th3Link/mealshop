import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/controller.dart';
import 'content_common.dart';
import 'cooking_scope.dart';
import 'cooking.dart' show CookingCacheScreen;
import 'brewing_common.dart';
import 'brewing_recipe.dart';
import 'brewing_batch.dart';
import 'brewing_detail.dart';
import 'brewing_review.dart';
import 'messages.dart';

class BrewingScreen extends StatefulWidget {
  const BrewingScreen(this.api, {super.key, this.controller});
  final HouseholdApi api;
  final CookingController? controller;
  @override
  State<BrewingScreen> createState() => _BrewingScreenState();
}

class _BrewingScreenState extends State<BrewingScreen>
    with SingleTickerProviderStateMixin {
  late final tabs = TabController(length: 3, vsync: this);
  CookingController? c;
  bool owned = false, archived = false;
  int version = 0;
  String query = '';
  bool matches(String value) => value.toLowerCase().contains(query);
  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    if (c == null) {
      c = widget.controller ?? CookingScope.maybeOf(context);
      owned = c == null;
      c ??= CookingController(widget.api);
    }
  }

  Future<Json> load() async {
    await c!.initialize();
    if (c!.list == null) await c!.refresh();
    return widget.api.household('/brewing');
  }

  @override
  void dispose() {
    tabs.dispose();
    if (owned) c?.dispose();
    super.dispose();
  }

  Future<void> page(Widget view) async {
    await openPage<Object?>(context, view);
    if (mounted) setState(() => version++);
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: Text(context.t('brewing')),
      bottom: TabBar(
        controller: tabs,
        isScrollable: true,
        tabAlignment: TabAlignment.start,
        tabs: [
          Tab(text: context.t('brewBatches')),
          Tab(text: context.t('brewRecipes')),
          Tab(text: context.t('brewEquipment')),
        ],
      ),
    ),
    body: Column(
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 16, 16, 0),
          child: TextField(
            decoration: InputDecoration(
              labelText: context.t('browse'),
              prefixIcon: const Icon(Icons.search),
            ),
            onChanged: (v) => setState(() => query = v.trim().toLowerCase()),
          ),
        ),
        Expanded(
          child: Resource<Json>(
            key: ValueKey(version),
            sensitive: true,
            pollInterval: null,
            load: load,
            builder: (context, data, reload) => Column(
              children: [
                Padding(
                  padding: const EdgeInsets.symmetric(horizontal: 16),
                  child: Row(
                    children: [
                      Expanded(child: Text(context.t('brewIntro'))),
                      IconButton(
                        tooltip: context.t('refresh'),
                        icon: const Icon(Icons.refresh),
                        onPressed: reload,
                      ),
                    ],
                  ),
                ),
                CheckboxListTile(
                  title: Text(context.t('brew_show_archived')),
                  value: archived,
                  onChanged: (v) => setState(() => archived = v == true),
                ),
                Expanded(
                  child: TabBarView(
                    controller: tabs,
                    children: [
                      _batches(context, data),
                      _recipes(context, data),
                      _equipment(context, data),
                    ],
                  ),
                ),
              ],
            ),
          ),
        ),
      ],
    ),
  );
  List<Widget> common(BuildContext context) => [
    if (!widget.api.writable) Note(context.t('household_read_only')),
    ListenableBuilder(
      listenable: c!,
      builder: (context, _) => BrewingPending(c!),
    ),
    TextButton.icon(
      icon: const Icon(Icons.offline_pin_outlined),
      label: Text(context.t('cook_offline_copies')),
      onPressed: () => page(CookingCacheScreen(c!)),
    ),
  ];
  Widget _batches(BuildContext context, Json data) {
    final batches = objects(data['batches'])
        .where((b) => archived || object(b['batch'])['archived'] != true)
        .where((b) => matches(object(b['run'])['name'] as String))
        .toList();
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        if (c!.writable)
          FilledButton.icon(
            icon: const Icon(Icons.add),
            label: Text(context.t('brewNewBatch')),
            onPressed:
                objects(data['recipes']).any((r) => r['archived'] != true)
                ? () => page(BrewingBatchEditor(c!, data))
                : null,
          ),
        if (batches.isEmpty)
          Note(
            context.t(
              objects(data['recipes']).any((r) => r['archived'] != true)
                  ? 'brewEmptyBatches'
                  : 'brewFirstRecipe',
            ),
          ),
        for (final row in batches)
          ListTile(
            contentPadding: EdgeInsets.zero,
            leading: const Icon(Icons.science_outlined),
            title: Text(object(row['run'])['name'] as String),
            subtitle: Text(
              '${object(object(row['run'])['brew_recipe'])['title']} · v${object(object(row['run'])['brew_recipe'])['revision']}\n${context.t(object(row['run'])['status'] == 'active'
                  ? 'cookRunActive'
                  : object(row['run'])['status'] == 'completed'
                  ? 'cookCompleted'
                  : 'cookCancelled')} · ${row['reading_count']} ${context.t('brewMeasurements')}',
            ),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => page(
              BrewingDetailScreen(c!, object(row['run'])['id'] as String),
            ),
          ),
        ...common(context),
      ],
    );
  }

  Widget _recipes(BuildContext context, Json data) {
    final records = objects(data['recipes'])
        .where((r) => archived || r['archived'] != true)
        .where(
          (r) =>
              matches(object(object(r['record'])['recipe'])['title'] as String),
        )
        .toList();
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        if (c!.writable)
          FilledButton.icon(
            icon: const Icon(Icons.add),
            label: Text(context.t('brewNewRecipe')),
            onPressed: () => page(BrewingRecipeEditor(c!)),
          ),
        if (records.isEmpty) Note(context.t('brewEmptyRecipes')),
        for (final row in records)
          ListTile(
            contentPadding: EdgeInsets.zero,
            leading: const Icon(Icons.menu_book_outlined),
            title: Text(
              object(object(row['record'])['recipe'])['title'] as String,
            ),
            subtitle: Text(
              'v${object(row['record'])['revision']} · ${brewQuantityText(context, object(object(row['record'])['recipe'])['volume'])}',
            ),
            onTap: () => page(
              _BrewRecipeDetail(
                c!,
                object(row['record']),
                row['archived'] == true,
                data,
              ),
            ),
          ),
        ...common(context),
      ],
    );
  }

  Widget _equipment(BuildContext context, Json data) {
    final records = objects(data['equipment'])
        .where((e) => archived || e['archived'] != true)
        .where((e) => matches(object(e['details'])['name'] as String))
        .toList();
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        if (c!.writable)
          FilledButton.icon(
            icon: const Icon(Icons.add),
            label: Text(context.t('brewNewEquipment')),
            onPressed: () => page(BrewingEquipmentEditor(c!)),
          ),
        if (records.isEmpty) Note(context.t('brewEmptyEquipment')),
        for (final equipment in records)
          ListTile(
            contentPadding: EdgeInsets.zero,
            leading: const Icon(Icons.kitchen_outlined),
            title: Text(object(equipment['details'])['name'] as String),
            subtitle: Text(
              '${object(equipment['details'])['model']} · ${brewQuantityText(context, object(equipment['details'])['capacity'])}',
            ),
            trailing: PopupMenuButton<String>(
              tooltip: context.t('details'),
              enabled: c!.writable,
              itemBuilder: (context) => [
                PopupMenuItem(value: 'edit', child: Text(context.t('edit'))),
                PopupMenuItem(
                  value: 'archive',
                  child: Text(
                    context.t(
                      equipment['archived'] == true
                          ? 'brew_restore'
                          : 'brew_archive',
                    ),
                  ),
                ),
              ],
              onSelected: (action) async {
                if (action == 'edit') {
                  await page(BrewingEquipmentEditor(c!, record: equipment));
                } else {
                  await runAction(context, () async {
                    await reviewBrewing(context, c!, {
                      'operation': 'archive_equipment',
                      'id': equipment['id'],
                      'archived': equipment['archived'] != true,
                    });
                    if (mounted) setState(() => version++);
                  });
                }
              },
            ),
          ),
        ...common(context),
      ],
    );
  }
}

class _BrewRecipeDetail extends StatelessWidget {
  const _BrewRecipeDetail(this.c, this.record, this.archived, this.state);
  final CookingController c;
  final Json record, state;
  final bool archived;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(object(record['recipe'])['title'] as String)),
    body: ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Text(
          'v${record['revision']} · ${context.t('brew_original_language')}: ${object(record['recipe'])['language']}',
        ),
        BrewRecipeSummary(object(record['recipe'])),
        if (c.writable) ...[
          if (!archived)
            FilledButton.icon(
              icon: const Icon(Icons.add),
              label: Text(context.t('brewNewBatch')),
              onPressed: () => openPage<String>(
                context,
                BrewingBatchEditor(c, state, recipeId: record['id'] as String),
              ),
            ),
          OutlinedButton.icon(
            icon: const Icon(Icons.edit_outlined),
            label: Text(context.t('edit')),
            onPressed: () async {
              final result = await openPage<String>(
                context,
                BrewingRecipeEditor(c, record: record),
              );
              if (result != null && context.mounted) Navigator.pop(context);
            },
          ),
          TextButton(
            onPressed: () => runAction(context, () async {
              final result = await reviewBrewing(context, c, {
                'operation': 'archive_recipe',
                'id': record['id'],
                'archived': !archived,
              });
              if (result != null && context.mounted) Navigator.pop(context);
            }),
            child: Text(context.t(archived ? 'brew_restore' : 'brew_archive')),
          ),
        ],
      ],
    ),
  );
}
