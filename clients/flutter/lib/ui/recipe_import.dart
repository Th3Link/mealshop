import 'package:material_ui/material_ui.dart';

import '../capture/import.dart';
import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'recipe_editor.dart';
import 'capture_photo.dart';
import 'capture_classify.dart';

class RecipeImport extends StatefulWidget {
  const RecipeImport(this.api, {super.key, this.sharedText});
  final HouseholdApi api;
  final String? sharedText;
  @override
  State<RecipeImport> createState() => _RecipeImportState();
}

class _RecipeImportState extends State<RecipeImport> {
  final url = TextEditingController(),
      title = TextEditingController(),
      ingredients = TextEditingController(),
      steps = TextEditingController();
  Json? preview, photo;
  bool busy = false;
  String? error;
  @override
  void initState() {
    super.initState();
    final text = widget.sharedText?.trim() ?? '';
    if (Uri.tryParse(text)?.scheme == 'https' &&
        !RegExp(r'\s').hasMatch(text)) {
      url.text = text;
    }
  }

  @override
  void dispose() {
    for (final c in [url, title, ingredients, steps]) {
      c.dispose();
    }
    super.dispose();
  }

  Future<void> fetch(bool remote) async {
    if (remote &&
        !await confirm(context, 'capture_fetch', body: 'capture_fetch_body')) {
      return;
    }
    setState(() {
      busy = true;
      error = null;
    });
    try {
      final value = await widget.api.household(
        '/imports/preview',
        method: 'POST',
        body: {
          'mode': remote ? 'url' : 'manual',
          'url': url.text.trim(),
          if (remote) 'consent': true,
        },
      );
      if (mounted) setState(() => preview = value);
    } catch (e) {
      if (mounted) {
        setState(() => error = e is ApiFailure ? e.code : 'request_failed');
      }
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> fromPhoto() async {
    final value = await openPage<Json>(context, CapturePhoto(widget.api));
    if (value != null && mounted) {
      setState(() {
        photo = value;
        preview = null;
      });
      if ((value['text'] as String).trim().isNotEmpty) await classify();
    }
  }

  Future<void> classify() async {
    final result = await openPage<Json>(
      context,
      CaptureClassify(photo!['text'] as String),
    );
    if (result == null || !mounted) return;
    setState(() {
      title.text = result['title'] as String? ?? '';
      ingredients.text = (result['ingredients'] as List).join('\n');
      steps.text = objects(result['steps']).map((s) => s['text']).join('\n');
    });
  }

  Future<void> preparePhoto() async {
    setState(() => busy = true);
    try {
      final original = blankCandidate();
      original['title'] = title.text.trim().isEmpty ? null : title.text.trim();
      original['ingredients'] = ingredients.text
          .split('\n')
          .where((s) => s.trim().isNotEmpty)
          .toList();
      original['steps'] = [
        for (final s
            in steps.text.split('\n').where((s) => s.trim().isNotEmpty))
          {'text': s, 'section': null},
      ];
      original['language'] = widget.api.controller.language;
      final value = await widget.api.household(
        '/imports/preview',
        method: 'POST',
        body: {'mode': 'photo', 'capture': photo, 'original': original},
      );
      if (mounted) setState(() => preview = value);
    } catch (e) {
      if (mounted) await showFailure(context, e);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> review(int candidate) async {
    final session = ImportSession(widget.api, preview!, candidate);
    await session.retain(session.seed());
    if (!mounted) return;
    final id = await openPage<String>(
      context,
      RecipeEditor(widget.api, importSession: session),
    );
    if (mounted && id != null) Navigator.pop(context, id);
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('capture_import'))),
    body: ListView(
      padding: const EdgeInsets.all(20),
      children: [
        Text(
          widget.api.controller.home?.name ?? '',
          style: Theme.of(context).textTheme.titleMedium,
        ),
        if (widget.sharedText != null) ...[
          SectionTitle(context.t('capture_incoming')),
          SelectableText(widget.sharedText!),
        ],
        if (widget.api.controller.draft(ImportSession.key) != null)
          ListTile(
            leading: const Icon(Icons.restore),
            title: Text(context.t('capture_resume')),
            onTap: () async {
              try {
                final session = ImportSession.restore(widget.api);
                if (session == null) return;
                final id = await openPage<String>(
                  context,
                  RecipeEditor(widget.api, importSession: session),
                );
                if (context.mounted && id != null) Navigator.pop(context, id);
              } catch (e) {
                if (context.mounted) await showFailure(context, e);
              }
            },
            trailing: IconButton(
              tooltip: context.t('delete'),
              icon: const Icon(Icons.delete_outline),
              onPressed: () async {
                if (await confirm(
                  context,
                  'capture_discard',
                  body: 'capture_discard_body',
                )) {
                  await widget.api.controller.saveDraft(
                    ImportSession.key,
                    null,
                  );
                  if (mounted) setState(() {});
                }
              },
            ),
          ),
        if (widget.api.controller.draft(ImportSession.key) == null) ...[
          SectionTitle(context.t('capture_link')),
          TextField(
            controller: url,
            maxLength: 2048,
            keyboardType: TextInputType.url,
            decoration: InputDecoration(labelText: context.t('capture_url')),
          ),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              FilledButton(
                onPressed: busy || !widget.api.writable
                    ? null
                    : () => fetch(true),
                child: Text(context.t('capture_fetch')),
              ),
              TextButton(
                onPressed: busy || !widget.api.writable
                    ? null
                    : () => fetch(false),
                child: Text(context.t('capture_manual')),
              ),
              OutlinedButton.icon(
                onPressed: busy || !widget.api.writable ? null : fromPhoto,
                icon: const Icon(Icons.document_scanner_outlined),
                label: Text(context.t('capture_photo')),
              ),
            ],
          ),
          if (photo != null) ...[
            SectionTitle(context.t('capture_original')),
            SelectableText(photo!['text'] as String),
            Note(context.t('capture_classify')),
            EditText('title', title.text, (v) => title.text = v),
            TextField(
              controller: ingredients,
              minLines: 3,
              maxLines: 10,
              decoration: InputDecoration(labelText: context.t('ingredients')),
            ),
            const SizedBox(height: 12),
            TextField(
              controller: steps,
              minLines: 3,
              maxLines: 10,
              decoration: InputDecoration(labelText: context.t('steps')),
            ),
            FilledButton(
              onPressed: busy ? null : preparePhoto,
              child: Text(context.t('capture_review')),
            ),
          ],
          if (error != null) Note(context.t(error!)),
          if (busy) const LinearProgressIndicator(),
          if (preview != null) ...[
            Note(context.t('capture_candidate_help')),
            for (var i = 0; i < objects(preview!['candidates']).length; i++)
              ListTile(
                title: Text(
                  objects(preview!['candidates'])[i]['title'] as String? ??
                      context.t('capture_manual'),
                ),
                subtitle: Text(
                  '${(objects(preview!['candidates'])[i]['ingredients'] as List).length} ${context.t('ingredients')}',
                ),
                trailing: const Icon(Icons.chevron_right),
                onTap: busy ? null : () => review(i),
              ),
          ],
        ],
      ],
    ),
  );
}
