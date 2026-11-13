import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'ingredients.dart';
import 'messages.dart';
import 'recipe_editor.dart';
import 'sharing.dart';

class RecipesScreen extends StatefulWidget {
  const RecipesScreen(this.api, {super.key});
  final HouseholdApi api;
  @override
  State<RecipesScreen> createState() => _RecipesScreenState();
}

class _RecipesScreenState extends State<RecipesScreen> {
  String query = '';
  int version = 0;
  @override
  Widget build(BuildContext context) => Scaffold(
    floatingActionButton: widget.api.writable
        ? FloatingActionButton.extended(
            onPressed: () async {
              final id = await openPage<String>(
                context,
                RecipeEditor(widget.api),
              );
              if (!context.mounted) return;
              setState(() => version++);
              if (id != null) {
                await openPage<void>(context, RecipeDetail(widget.api, id));
              }
            },
            icon: const Icon(Icons.add),
            label: Text(context.t('new_recipe')),
          )
        : null,
    body: Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: TextField(
            onChanged: (v) => setState(() => query = v.toLowerCase().trim()),
            decoration: InputDecoration(
              labelText: context.t('browse'),
              prefixIcon: const Icon(Icons.search),
            ),
          ),
        ),
        Expanded(
          child: Resource<Json>(
            key: ValueKey(version),
            load: () => widget.api.recipes(),
            builder: (context, value, reload) {
              final items = objects(value['recipes'])
                  .where(
                    (r) => (r['title'] as String).toLowerCase().contains(query),
                  )
                  .toList();
              if (items.isEmpty) {
                return EmptyContent(
                  icon: Icons.menu_book_outlined,
                  title: query.isEmpty ? 'nothing_yet' : 'no_results',
                  body: query.isEmpty ? 'online_content' : null,
                );
              }
              return ListView.builder(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 100),
                itemCount: items.length,
                itemBuilder: (context, i) {
                  final r = items[i];
                  return ListTile(
                    contentPadding: const EdgeInsets.symmetric(vertical: 8),
                    leading: const CircleAvatar(
                      child: Icon(Icons.restaurant_menu),
                    ),
                    title: Text(r['title'] as String),
                    subtitle: Text(
                      '${r['servings']} ${context.t('servings')} · ${context.t('revision')} ${r['revision']}',
                    ),
                    trailing: const Icon(Icons.chevron_right),
                    onTap: () async {
                      await openPage<void>(
                        context,
                        RecipeDetail(widget.api, r['id'] as String),
                      );
                      await reload();
                    },
                  );
                },
              );
            },
          ),
        ),
      ],
    ),
  );
}

class RecipeDetail extends StatefulWidget {
  const RecipeDetail(
    this.api,
    this.id, {
    super.key,
    this.revision,
    this.sharedPath,
  });
  final HouseholdApi api;
  final String id;
  final int? revision;
  final String? sharedPath;
  @override
  State<RecipeDetail> createState() => _RecipeDetailState();
}

class _RecipeDetailState extends State<RecipeDetail> {
  int version = 0;
  int? revision, servings;
  String unit = 'g';
  bool get shared => widget.sharedPath != null;
  @override
  void initState() {
    super.initState();
    revision = widget.revision;
  }

