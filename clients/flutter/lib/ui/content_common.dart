import 'dart:async';
import 'dart:typed_data';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'messages.dart';

String exactNumber(BuildContext context, String value) =>
    decimal(value, Localizations.localeOf(context).languageCode);

class Resource<T> extends StatefulWidget {
  const Resource({
    super.key,
    required this.load,
    required this.builder,
    this.sensitive = false,
    this.pollInterval = const Duration(seconds: 20),
  });
  final Future<T> Function() load;
  final Widget Function(BuildContext, T, Future<void> Function()) builder;
  final bool sensitive;
  final Duration? pollInterval;
  @override
  State<Resource<T>> createState() => _ResourceState<T>();
}

class _ResourceState<T> extends State<Resource<T>> with WidgetsBindingObserver {
  T? data;
  Object? error;
  bool loading = true;
  int generation = 0;
  Timer? timer;
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
    unawaited(reload());
    if (widget.sensitive && widget.pollInterval != null) {
      timer = Timer.periodic(widget.pollInterval!, (_) {
        if (!loading) unawaited(reload());
      });
    }
  }

  @override
  void dispose() {
    timer?.cancel();
    WidgetsBinding.instance.removeObserver(this);
    generation++;
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) {
    if (!widget.sensitive) return;
    if (state != AppLifecycleState.resumed) {
      generation++;
      setState(() {
        data = null;
        loading = true;
      });
    } else {
      unawaited(reload());
    }
  }

  Future<void> reload() async {
    final current = ++generation;
    setState(() {
      loading = true;
      error = null;
    });
    try {
      final value = await widget.load();
      if (mounted && current == generation) {
        setState(() {
          data = value;
          loading = false;
        });
      }
    } catch (e) {
      if (mounted && current == generation) {
        setState(() {
          data = null;
          error = e;
          loading = false;
        });
      }
    }
  }

  @override
  Widget build(BuildContext context) {
    if (error != null) {
      return EmptyContent(
        icon: Icons.cloud_off_outlined,
        title: error is ApiFailure
            ? (error as ApiFailure).code
            : 'request_failed',
        action: 'retry',
        onAction: reload,
      );
    }
    if (data == null) return const Center(child: CircularProgressIndicator());
    return RefreshIndicator(
      onRefresh: reload,
      child: widget.builder(context, data as T, reload),
    );
  }
}

class EmptyContent extends StatelessWidget {
  const EmptyContent({
    super.key,
    required this.icon,
    required this.title,
    this.body,
    this.action,
    this.onAction,
  });
  final IconData icon;
  final String title;
  final String? body, action;
  final VoidCallback? onAction;
  @override
  Widget build(BuildContext context) => ListView(
    padding: const EdgeInsets.all(24),
    children: [
      const SizedBox(height: 48),
      Icon(icon, size: 48),
      const SizedBox(height: 20),
      Text(
        context.t(title),
        textAlign: TextAlign.center,
        style: Theme.of(context).textTheme.titleLarge,
      ),
      if (body != null)
        Padding(
          padding: const EdgeInsets.only(top: 12),
          child: Text(context.t(body!), textAlign: TextAlign.center),
        ),
      if (action != null)
        Padding(
          padding: const EdgeInsets.only(top: 24),
          child: OutlinedButton(
            onPressed: onAction,
            child: Text(context.t(action!)),
          ),
        ),
    ],
  );
}

class Note extends StatelessWidget {
  const Note(this.text, {super.key, this.warning = false});
  final String text;
  final bool warning;
  @override
  Widget build(BuildContext context) => Container(
    margin: const EdgeInsets.symmetric(vertical: 8),
    padding: const EdgeInsets.all(12),
    decoration: BoxDecoration(
      color: warning
          ? Theme.of(context).colorScheme.tertiaryContainer
          : Theme.of(context).colorScheme.surfaceContainerHigh,
      borderRadius: BorderRadius.circular(12),
    ),
    child: Row(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        Icon(warning ? Icons.info_outline : Icons.lightbulb_outline, size: 20),
        const SizedBox(width: 10),
        Expanded(child: Text(text)),
      ],
    ),
  );
}

