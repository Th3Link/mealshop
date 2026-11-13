import 'dart:async';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'recipes.dart';
import 'sharing.dart';

class CollectionsScreen extends StatefulWidget {
  const CollectionsScreen(this.api, {super.key});
  final HouseholdApi api;
  @override
  State<CollectionsScreen> createState() => _CollectionsScreenState();
}

class _CollectionsScreenState extends State<CollectionsScreen> {
  int version = 0, page = 1;
  String query = '';
  Timer? timer;
  @override
  void dispose() {
    timer?.cancel();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('collections'))),
    floatingActionButton: widget.api.writable
        ? FloatingActionButton.extended(
            onPressed: () async {
              await openPage<void>(context, CollectionEditor(widget.api));
              if (mounted) setState(() => version++);
            },
            icon: const Icon(Icons.add),
            label: Text(context.t('new_collection')),
          )
        : null,
    body: Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: TextField(
            onChanged: (v) {
              timer?.cancel();
              timer = Timer(const Duration(milliseconds: 300), () {
                if (mounted) {
                  setState(() {
                    query = v;
                    page = 1;
                  });
                }
              });
            },
            decoration: InputDecoration(
              labelText: context.t('browse'),
              prefixIcon: const Icon(Icons.search),
            ),
          ),
        ),
        Expanded(
          child: Resource<Json>(
            key: ValueKey((version, page, query)),
            load: () => widget.api.call(
              '${widget.api.root}/collections?page=$page&per_page=50&q=${Uri.encodeQueryComponent(query)}',
            ),
            builder: (context, data, reload) => ListView(
              padding: const EdgeInsets.fromLTRB(20, 0, 20, 100),
              children: [
                if (objects(data['collections']).isEmpty)
                  Text(context.t('nothing_yet')),
                for (final collection in objects(data['collections']))
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    leading: const Icon(Icons.collections_bookmark_outlined),
                    title: Text(collection['name'] as String),
                    subtitle: Text(
                      '${collection['count']} ${context.t('recipes')}',
                    ),
                    trailing: const Icon(Icons.chevron_right),
                    onTap: () async {
                      await openPage<void>(
                        context,
                        CollectionDetail(
                          widget.api,
                          collection['id'] as String,
                        ),
                      );
                      await reload();
                    },
                  ),
                PageControls(
                  page,
                  data['total'] as int,
                  (v) => setState(() => page = v),
                ),
              ],
            ),
          ),
        ),
      ],
    ),
  );
}

class CollectionDetail extends StatefulWidget {
  const CollectionDetail(this.api, this.id, {super.key});
  final HouseholdApi api;
  final String id;
  @override
  State<CollectionDetail> createState() => _CollectionDetailState();
}

class _CollectionDetailState extends State<CollectionDetail> {
  int page = 1;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('collections'))),
    body: Resource<Json>(
      key: ValueKey(page),
      load: () => widget.api.call(
        '${widget.api.root}/collections/${widget.id}?page=$page&per_page=50',
      ),
      builder: (context, data, reload) {
        final collection = object(data['collection']);
        return ListView(
          padding: const EdgeInsets.all(20),
          children: [
            Text(
              collection['name'] as String,
              style: Theme.of(context).textTheme.headlineMedium,
            ),
            Text(collection['description'] as String),
            Wrap(
              spacing: 8,
              children: [
                if (data['editable'] == true)
                  OutlinedButton.icon(
                    onPressed: () async {
                      await openPage<void>(
                        context,
                        CollectionEditor(widget.api, record: data),
                      );
                      await reload();
                    },
                    icon: const Icon(Icons.edit_outlined),
                    label: Text(context.t('edit')),
                  ),
                if (widget.api.owner)
                  OutlinedButton.icon(
                    onPressed: () => openPage<void>(
                      context,
                      ShareEditor(
                        widget.api,
                        resource: {
                          'kind': 'collection',
                          'collection_id': widget.id,
                        },
                      ),
                    ),
                    icon: const Icon(Icons.share_outlined),
                    label: Text(context.t('share')),
                  ),
              ],
            ),
            SectionTitle(context.t('pinned')),
            for (final item in objects(data['items']))
              ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(item['title'] as String),
                subtitle: Text('${context.t('revision')} ${item['revision']}'),
                onTap: () => openPage<void>(
                  context,
                  RecipeDetail(
                    widget.api,
                    item['recipe_id'] as String,
                    revision: item['revision'] as int,
                  ),
                ),
              ),
            PageControls(
              page,
              (collection['entries'] as List).length,
              (v) => setState(() => page = v),
            ),
            if (data['editable'] == true)
              TextButton.icon(
                onPressed: () => runAction(context, () async {
                  if (!await confirm(context, 'confirm_delete')) return;
                  await widget.api.call(
                    '${widget.api.root}/collections/${widget.id}',
                    method: 'DELETE',
                    body: {'expected_revision': data['revision']},
                  );
                  if (context.mounted) Navigator.pop(context);
                }),
                icon: const Icon(Icons.delete_outline),
                label: Text(context.t('delete')),
              ),
          ],
        );
      },
    ),
  );
}

class CollectionEditor extends StatefulWidget {
  const CollectionEditor(this.api, {super.key, this.record});
  final HouseholdApi api;
  final Json? record;
  @override
  State<CollectionEditor> createState() => _CollectionEditorState();
}

