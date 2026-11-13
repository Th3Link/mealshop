import 'package:material_ui/material_ui.dart';
import 'package:url_launcher/url_launcher.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'ingredient_editor.dart';
import 'messages.dart';
import 'recipes.dart';

Future<void> openSource(String value) async {
  final uri = Uri.tryParse(value);
  if (uri == null ||
      uri.scheme != 'https' ||
      uri.host.isEmpty ||
      uri.userInfo.isNotEmpty) {
    throw const ApiFailure('invalid_response');
  }
  if (!await launchUrl(uri, mode: LaunchMode.externalApplication)) {
    throw const ApiFailure('request_failed');
  }
}

class IngredientsScreen extends StatefulWidget {
  const IngredientsScreen(this.api, {super.key});
  final HouseholdApi api;
  @override
  State<IngredientsScreen> createState() => _IngredientsScreenState();
}

class _IngredientsScreenState extends State<IngredientsScreen> {
  String query = '', group = '';
  bool archived = false;
  int version = 0;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('ingredients'))),
    floatingActionButton: widget.api.writable
        ? FloatingActionButton.extended(
            onPressed: () async {
              await openPage<String>(context, IngredientEditor(widget.api));
              if (mounted) setState(() => version++);
            },
            icon: const Icon(Icons.add),
            label: Text(context.t('new_ingredient')),
          )
        : null,
    body: Column(
      children: [
        Padding(
          padding: const EdgeInsets.fromLTRB(16, 8, 16, 8),
          child: TextField(
            onChanged: (v) => setState(() => query = v),
            decoration: InputDecoration(
              labelText: context.t('browse'),
              prefixIcon: const Icon(Icons.search),
            ),
          ),
        ),
        Expanded(
          child: Resource<Catalogue>(
            key: ValueKey(version),
            load: () async =>
                Catalogue(await widget.api.household('/ingredients')),
            builder: (context, data, reload) {
              final lang = widget.api.controller.language;
              final items = data
                  .search(query, lang, archived: archived)
                  .where(
                    (i) =>
                        group.isEmpty ||
                        i['id'] == group ||
                        object(
                              i['organization'] ?? <String, dynamic>{},
                            )['group_id'] ==
                            group,
                  )
                  .toList();
              return ListView(
                padding: const EdgeInsets.fromLTRB(16, 0, 16, 100),
                children: [
                  Choice('ingredient_group', group, {
                    '': context.t('all_groups'),
                    for (final g in data.items.where(
                      (i) =>
                          object(
                            i['organization'] ?? <String, dynamic>{},
                          )['is_group'] ==
                          true,
                    ))
                      g['id'] as String: data.name(g, lang),
                  }, (v) => setState(() => group = v)),
                  SwitchListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(context.t('include_archived')),
                    value: archived,
                    onChanged: (v) => setState(() => archived = v),
                  ),
                  if (items.isEmpty)
                    Padding(
                      padding: const EdgeInsets.all(24),
                      child: Text(context.t('no_results')),
                    ),
                  for (final item in items)
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      leading: Icon(
                        item['favorite'] == true
                            ? Icons.star
                            : object(
                                    item['organization'] ?? <String, dynamic>{},
                                  )['is_group'] ==
                                  true
                            ? Icons.folder_outlined
                            : Icons.eco_outlined,
                      ),
                      title: Text(data.name(item, lang)),
                      subtitle: Text(item['form'] as String),
                      trailing: const Icon(Icons.chevron_right),
                      onTap: () async {
                        await openPage<void>(
                          context,
                          IngredientDetail(widget.api, item['id'] as String),
                        );
                        await reload();
                      },
                    ),
                ],
              );
            },
          ),
        ),
      ],
    ),
  );
}

class IngredientDetail extends StatefulWidget {
  const IngredientDetail(this.api, this.id, {super.key});
  final HouseholdApi api;
  final String id;
  @override
  State<IngredientDetail> createState() => _IngredientDetailState();
}