class EditText extends StatelessWidget {
  const EditText(
    this.label,
    this.value,
    this.changed, {
    super.key,
    this.required = false,
    this.lines = 1,
    this.numeric = false,
    this.signed = false,
    this.integer = false,
    this.max = 200,
    this.hint,
  });
  final String label, value;
  final ValueChanged<String> changed;
  final bool required, numeric, signed, integer;
  final int lines, max;
  final String? hint;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 6),
    child: TextFormField(
      key: ValueKey(label),
      initialValue: value,
      onChanged: changed,
      maxLines: lines,
      maxLength: max,
      keyboardType: numeric
          ? TextInputType.numberWithOptions(decimal: true, signed: signed)
          : null,
      decoration: InputDecoration(
        labelText: messages.containsKey(label) ? context.t(label) : label,
        helperText: hint == null ? null : context.t(hint!),
        counterText: '',
      ),
      validator: (text) {
        if (text == null || text.trim().isEmpty) {
          return required ? context.t('name_required') : null;
        }
        if (integer && int.tryParse(text) == null) {
          return context.t('integer_required');
        }
        if (numeric) {
          try {
            exactInput(
              text,
              Localizations.localeOf(context).languageCode,
              signed: signed,
            );
          } on ApiFailure {
            return context.t('quantity_invalid');
          }
        }
        return null;
      },
    ),
  );
}

class Choice extends StatelessWidget {
  const Choice(this.label, this.value, this.options, this.changed, {super.key});
  final String label;
  final String? value;
  final Map<String, String> options;
  final ValueChanged<String> changed;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 6),
    child: DropdownButtonFormField<String>(
      key: ValueKey((label, value)),
      initialValue: options.containsKey(value) ? value : null,
      isExpanded: true,
      decoration: InputDecoration(
        labelText: messages.containsKey(label) ? context.t(label) : label,
      ),
      items: options.entries
          .map(
            (e) => DropdownMenuItem(
              value: e.key,
              child: Text(
                e.value,
                maxLines: 2,
                overflow: TextOverflow.ellipsis,
              ),
            ),
          )
          .toList(),
      onChanged: (v) {
        if (v != null) changed(v);
      },
    ),
  );
}

Future<T?> openPage<T>(BuildContext context, Widget page) =>
    Navigator.push<T>(context, MaterialPageRoute(builder: (_) => page));
Future<void> runAction(
  BuildContext context,
  Future<void> Function() action,
) async {
  try {
    await action();
  } catch (e) {
    if (context.mounted) await showFailure(context, e);
  }
}

class PrivateImage extends StatefulWidget {
  const PrivateImage(this.api, this.path, {super.key});
  final HouseholdApi api;
  final String path;
  @override
  State<PrivateImage> createState() => _PrivateImageState();
}

class _PrivateImageState extends State<PrivateImage> {
  late final Future<Uint8List> bytes = widget.api.image(widget.path);
  @override
  Widget build(BuildContext context) => FutureBuilder<Uint8List>(
    future: bytes,
    builder: (context, snapshot) => snapshot.hasData
        ? Image.memory(
            snapshot.data!,
            cacheWidth: 1080,
            height: 200,
            width: double.infinity,
            fit: BoxFit.cover,
            excludeFromSemantics: true,
          )
        : const SizedBox(
            height: 100,
            child: Center(child: Icon(Icons.restaurant_outlined)),
          ),
  );
}

List<String> lines(String value) =>
    value.split('\n').map((v) => v.trim()).where((v) => v.isNotEmpty).toList();
String personDescription(Json person, Catalogue catalogue, String language) {
  final rules = object(person['rules'] ?? <String, dynamic>{});
  String name(Object? id) => catalogue.find(id as String) == null
      ? id
      : catalogue.name(catalogue.find(id)!, language);
  return [
    person['name'],
    message(rules['preset'] as String? ?? 'unrestricted', language),
    for (final key in [
      'allergies',
      'intolerances',
      'allergen_ids',
      'excluded_tags',
      'excluded_ingredients',
      'disliked_ingredients',
    ])
      '${message(key, language)}: ${(person[key] as List? ?? []).map((id) => key.endsWith('_ingredients') ? name(id) : id).join(', ')}',
    for (final e in objects(rules['exceptions'] ?? []))
      '${name(e['ingredient_id'])}${e['household_produced_only'] == true ? ' · ${message('household_only', language)}' : ''}',
    '${message('max_active', language)}: ${rules['max_active_seconds'] ?? '—'}',
    for (final w in objects(rules['availability'] ?? []))
      '${w['weekdays']} · ${message(w['meal'] as String, language)}',
  ].join('\n');
}
