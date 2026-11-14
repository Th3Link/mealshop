import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'content_common.dart';
import 'ingredient_editor.dart';
import 'messages.dart';

Future<Json?> pickIngredient(
  BuildContext context,
  HouseholdApi api, {
  bool create = false,
}) => openPage<Json>(context, IngredientSearch(api, create: create));

class IngredientSearch extends StatefulWidget {
  const IngredientSearch(this.api, {super.key, this.create = false});
  final HouseholdApi api;
  final bool create;
  @override
  State<IngredientSearch> createState() => _IngredientSearchState();
}

class _IngredientSearchState extends State<IngredientSearch> {
  String query = '';
  int generation = 0;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('find_ingredient'))),
    body: Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: TextField(
            autofocus: true,
            onChanged: (v) => setState(() => query = v),
            decoration: InputDecoration(
              labelText: context.t('browse'),
              prefixIcon: const Icon(Icons.search),
            ),
          ),
        ),
        if (widget.create && widget.api.writable)
          Padding(
            padding: const EdgeInsets.symmetric(horizontal: 16),
            child: ListTile(
              contentPadding: EdgeInsets.zero,
              leading: const Icon(Icons.add_circle_outline),
              title: Text(context.t('new_ingredient')),
              subtitle: query.trim().isEmpty ? null : Text(query),
              onTap: () async {
                final id = await openPage<String>(
                  context,
                  IngredientEditor(widget.api, initialName: query),
                );
                if (id == null || !mounted) return;
                final catalogue = Catalogue(
                  await widget.api.household('/ingredients'),
                );
                if (context.mounted) Navigator.pop(context, catalogue.find(id));
              },
            ),
          ),
        Expanded(
          child: Resource<Catalogue>(
            key: ValueKey(generation),
            load: () async =>
                Catalogue(await widget.api.household('/ingredients')),
            builder: (context, catalogue, reload) {
              final items = catalogue.search(
                query,
                Localizations.localeOf(context).languageCode,
              );
              return items.isEmpty
                  ? EmptyContent(icon: Icons.search_off, title: 'no_results')
                  : ListView.builder(
                      itemCount: items.length,
                      itemBuilder: (context, i) {
                        final item = items[i];
                        final org = object(
                          item['organization'] ?? <String, dynamic>{},
                        );
                        final parent = org['group_id'] == null
                            ? null
                            : catalogue.find(org['group_id'] as String);
                        return ListTile(
                          leading: Icon(
                            org['is_group'] == true
                                ? Icons.folder_outlined
                                : Icons.eco_outlined,
                          ),
                          title: Text(
                            catalogue.name(
                              item,
                              Localizations.localeOf(context).languageCode,
                            ),
                          ),
                          subtitle: parent == null
                              ? Text(item['form'] as String)
                              : Text(
                                  catalogue.name(
                                    parent,
                                    Localizations.localeOf(context)
                                        .languageCode,
                                  ),
                                ),
                          onTap: () => Navigator.pop(context, item),
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