class _CollectionEditorState extends State<CollectionEditor> {
  final form = GlobalKey<FormState>();
  late Json collection;
  int? revision;
  bool dirty = false, busy = false;
  List<Json> recipes = [];
  String? failure;
  bool loaded = false;
  @override
  void initState() {
    super.initState();
    collection = widget.record == null
        ? {'name': '', 'description': '', 'entries': <Json>[]}
        : clone(object(widget.record!['collection']));
    unawaited(load());
  }

  Future<void> load() async {
    try {
      revision =
          widget.record?['revision'] as int? ??
          (await widget.api.call(
                '${widget.api.root}/collections?per_page=1',
              ))['revision']
              as int;
      final values = objects((await widget.api.recipes())['recipes']);
      if (mounted) {
        setState(() {
          recipes = values;
          loaded = true;
        });
      }
    } catch (e) {
      if (mounted) {
        setState(() => failure = e is ApiFailure ? e.code : 'request_failed');
      }
    }
  }

  void change(VoidCallback action) => setState(() {
    action();
    dirty = true;
  });
  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: dirty,
    child: Scaffold(
      appBar: AppBar(
        title: Text(
          context.t(widget.record == null ? 'new_collection' : 'edit'),
        ),
      ),
      body: !loaded
          ? failure == null
                ? const Center(child: CircularProgressIndicator())
                : EmptyContent(
                    icon: Icons.cloud_off_outlined,
                    title: failure!,
                    action: 'retry',
                    onAction: load,
                  )
          : Form(
              key: form,
              child: ListView(
                padding: const EdgeInsets.all(20),
                children: [
                  EditText(
                    'name',
                    collection['name'] as String,
                    (v) => change(() => collection['name'] = v),
                    required: true,
                  ),
                  EditText(
                    'description',
                    collection['description'] as String,
                    (v) => change(() => collection['description'] = v),
                    lines: 3,
                    max: 2000,
                  ),
                  SectionTitle(context.t('pinned')),
                  for (final entry in objects(collection['entries']))
                    ListTile(
                      contentPadding: EdgeInsets.zero,
                      title: Text(
                        recipes
                                    .where((r) => r['id'] == entry['recipe_id'])
                                    .firstOrNull?['title']
                                as String? ??
                            entry['recipe_id'] as String,
                      ),
                      subtitle: Text(
                        '${context.t('revision')} ${entry['revision']}',
                      ),
                      trailing: IconButton(
                        tooltip: context.t('delete'),
                        icon: const Icon(Icons.delete_outline),
                        onPressed: () => change(
                          () => (collection['entries'] as List).remove(entry),
                        ),
                      ),
                      onTap: () => openPage<void>(
                        context,
                        RecipeDetail(
                          widget.api,
                          entry['recipe_id'] as String,
                          revision: entry['revision'] as int,
                        ),
                      ),
                    ),
                  OutlinedButton.icon(
                    onPressed: () async {
                      final id = await showModalBottomSheet<String>(
                        context: context,
                        isScrollControlled: true,
                        useSafeArea: true,
                        showDragHandle: true,
                        builder: (context) => DraggableScrollableSheet(
                          expand: false,
                          builder: (context, scroll) => ListView(
                            controller: scroll,
                            children: [
                              for (final r in recipes)
                                ListTile(
                                  title: Text(r['title'] as String),
                                  subtitle: Text(
                                    '${context.t('revision')} ${r['revision']}',
                                  ),
                                  onTap: () =>
                                      Navigator.pop(context, r['id'] as String),
                                ),
                            ],
                          ),
                        ),
                      );
                      if (id == null || !context.mounted) return;
                      final chosen = recipes.firstWhere((r) => r['id'] == id);
                      final rev = await askName(
                        context,
                        'revision',
                        initial: '${chosen['revision']}',
                        field: 'revision',
                        integer: true,
                      );
                      if (rev == null || !context.mounted) return;
                      await runAction(context, () async {
                        await widget.api.recipes('/$id?revision=$rev');
                        if (objects(collection['entries'])
                            .any((e) => e['recipe_id'] == id)) {
                          throw const ApiFailure('duplicate_reference');
                        }
                        change(
                          () => (collection['entries'] as List).add({
                            'recipe_id': id,
                            'revision': int.parse(rev),
                          }),
                        );
                      });
                    },
                    icon: const Icon(Icons.add),
                    label: Text(context.t('recipes')),
                  ),
                  const SizedBox(height: 20),
                  FilledButton(
                    onPressed: revision == null || busy
                        ? null
                        : () async {
                            if (!form.currentState!.validate()) return;
                            setState(() => busy = true);
                            try {
                              await widget.api.call(
                                '${widget.api.root}/collections${widget.record == null ? '' : '/${object(widget.record!['collection'])['id']}'}',
                                method: widget.record == null ? 'POST' : 'PUT',
                                body: {
                                  'expected_revision': revision,
                                  'collection': {...collection}..remove('id'),
                                },
                              );
                              dirty = false;
                              if (context.mounted) Navigator.pop(context);
                            } catch (e) {
                              if (context.mounted) {
                                await showFailure(context, e);
                              }
                            } finally {
                              if (mounted) setState(() => busy = false);
                            }
                          },
                    child: Text(context.t('save')),
                  ),
                ],
              ),
            ),
    ),
  );
}
