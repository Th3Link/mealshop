import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../shopping/controller.dart';
import '../shopping/quantity.dart';
import '../shopping/models.dart';
import 'common.dart';
import 'messages.dart';

Future<Json?> chooseIngredient(BuildContext context, ShoppingController c) =>
    Navigator.push<Json>(
      context,
      MaterialPageRoute(builder: (_) => IngredientPicker(c)),
    );

class IngredientPicker extends StatefulWidget {
  const IngredientPicker(this.controller, {super.key});
  final ShoppingController controller;
  @override
  State<IngredientPicker> createState() => _IngredientPickerState();
}

class _IngredientPickerState extends State<IngredientPicker> {
  late Future<List<Json>> request;
  String search = '';
  @override
  void initState() {
    super.initState();
    request = widget.controller.ingredients();
  }

  String label(Json item) {
    final tr =
        (item['translations'] as Json?)?[widget.controller.language] as Json?;
    return tr?['name'] as String? ?? item['name'] as String;
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('find_ingredient'))),
    body: Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: TextField(
            autofocus: true,
            onChanged: (v) => setState(() => search = v.toLowerCase()),
            decoration: InputDecoration(
              labelText: context.t('search'),
              prefixIcon: const Icon(Icons.search),
            ),
          ),
        ),
        Expanded(
          child: FutureBuilder<List<Json>>(
            future: request,
            builder: (context, snapshot) {
              if (snapshot.hasError) {
                return Center(
                  child: FilledButton(
                    onPressed: () => setState(
                      () => request = widget.controller.ingredients(),
                    ),
                    child: Text(context.t('retry')),
                  ),
                );
              }
              if (!snapshot.hasData) {
                return const Center(child: CircularProgressIndicator());
              }
              final items = snapshot.data!
                  .where(
                    (i) => [
                      i['name'],
                      label(i),
                      i['aliases'],
                      i['translations'],
                    ].join(' ').toLowerCase().contains(search),
                  )
                  .take(80)
                  .toList();
              if (items.isEmpty) {
                return Center(child: Text(context.t('no_results')));
              }
              return ListView.builder(
                itemCount: items.length,
                itemBuilder: (context, index) {
                  final i = items[index];
                  return ListTile(
                    title: Text(label(i)),
                    subtitle: label(i) == i['name']
                        ? null
                        : Text(i['name'] as String),
                    trailing: const Icon(Icons.add),
                    onTap: () => Navigator.pop(context, i),
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

class AddItemScreen extends StatefulWidget {
  const AddItemScreen(this.controller, {super.key, this.source, this.pending});
  final ShoppingController controller;
  final Json? source;
  final Pending? pending;
  @override
  State<AddItemScreen> createState() => _AddItemScreenState();
}

class _AddItemScreenState extends State<AddItemScreen> {
  final form = GlobalKey<FormState>();
  final name = TextEditingController(),
      quantity = TextEditingController(text: '1');
  String unit = 'piece';
  String? ingredientId, ingredientName;
  bool saving = false, dirty = false;
  int? openedRevision;
  @override
  void initState() {
    super.initState();
    openedRevision = widget.controller.shopping?.revision;
    final pending = widget.pending;
    if (pending != null) {
      final command = pending.command;
      final q = object(command['quantity']);
      name.text = command['label'] as String;
      quantity.text = (q['amount'] as String).replaceAll(
        '.',
        widget.controller.language == 'de' ? ',' : '.',
      );
      unit = q['unit'] as String;
      ingredientId = command['ingredient_id'] as String?;
      ingredientName = name.text;
    }
    final source = widget.source;
    if (source != null) {
      final origin = object(source['origin']);
      final q = object(origin['quantity']);
      name.text = origin['label'] as String;
      quantity.text = (q['amount'] as String).replaceAll(
        '.',
        widget.controller.language == 'de' ? ',' : '.',
      );
      unit = q['unit'] as String;
      final lines = source['requirements'] as List;
      ingredientId =
          object(object(lines.first)['item'])['ingredient_id'] as String?;
      ingredientName = name.text;
    }
  }

  @override
  void dispose() {
    name.dispose();
    quantity.dispose();
    super.dispose();
  }

  Future<void> save() async {
    if (!form.currentState!.validate()) return;
    setState(() => saving = true);
    try {
      final q = Quantity.parse(quantity.text, unit, widget.controller.language);
      final command = <String, dynamic>{
        'operation': 'add_manual',
        if (widget.source != null) 'source_id': widget.source!['id'],
        if (widget.pending?.command['source_id'] != null)
          'source_id': widget.pending!.command['source_id'],
        'label': name.text.trim(),
        'ingredient_id': ingredientId,
        'quantity': q.toJson(),
        if (widget.pending?.command['purchased'] == true) 'purchased': true,
      };
      if (widget.pending != null) {
        await widget.controller.replaceReviewed(
          widget.pending!.id,
          command,
          original: q.original,
          expectedRevision: openedRevision,
        );
      } else {
        await widget.controller.enqueue(
          command,
          original: q.original,
          offline: widget.source == null,
          expectedRevision: openedRevision,
        );
      }
      if (mounted) Navigator.pop(context);
    } catch (e) {
      if (mounted) await showFailure(context, e);
    } finally {
      if (mounted) setState(() => saving = false);
    }
  }

  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: Text(context.t(widget.source == null ? 'add' : 'edit')),
    ),
    body: DraftGuard(
      dirty: () => dirty,
      child: SafeArea(
        child: Form(
          onChanged: () => setState(() => dirty = true),
          key: form,
          child: ListView(
            padding: const EdgeInsets.all(20),
            children: [
              TextFormField(
                controller: name,
                autofocus: widget.source == null,
                maxLength: 200,
                decoration: InputDecoration(labelText: context.t('name')),
                validator: (v) => v == null || v.trim().isEmpty
                    ? context.t('name_required')
                    : null,
              ),
              const SizedBox(height: 16),
              ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(context.t('ingredient')),
                subtitle: Text(
                  ingredientId == null
                      ? context.t('free_text')
                      : ingredientName ?? name.text,
                ),
                trailing: ingredientId != null
                    ? IconButton(
                        tooltip: context.t('delete'),
                        onPressed: () => setState(() {
                          ingredientId = null;
                          ingredientName = null;
                        }),
                        icon: const Icon(Icons.close),
                      )
                    : const Icon(Icons.search),
                onTap: !widget.controller.online
                    ? null
                    : () async {
                        final item = await chooseIngredient(
                          context,
                          widget.controller,
                        );
                        if (item != null && mounted) {
                          final tr =
                              (item['translations']
                                      as Json?)?[widget.controller.language]
                                  as Json?;
                          setState(() {
                            ingredientId = item['id'] as String;
                            ingredientName =
                                tr?['name'] as String? ??
                                item['name'] as String;
                            name.text = ingredientName!;
                          });
                        }
                      },
              ),
              if (!widget.controller.online)
                Text(context.t('ingredients_online')),
              const SizedBox(height: 24),
              TextFormField(
                controller: quantity,
                keyboardType: const TextInputType.numberWithOptions(
                  decimal: true,
                ),
                decoration: InputDecoration(
                  labelText: context.t('quantity'),
                  helperText: context.t('exact_hint'),
                  helperMaxLines: 3,
                ),
                validator: (v) {
                  try {
                    Quantity.parse(v ?? '', unit, widget.controller.language);
                    return null;
                  } catch (_) {
                    return context.t('quantity_invalid');
                  }
                },
              ),
              const SizedBox(height: 20),
              DropdownButtonFormField<String>(
                initialValue: unit,
                decoration: InputDecoration(labelText: context.t('unit')),
                items: ['g', 'kg', 'ml', 'l', 'piece']
                    .map(
                      (u) => DropdownMenuItem(
                        value: u,
                        child: Text(u == 'piece' ? context.t('piece') : u),
                      ),
                    )
                    .toList(),
                onChanged: saving ? null : (v) => setState(() => unit = v!),
              ),
              const SizedBox(height: 28),
              FilledButton.icon(
                onPressed: saving ? null : save,
                icon: saving
                    ? const SizedBox.square(
                        dimension: 20,
                        child: CircularProgressIndicator(strokeWidth: 2),
                      )
                    : const Icon(Icons.add),
                label: Text(context.t('save')),
              ),
            ],
          ),
        ),
      ),
    ),
  );
}