  Future<Json> load() async {
    if (shared) {
      final v = await widget.api.call(
        '${widget.sharedPath}${servings == null ? '' : '?servings=$servings&display_unit=$unit'}',
      );
      final content = object(v['content']);
      if (content['kind'] == 'source_link') {
        return {'source_link': content['source_url']};
      }
      final record = object(content['record']);
      return {...record, 'scaled': content['scaled']};
    }
    final r = await widget.api.recipes(
      '/${widget.id}${revision == null ? '' : '?revision=$revision'}',
    );
    if (servings != null) {
      r['scaled'] = await widget.api.recipes(
        '/${widget.id}/scale?revision=${r['revision']}&servings=$servings&display_unit=$unit',
      );
    }
    return r;
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: Text(context.t('recipes')),
      actions: [
        if (!shared && widget.api.writable && widget.revision == null)
          IconButton(
            tooltip: context.t('edit'),
            icon: const Icon(Icons.edit_outlined),
            onPressed: () => runAction(context, () async {
              final record = await widget.api.recipes('/${widget.id}');
              if (!context.mounted) return;
              await openPage<String>(
                context,
                RecipeEditor(widget.api, record: record),
              );
              if (mounted) {
                setState(() {
                  version++;
                  revision = null;
                  servings = null;
                });
              }
            }),
          ),
      ],
    ),
    body: Resource<Json>(
      key: ValueKey((version, revision, servings, unit)),
      sensitive: shared,
      load: load,
      builder: (context, data, reload) {
        if (data['source_link'] != null) {
          return ListView(
            padding: const EdgeInsets.all(20),
            children: [
              Note(context.t('source_only')),
              OutlinedButton.icon(
                onPressed: () => runAction(
                  context,
                  () => openSource(data['source_link'] as String),
                ),
                icon: const Icon(Icons.open_in_new),
                label: Text(context.t('open_source')),
              ),
            ],
          );
        }
        final recipe = object(data['recipe']);
        final scaled = data['scaled'] as Json?;
        final lines = objects(scaled?['lines'] ?? recipe['lines']);
        final display = objects(
          scaled?['display_lines'] ?? data['display_lines'] ?? [],
        );
        final parts = objects(recipe['parts'] ?? []);
        final names = <String, String>{
          for (final d in display)
            d['ingredient_id'] as String:
                (d['translations']
                        as Json?)?[widget.api.controller.language]?['name']
                    as String? ??
                d['name'] as String? ??
                d['ingredient_id'] as String,
        };
        String partName(String? id) =>
            parts.where((p) => p['id'] == id).firstOrNull?['name'] as String? ??
            context.t('whole_recipe');
        return ListView(
          padding: const EdgeInsets.all(20),
          children: [
            if (recipe['image_id'] != null)
              ClipRRect(
                borderRadius: BorderRadius.circular(16),
                child: PrivateImage(
                  widget.api,
                  shared
                      ? '${widget.sharedPath}/image'
                      : '${widget.api.domain}/household/images/${recipe['image_id']}',
                  key: ValueKey((widget.sharedPath, recipe['image_id'])),
                ),
              ),
            const SizedBox(height: 16),
            Text(
              recipe['title'] as String,
              style: Theme.of(context).textTheme.headlineMedium,
            ),
            if ((recipe['author'] as String? ?? '').isNotEmpty)
              Text(recipe['author'] as String),
            Text(
              '${context.t('revision')} ${data['revision']} · ${recipe['language']}',
            ),
            if (!shared)
              Wrap(
                children: [
                  IconButton(
                    tooltip: context.t('back'),
                    onPressed: (data['revision'] as int) > 1
                        ? () => setState(
                            () => revision = (data['revision'] as int) - 1,
                          )
                        : null,
                    icon: const Icon(Icons.history),
                  ),
                  TextButton(
                    onPressed: () => setState(() => revision = null),
                    child: Text(context.t('latest_revision')),
                  ),
                ],
              ),
            Wrap(
              spacing: 8,
              runSpacing: 8,
              children: [
                OutlinedButton.icon(
                  onPressed: () async {
                    final amount = await askName(
                      context,
                      'servings',
                      initial: '${servings ?? recipe['servings']}',
                      field: 'servings',
                      integer: true,
                    );
                    if (amount != null) {
                      setState(() => servings = int.parse(amount));
                    }
                  },
                  icon: const Icon(Icons.people_outline),
                  label: Text(
                    '${servings ?? recipe['servings']} ${context.t('servings')}',
                  ),
                ),
                OutlinedButton(
                  onPressed: () => setState(() {
                    unit = unit == 'g' ? 'kg' : 'g';
                    servings ??= recipe['servings'] as int;
                  }),
                  child: Text(unit),
                ),
                if (!shared && widget.api.owner)
                  OutlinedButton.icon(
                    onPressed: () => openPage<void>(
                      context,
                      ShareEditor(
                        widget.api,
                        resource: {
                          'kind': 'recipe',
                          'recipe_id': widget.id,
                          'revision': data['revision'],
                        },
                      ),
                    ),
                    icon: const Icon(Icons.share_outlined),
                    label: Text(context.t('share')),
                  ),
              ],
            ),
            if (object(
                  data['sharing_origin'] ??
                      recipe['origin'] ??
                      <String, dynamic>{},
                )['kind'] ==
                'imported')
              Note(context.t('source_only')),
            for (final partId in [
              null,
              ...parts.map((p) => p['id'] as String),
            ]) ...[
              if (parts.isNotEmpty &&
                  (partId != null ||
                      lines.any((line) => line['part_id'] == null)))
                SectionTitle(partName(partId)),
              if (partId != null)
                for (final key in ['uses_parts', 'starts_after'])
                  if ((parts.firstWhere((p) => p['id'] == partId)[key]
                              as List? ??
                          [])
                      .isNotEmpty)
                    Text(
                      '${context.t(key)}: ${(parts.firstWhere((p) => p['id'] == partId)[key] as List).map((id) => partName(id as String)).join(', ')}',
                    ),
              for (final line in lines.where((l) => l['part_id'] == partId))
                ListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(
                    names[line['ingredient_id']] ??
                        line['ingredient_id'] as String,
                  ),
                  subtitle: line['prepared_part'] != null
                      ? Text(
                          '${context.t('homemade')}: ${partName(line['prepared_part'] as String)}',
                        )
                      : line['quantity'] != null &&
                            object(line['quantity'])['unit'] !=
                                (display.isNotEmpty
                                    ? display[lines.indexOf(line)]['unit']
                                    : 'g')
                      ? Text(
                          '${decimal(object(line['quantity'])['amount'] as String, widget.api.controller.language)} ${object(line['quantity'])['unit']}',
                        )
                      : null,
                  trailing: Text(
                    '${display.isNotEmpty ? decimal(display[lines.indexOf(line)]['amount'] as String, widget.api.controller.language) : decimal(line['grams'] as String, widget.api.controller.language)} ${display.isNotEmpty ? display[lines.indexOf(line)]['unit'] : 'g'}',
                  ),
                  onTap: shared
                      ? null
                      : () => openPage<void>(
                          context,
                          IngredientDetail(
                            widget.api,
                            line['ingredient_id'] as String,
                          ),
                        ),
                ),
            ],
            SectionTitle(context.t('steps')),
            for (final (i, step) in objects(recipe['steps']).indexed) ...[
              ListTile(
                contentPadding: EdgeInsets.zero,
                leading: CircleAvatar(child: Text('${i + 1}')),
                title: Text(partName(step['part_id'] as String?)),
                subtitle: Text('${step['instruction']}'),
              ),
              if ((step['ingredient_line_ids'] as List? ?? []).isNotEmpty)
                Text(
                  (step['ingredient_line_ids'] as List)
                      .map((id) {
                        final l = lines.where((l) => l['id'] == id).firstOrNull;
                        return l == null
                            ? id
                            : names[l['ingredient_id']] ?? l['ingredient_id'];
                      })
                      .join(', '),
                ),
              if ((step['prepared_parts'] as List? ?? []).isNotEmpty)
                Text(
                  '${context.t('uses_parts')}: ${(step['prepared_parts'] as List).map((id) => partName(id as String)).join(', ')}',
                ),
              Text(
                '${context.t('active_seconds')}: ${step['active_seconds']} · ${context.t('passive_seconds')}: ${step['passive_seconds']}${step['passive_max_seconds'] == null ? '' : '–${step['passive_max_seconds']}'}',
              ),
              const Divider(),
            ],
            NutritionView(object(scaled?['nutrition'] ?? data['nutrition'])),
            if (!shared)
              OutlinedButton.icon(
                onPressed: () =>
                    openPage<void>(context, RecipeRules(widget.api, recipe)),
                icon: const Icon(Icons.fact_check_outlined),
                label: Text(context.t('check_rules')),
              ),
          ],
        );
      },
    ),
  );
}

