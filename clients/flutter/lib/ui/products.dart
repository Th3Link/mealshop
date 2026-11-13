import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../shopping/controller.dart';
import '../shopping/quantity.dart';
import 'add_item.dart';
import 'common.dart';
import 'messages.dart';

class ProductsScreen extends StatelessWidget {
  const ProductsScreen(this.c, {super.key});
  final ShoppingController c;
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: c,
    builder: (context, _) {
      final observedRevision = c.shopping?.revision;
      final can = c.writable && c.online && !c.busy && c.pending.isEmpty;
      return Scaffold(
        appBar: AppBar(title: Text(context.t('products'))),
        body: ListView(
          children: [
            if (c.writable)
              Padding(
                padding: const EdgeInsets.all(20),
                child: FilledButton.icon(
                  onPressed: can
                      ? () => Navigator.push<void>(
                          context,
                          MaterialPageRoute(builder: (_) => ProductEditor(c)),
                        )
                      : null,
                  icon: const Icon(Icons.add),
                  label: Text(context.t('new_product')),
                ),
              ),
            for (final product
                in c.shopping?.products.values.map(object) ?? <Json>[])
              ListTile(
                leading: const Icon(Icons.inventory_2_outlined),
                title: Text(product['name'] as String),
                subtitle: Text(
                  '${product['barcode']} · ${Quantity.display(object(product['quantity'])['amount'] as String, c.language)} ${object(product['quantity'])['unit']}',
                ),
                onTap: () => Navigator.push<void>(
                  context,
                  MaterialPageRoute(
                    builder: (_) => ProductEditor(c, product: product),
                  ),
                ),
                trailing: can
                    ? IconButton(
                        tooltip: context.t('add'),
                        icon: const Icon(Icons.add_shopping_cart),
                        onPressed: () async {
                          final text = await askName(
                            context,
                            'packages',
                            initial: '1',
                            field: 'packages',
                            integer: true,
                          );
                          final count = int.tryParse(text ?? '');
                          if (count == null || count < 1 || count > 1000000) {
                            return;
                          }
                          try {
                            await c.enqueue({
                              'operation': 'add_product',
                              'barcode': product['barcode'],
                              'packages': count,
                            }, expectedRevision: observedRevision);
                          } catch (e) {
                            if (context.mounted) await showFailure(context, e);
                          }
                        },
                      )
                    : null,
              ),
            if (c.shopping?.products.isEmpty ?? true)
              Padding(
                padding: const EdgeInsets.all(24),
                child: Text(context.t('no_results')),
              ),
          ],
        ),
      );
    },
  );
}

class ProductEditor extends StatefulWidget {
  const ProductEditor(this.c, {super.key, this.product});
  final ShoppingController c;
  final Json? product;
  @override
  State<ProductEditor> createState() => _ProductEditorState();
}

class _ProductEditorState extends State<ProductEditor> {
  final form = GlobalKey<FormState>();
  final barcode = TextEditingController(),
      name = TextEditingController(),
      brand = TextEditingController(),
      label = TextEditingController(),
      quantity = TextEditingController();
  String unit = 'g';
  String? ingredientId, ingredientName, notice;
  Json? candidate;
  bool confirmed = false, busy = false, dirty = false;
  int? openedRevision;
  @override
  void initState() {
    super.initState();
    openedRevision = widget.c.shopping?.revision;
    final p = widget.product;
    if (p != null) {
      barcode.text = p['barcode'] as String;
      name.text = p['name'] as String;
      brand.text = p['brand'] as String;
      label.text = p['label_quantity'] as String;
      quantity.text = (object(p['quantity'])['amount'] as String).replaceAll(
        '.',
        widget.c.language == 'de' ? ',' : '.',
      );
      unit = object(p['quantity'])['unit'] as String;
      ingredientId = p['ingredient_id'] as String?;
      confirmed = ingredientId != null;
    }
  }

  @override
  void dispose() {
    for (final c in [barcode, name, brand, label, quantity]) {
      c.dispose();
    }
    super.dispose();
  }

