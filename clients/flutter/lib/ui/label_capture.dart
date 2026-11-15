import 'dart:convert';

import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'common.dart';
import 'content_common.dart';
import 'capture_photo.dart';
import 'messages.dart';

class LabelCapture extends StatefulWidget {
  const LabelCapture(this.api, {super.key, this.value});
  final HouseholdApi api;
  final Json? value;
  @override
  State<LabelCapture> createState() => _LabelCaptureState();
}

class _LabelCaptureState extends State<LabelCapture> {
  final form = GlobalKey<FormState>();
  late Json label;
  bool confirmed = false;
  @override
  void initState() {
    super.initState();
    label = widget.value == null
        ? {'basis': '', 'values': <Json>[]}
        : clone(widget.value!);
    localize();
  }

  void localize() {
    for (final row in objects(label['values'])) {
      row['value'] = inputDecimal(
        row['value'] as String,
        widget.api.controller.language,
      );
    }
  }

  void change(VoidCallback fn) => setState(() {
    fn();
    confirmed = false;
  });
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('capture_label'))),
    body: !widget.api.writable
        ? ListView(
            padding: const EdgeInsets.all(20),
            children: [
              Note(context.t('capture_label_help')),
              if (label['photo'] != null) ...[
                PrivateImage(
                  widget.api,
                  '${widget.api.domain}/household/images/${object(label['photo'])['image_id']}',
                  original: true,
                ),
                SelectableText(object(label['photo'])['text'] as String),
                SectionTitle(context.t('capture_basis')),
                SelectableText(label['basis'] as String),
                for (final value in objects(label['values']))
                  ListTile(
                    contentPadding: EdgeInsets.zero,
                    title: Text(
                      '${value['name']}: ${context.t('capture_${value['qualifier'] ?? 'exact'}')} ${value['value']} ${value['unit']}',
                    ),
                    subtitle: Text(
                      value['definition'] as String? ?? context.t('unknown'),
                    ),
                  ),
              ],
            ],
          )
        : Form(
            key: form,
            child: ListView(
              padding: const EdgeInsets.all(20),
              children: [
                Note(context.t('capture_label_help')),
                if (label['photo'] == null)
                  FilledButton.icon(
                    icon: const Icon(Icons.document_scanner_outlined),
                    label: Text(context.t('capture_photo')),
                    onPressed: () async {
                      final photo = await openPage<Json>(
                        context,
                        CapturePhoto(widget.api),
                      );
                      if (photo == null || !context.mounted) return;
                      if (utf8.encode(photo['text'] as String).length > 4096) {
                        await showFailure(
                          context,
                          const ApiFailure('capture_label_limit'),
                        );
                        return;
                      }
                      change(() => label['photo'] = photo);
                    },
                  ),
                if (label['photo'] != null) ...[
                  SectionTitle(context.t('capture_original')),
                  PrivateImage(
                    widget.api,
                    '${widget.api.domain}/household/images/${object(label['photo'])['image_id']}',
                    original: true,
                  ),
                  SelectableText(object(label['photo'])['text'] as String),
                  EditText(
                    'capture_basis',
                    label['basis'] as String,
                    (v) => change(() => label['basis'] = v),
                    required: true,
                  ),
                  for (final row in objects(label['values']))
                    Column(
                      key: ObjectKey(row),
                      children: [
                        Row(
                          children: [
                            Expanded(
                              child: EditText(
                                'name',
                                row['name'] as String,
                                (v) => change(() => row['name'] = v),
                                required: true,
                              ),
                            ),
                            IconButton(
                              tooltip: context.t('delete'),
                              icon: const Icon(Icons.delete_outline),
                              onPressed: () => change(
                                () => (label['values'] as List).remove(row),
                              ),
                            ),
                          ],
                        ),
                        Row(
                          children: [
                            Expanded(
                              child: EditText(
                                'quantity',
                                row['value'] as String,
                                (v) => change(() => row['value'] = v),
                                required: true,
                                numeric: true,
                              ),
                            ),
                            const SizedBox(width: 12),
                            Expanded(
                              child: Choice('unit', row['unit'] as String, {
                                for (final u in [
                                  'g',
                                  'mg',
                                  'µg',
                                  'kJ',
                                  'kcal',
                                  '%',
                                ])
                                  u: u,
                              }, (v) => change(() => row['unit'] = v)),
                            ),
                          ],
                        ),
                        Choice(
                          'capture_qualifier',
                          row['qualifier'] as String? ?? 'exact',
                          {
                            'exact': context.t('capture_exact'),
                            'less_than': context.t('capture_less_than'),
                            'approximate': context.t('capture_approximate'),
                          },
                          (v) => change(() => row['qualifier'] = v),
                        ),
                        EditText(
                          'capture_definition',
                          row['definition'] as String? ?? '',
                          (v) => change(
                            () =>
                                row['definition'] = v.trim().isEmpty ? null : v,
                          ),
                        ),
                        const Divider(),
                      ],
                    ),
                  OutlinedButton.icon(
                    icon: const Icon(Icons.add),
                    label: Text(context.t('capture_value')),
                    onPressed: objects(label['values']).length >= 20
                        ? null
                        : () => change(
                            () => (label['values'] as List).add({
                              'name': '',
                              'value': '',
                              'unit': 'g',
                              'definition': null,
                              'qualifier': 'exact',
                            }),
                          ),
                  ),
                  CheckboxListTile(
                    contentPadding: EdgeInsets.zero,
                    value: confirmed,
                    title: Text(context.t('capture_label_confirm')),
                    onChanged: (v) => setState(() => confirmed = v == true),
                  ),
                  FilledButton(
                    onPressed: !confirmed || !widget.api.writable
                        ? null
                        : () async {
                            if (!form.currentState!.validate()) return;
                            try {
                              final result = clone(label);
                              result['reviewed'] = true;
                              for (final row in objects(result['values'])) {
                                row['value'] = exactInput(
                                  row['value'] as String,
                                  widget.api.controller.language,
                                );
                              }
                              Navigator.pop(context, result);
                            } catch (e) {
                              await showFailure(context, e);
                            }
                          },
                    child: Text(context.t('capture_use_label')),
                  ),
                ],
              ],
            ),
          ),
  );
}