class NutritionView extends StatelessWidget {
  const NutritionView(this.data, {super.key});
  final Json data;
  @override
  Widget build(BuildContext context) {
    final lang = Localizations.localeOf(context).languageCode;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        SectionTitle(context.t('nutrition')),
        Text(context.t('per_serving')),
        for (final row in objects(data['nutrients']))
          Builder(
            builder: (context) {
              final range = row['range'] as Json?;
              final bounds =
                  (range?['per_serving'] ?? range?['known_per_serving'])
                      as Json?;
              final estimated =
                  range != null &&
                  (range['estimated_ingredients'] as List).isNotEmpty;
              return Container(
                decoration: estimated
                    ? BoxDecoration(
                        color: Theme.of(context).colorScheme.tertiaryContainer,
                        borderRadius: BorderRadius.circular(8),
                      )
                    : null,
                child: ListTile(
                  contentPadding: const EdgeInsets.symmetric(horizontal: 8),
                  title: Text('${row['label']} (${row['unit']})'),
                  subtitle: Text(
                    context.t(
                      row['status'] == 'complete'
                          ? (estimated ? 'estimated' : 'per_serving')
                          : 'partial',
                    ),
                  ),
                  trailing: Text(
                    bounds != null
                        ? '${decimal(bounds['lower'] as String, lang)}–${decimal(bounds['upper'] as String, lang)}'
                        : row['per_serving'] == null
                        ? '—'
                        : decimal(row['per_serving'] as String, lang),
                  ),
                ),
              );
            },
          ),
        Note(context.t('nutrition_unknown')),
        for (final c in objects(data['constituents'] ?? []))
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(
              (c['translations'] as Json?)?[lang] as String? ??
                  c['label'] as String,
            ),
            subtitle: c['status'] == 'complete'
                ? null
                : Text(context.t('partial')),
            trailing: Text(
              '${c['per_serving'] == null ? '—' : decimal(c['per_serving'] as String, lang)} ${c['unit']}',
            ),
          ),
        for (final item in objects(data['ingredients'] ?? [])) ...[
          SectionTitle(item['name'] as String),
          if (item['source'] != null) SourceTile(object(item['source'])),
          for (final estimate in objects(
            item['nutrition_estimates'] ?? [],
          )) ...[
            Note(estimate['rationale'] as String, warning: true),
            for (final reference in objects(estimate['references']))
              SourceTile(
                object(reference['source']),
                name: reference['name'] as String,
              ),
          ],
        ],
      ],
    );
  }
}