  Future<void> lookup() async {
    if (!await confirm(context, 'lookup', body: 'lookup_consent')) return;
    setState(() => busy = true);
    try {
      final result = await widget.c.lookup(barcode.text.trim());
      if (mounted) {
        setState(() {
          candidate = result;
          openedRevision = widget.c.shopping?.revision;
          if (result == null) {
            notice = 'lookup_empty';
          } else {
            final e = object(result['evidence']);
            name.text = e['original_name'] as String;
            brand.text = e['original_brand'] as String;
            label.text = e['original_quantity'] as String;
            notice = null;
          }
        });
      }
    } catch (e) {
      if (mounted) await showFailure(context, e);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  Future<void> save() async {
    if (!form.currentState!.validate()) return;
    setState(() => busy = true);
    try {
      await widget.c.enqueue({
        'operation': 'save_product',
        'product': {
          'barcode': barcode.text.trim(),
          'name': name.text.trim(),
          'brand': brand.text.trim(),
          'label_quantity': label.text.trim(),
          'quantity': Quantity.parse(
            quantity.text,
            unit,
            widget.c.language,
          ).toJson(),
          'ingredient_id': ingredientId,
          'match_confirmed': confirmed,
          if (candidate != null) 'candidate_id': candidate!['id'],
        },
      }, expectedRevision: openedRevision);
      if (mounted) Navigator.pop(context);
    } catch (e) {
      if (mounted) await showFailure(context, e);
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final can = widget.c.writable && widget.c.online && !widget.c.busy && !busy;
    return Scaffold(
      appBar: AppBar(
        title: Text(
          widget.product?['name'] as String? ?? context.t('new_product'),
        ),
        actions: [
          if (widget.product != null && can)
            IconButton(
              tooltip: context.t('delete'),
              onPressed: () async {
                if (await confirm(context, 'confirm_delete')) {
                  try {
                    await widget.c.enqueue({
                      'operation': 'remove_product',
                      'barcode': barcode.text,
                    }, expectedRevision: openedRevision);
                    if (context.mounted) Navigator.pop(context);
                  } catch (e) {
                    if (context.mounted) await showFailure(context, e);
                  }
                }
              },
              icon: const Icon(Icons.delete_outline),
            ),
        ],
      ),
      body: DraftGuard(
        dirty: () => dirty,
        child: Form(
          onChanged: () => setState(() => dirty = true),
          key: form,
          child: ListView(
            padding: const EdgeInsets.all(20),
            children: [
              TextFormField(
                controller: barcode,
                readOnly: widget.product != null || !widget.c.writable,
                maxLength: 14,
                keyboardType: TextInputType.number,
                decoration: InputDecoration(labelText: context.t('barcode')),
                onChanged: (_) => candidate = null,
                validator: (v) => validGtin(v ?? '')
                    ? null
                    : context.t('shopping_barcode_invalid'),
              ),
              if (widget.product == null)
                TextButton.icon(
                  onPressed: can ? lookup : null,
                  icon: const Icon(Icons.search),
                  label: Text(context.t('lookup')),
                ),
              if (notice != null) Text(context.t(notice!)),
              const SizedBox(height: 16),
              TextFormField(
                readOnly: !widget.c.writable,
                controller: name,
                maxLength: 200,
                decoration: InputDecoration(labelText: context.t('name')),
                validator: (v) => v == null || v.trim().isEmpty
                    ? context.t('name_required')
                    : null,
              ),
              const SizedBox(height: 16),
              TextFormField(
                readOnly: !widget.c.writable,
                controller: brand,
                maxLength: 200,
                decoration: InputDecoration(labelText: context.t('brand')),
              ),
              const SizedBox(height: 16),
              TextFormField(
                readOnly: !widget.c.writable,
                controller: label,
                maxLength: 200,
                decoration: InputDecoration(
                  labelText: context.t('label_quantity'),
                ),
              ),
              const SizedBox(height: 16),
              TextFormField(
                readOnly: !widget.c.writable,
                controller: quantity,
                keyboardType: const TextInputType.numberWithOptions(
                  decimal: true,
                ),
                decoration: InputDecoration(
                  labelText: context.t('package_quantity'),
                  helperText: context.t('exact_hint'),
                  helperMaxLines: 3,
                ),
                validator: (v) {
                  try {
                    Quantity.parse(v ?? '', unit, widget.c.language);
                    return null;
                  } catch (_) {
                    return context.t('quantity_invalid');
                  }
                },
              ),
              const SizedBox(height: 16),
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
                onChanged: can ? (v) => setState(() => unit = v!) : null,
              ),
              const SizedBox(height: 16),
              ListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(context.t('ingredient')),
                subtitle: Text(
                  ingredientName ?? ingredientId ?? context.t('free_text'),
                ),
                trailing: ingredientId == null
                    ? const Icon(Icons.search)
                    : IconButton(
                        tooltip: context.t('delete'),
                        onPressed: can
                            ? () => setState(() {
                                ingredientId = null;
                                ingredientName = null;
                                confirmed = false;
                              })
                            : null,
                        icon: const Icon(Icons.close),
                      ),
                onTap: can
                    ? () async {
                        final i = await chooseIngredient(context, widget.c);
                        if (i != null && mounted) {
                          setState(() {
                            ingredientId = i['id'] as String;
                            ingredientName = i['name'] as String;
                            confirmed = false;
                          });
                        }
                      }
                    : null,
              ),
              if (ingredientId != null)
                CheckboxListTile(
                  contentPadding: EdgeInsets.zero,
                  title: Text(context.t('match_confirmed')),
                  value: confirmed,
                  onChanged: can
                      ? (v) => setState(() => confirmed = v ?? false)
                      : null,
                ),
              if (candidate != null || widget.product?['evidence'] != null)
                Builder(
                  builder: (context) {
                    final e = object(
                      candidate?['evidence'] ?? widget.product!['evidence'],
                    );
                    return Padding(
                      padding: const EdgeInsets.symmetric(vertical: 16),
                      child: SelectableText(
                        '${context.t('source')}: ${e['source_url']}\n${context.t('licence')}: ${e['licence']}',
                      ),
                    );
                  },
                ),
              const SizedBox(height: 24),
              FilledButton(
                onPressed: can ? save : null,
                child: Text(context.t('save')),
              ),
            ],
          ),
        ),
      ),
    );
  }
}

bool validGtin(String value) {
  if (!RegExp(r'^(?:\d{8}|\d{12,14})$').hasMatch(value) ||
      RegExp(r'^0+$').hasMatch(value)) {
    return false;
  }
  var sum = 0;
  final digits = value.split('').reversed.toList();
  for (var i = 0; i < digits.length; i++) {
    sum += int.parse(digits[i]) * (i.isEven ? 1 : 3);
  }
  return sum % 10 == 0;
}