class _IngredientDetailState extends State<IngredientDetail> {
  int version = 0;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: Text(context.t('ingredient_name')),
      actions: [
        if (widget.api.writable)
          IconButton(
            tooltip: context.t('edit'),
            onPressed: () async {
              await openPage<String>(
                context,
                IngredientEditor(widget.api, id: widget.id),
              );
              if (mounted) setState(() => version++);
            },
            icon: const Icon(Icons.edit_outlined),
          ),
      ],
    ),
    body: Resource<Catalogue>(
      key: ValueKey(version),
      load: () async => Catalogue(await widget.api.household('/ingredients')),
      builder: (context, catalogue, reload) {
        final item = catalogue.find(widget.id);
        if (item == null) {
          return EmptyContent(icon: Icons.search_off, title: 'no_results');
        }
        final source = object(item['source']);
        final org = object(item['organization'] ?? <String, dynamic>{});
        final lang = widget.api.controller.language;
        return ListView(
          padding: const EdgeInsets.all(20),
          children: [
            Text(
              catalogue.name(item, lang),
              style: Theme.of(context).textTheme.headlineMedium,
            ),
            Text(item['form'] as String),
            if (org['group_id'] != null)
              ListTile(
                contentPadding: EdgeInsets.zero,
                leading: const Icon(Icons.folder_outlined),
                title: Text(
                  catalogue.name(
                    catalogue.find(org['group_id'] as String)!,
                    lang,
                  ),
                ),
                onTap: () => openPage<void>(
                  context,
                  IngredientDetail(widget.api, org['group_id'] as String),
                ),
              ),
            Wrap(
              spacing: 8,
              children: [
                for (final key in ['diet', 'origin', 'production'])
                  Chip(
                    label: Text(
                      context.t(
                        org[key] == 'unknown'
                            ? 'unknown_value'
                            : org[key] == 'purchased'
                            ? 'purchased_decl'
                            : org[key] as String? ?? 'unknown_value',
                      ),
                    ),
                  ),
              ],
            ),
            if (widget.api.writable)
              SwitchListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(context.t('favorite')),
                value: item['favorite'] as bool? ?? false,
                onChanged: (v) => runAction(context, () async {
                  final annotations = {
                    for (final key in [
                      'organization',
                      'tags',
                      'components',
                      'declared_allergens',
                      'archived',
                      'favorite',
                    ])
                      key: item[key],
                  };
                  annotations['favorite'] = v;
                  await widget.api.household(
                    '/ingredients/${widget.id}',
                    method: 'PUT',
                    body: {
                      'expected_revision': catalogue.revision,
                      'annotations': annotations,
                    },
                  );
                  await reload();
                }),
              ),
            Note(context.t('allergen_review'), warning: true),
            Wrap(
              spacing: 8,
              children: [
                for (final tag in [
                  ...item['tags'] as List,
                  ...item['declared_allergens'] as List,
                ])
                  Chip(label: Text(tag as String)),
              ],
            ),
            SectionTitle(context.t('nutrition')),
            Text(context.t('per_100g')),
            for (final definition in catalogue.definitions.where(
              (d) => !['beta_glucan', 'soluble_fibre'].contains(d['id']),
            ))
              Builder(
                builder: (context) {
                  final reading = object(
                    object(item['nutrients'])[definition['id']] ??
                        <String, dynamic>{},
                  );
                  final range = reading['range'] as Json?;
                  return ListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(
                      '${definition['label']} (${definition['unit']})',
                    ),
                    subtitle: (reading['uncertainty'] as String? ?? '').isEmpty
                        ? null
                        : Text(reading['uncertainty'] as String),
                    trailing: Text(
                      range != null
                          ? '${decimal(range['lower'] as String, lang)}–${decimal(range['upper'] as String, lang)}'
                          : reading['amount'] == null
                          ? '—'
                          : decimal(reading['amount'] as String, lang),
                    ),
                  );
                },
              ),
            Note(context.t('nutrition_unknown')),
            for (final profile in objects(
              item['nutrition_estimates'] ?? [],
            )) ...[
              Note(
                '${context.t('estimated')}: ${profile['rationale']}',
                warning: true,
              ),
              for (final reference in objects(profile['references']))
                SourceTile(
                  object(reference['source']),
                  name: reference['name'] as String,
                ),
            ],
            if (object(item['constituents'] ?? <String, dynamic>{}).isNotEmpty)
              SectionTitle(context.t('constituents')),
            for (final d in object(
              item['constituents'] ?? <String, dynamic>{},
            ).values)
              ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(object(d)['name'] as String),
                trailing: Text(
                  '${object(d)['amount'] == null ? '—' : decimal(object(d)['amount'] as String, lang)} ${object(d)['unit']}',
                ),
              ),
            SectionTitle(context.t('source')),
            SourceTile(source),
            if (item['reference_source'] != null)
              SourceTile(object(item['reference_source'])),
            for (final component in List<String>.from(
              item['components'] as List? ?? [],
            ))
              ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(catalogue.name(catalogue.find(component)!, lang)),
                leading: const Icon(Icons.eco_outlined),
                onTap: () => openPage<void>(
                  context,
                  IngredientDetail(widget.api, component),
                ),
              ),
            const SizedBox(height: 20),
            OutlinedButton.icon(
              onPressed: () => openPage<void>(
                context,
                IngredientRecipes(widget.api, widget.id),
              ),
              icon: const Icon(Icons.menu_book_outlined),
              label: Text(context.t('reverse_recipes')),
            ),
          ],
        );
      },
    ),
  );
}

class SourceTile extends StatelessWidget {
  const SourceTile(this.source, {super.key, this.name});
  final Json source;
  final String? name;
  @override
  Widget build(BuildContext context) => ListTile(
    contentPadding: EdgeInsets.zero,
    title: Text(name ?? source['name'] as String),
    subtitle: Text(
      '${source['record_id']} · ${source['version']}\n${source['license']}',
    ),
    isThreeLine: true,
    trailing: (source['url'] as String).isEmpty
        ? null
        : const Icon(Icons.open_in_new),
    onTap: (source['url'] as String).isEmpty
        ? null
        : () => runAction(context, () => openSource(source['url'] as String)),
  );
}

class IngredientRecipes extends StatelessWidget {
  const IngredientRecipes(this.api, this.id, {super.key});
  final HouseholdApi api;
  final String id;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('reverse_recipes'))),
    body: Resource<Json>(
      load: () => api.household('/ingredients/$id/recipes'),
      builder: (context, data, reload) {
        final items = objects(data['recipes']);
        return items.isEmpty
            ? EmptyContent(icon: Icons.menu_book_outlined, title: 'nothing_yet')
            : ListView(
                children: [
                  for (final r in items)
                    ListTile(
                      title: Text(r['title'] as String),
                      trailing: const Icon(Icons.chevron_right),
                      onTap: () => openPage<void>(
                        context,
                        RecipeDetail(
                          api,
                          r['id'] as String,
                          revision: r['revision'] as int,
                        ),
                      ),
                    ),
                ],
              );
      },
    ),
  );
}