class RecipeRules extends StatelessWidget {
  const RecipeRules(this.api, this.recipe, {super.key});
  final HouseholdApi api;
  final Json recipe;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('check_rules'))),
    body: Resource<Json>(
      load: () async {
        final profiles = await api.household('/profiles');
        if (objects(profiles['members']).isEmpty) return {'empty': true};
        return await api.household(
          '/evaluate',
          method: 'POST',
          body: {
            'member_ids': objects(profiles['members'])
                .map((m) => m['id'])
                .toList(),
            'ingredient_ids': objects(recipe['lines'])
                .map((l) => l['ingredient_id'])
                .toSet()
                .toList(),
          },
        );
      },
      builder: (context, data, reload) => ListView(
        padding: const EdgeInsets.all(20),
        children: [
          Note(context.t('allergen_review'), warning: true),
          if (data['empty'] == true)
            Text(context.t('nothing_yet'))
          else ...[
            Text(
              context.t(data['status'] as String),
              style: Theme.of(context).textTheme.titleLarge,
            ),
            for (final rule in objects(data['dietary_rules'])) ...[
              SectionTitle(
                objects(data['review_notes'])
                            .where((m) => m['member_id'] == rule['member_id'])
                            .firstOrNull?['name']
                        as String? ??
                    '',
              ),
              Text(context.t(rule['preset'] as String)),
              for (final key in [
                'excluded_ingredients',
                'unknown_ingredients',
                'exceptions',
              ])
                if ((rule[key] as List).isNotEmpty)
                  Text('${context.t(key)}: ${(rule[key] as List).join(', ')}'),
            ],
            for (final key in [
              'excluded_ingredients',
              'disliked_ingredients',
              'unknown_ingredients',
            ])
              if ((data[key] as List).isNotEmpty)
                Text('${context.t(key)}: ${(data[key] as List).join(', ')}'),
          ],
        ],
      ),
    ),
  );
}
