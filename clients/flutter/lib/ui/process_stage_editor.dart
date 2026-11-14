import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import 'content_common.dart';
import 'messages.dart';
import 'process_common.dart';
import 'recipe_editor.dart' show chooseReferences;

class ProcessStageEditor extends StatefulWidget {
  const ProcessStageEditor(
    this.stage, {
    super.key,
    required this.stages,
    required this.changed,
    this.remove,
    this.maximum = 604800,
    this.knownTiming = true,
    this.timezone = 'UTC',
    this.allowAnchor = true,
    this.allowEquipment = true,
    this.completionWait = true,
  });
  final Json stage;
  final List<Json> stages;
  final VoidCallback changed;
  final VoidCallback? remove;
  final int maximum;
  final bool knownTiming;
  final String timezone;
  final bool allowAnchor;
  final bool allowEquipment, completionWait;
  @override
  State<ProcessStageEditor> createState() => _ProcessStageEditorState();
}

class _ProcessStageEditorState extends State<ProcessStageEditor> {
  late final visible = <String>{
    if ((widget.stage['equipment'] as List).isNotEmpty) 'equipment',
    if (widget.stage['repeat'] != 1) 'repeat',
    if (widget.stage['passive_min_seconds'] !=
            widget.stage['passive_seconds'] ||
        widget.stage['passive_max_seconds'] != widget.stage['passive_seconds'])
      'windows',
    if ((widget.stage['uncertainty'] as String).isNotEmpty) 'uncertainty',
    if (widget.stage['start_at'] != null) 'anchor',
  };
  void update(String key, Object? value) {
    widget.stage[key] = value;
    widget.changed();
  }

  @override
  Widget build(BuildContext context) {
    final stage = widget.stage;
    return Padding(
      padding: const EdgeInsets.symmetric(vertical: 16),
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          EditText(
            'cookStageName',
            stage['name'] as String,
            (v) => update('name', v),
            required: true,
          ),
          EditText(
            'step_instruction',
            stage['instruction'] as String,
            (v) => update('instruction', v),
            required: true,
            lines: 3,
            max: 4000,
          ),
          if (object(stage['origin'])['kind'] == 'custom')
            EditText(
              'cookSourceNote',
              object(stage['origin'])['note'] as String,
              (v) {
                object(stage['origin'])['note'] = v;
                widget.changed();
              },
              required: true,
              max: 500,
            ),
          if (widget.knownTiming) ...[
            DurationField(
              'cookActiveDuration',
              stage['active_seconds'] as int,
              (v) => update('active_seconds', v),
              maximum: widget.maximum,
            ),
            DurationField(
              'cookPassiveDuration',
              stage['passive_seconds'] as int,
              (v) {
                final old = stage['passive_seconds'];
                stage['passive_seconds'] = v;
                if (!visible.contains('windows')) {
                  stage['passive_min_seconds'] = v;
                  stage['passive_max_seconds'] = v;
                } else {
                  if (stage['passive_min_seconds'] == old) {
                    stage['passive_min_seconds'] = v;
                  }
                  if (stage['passive_max_seconds'] == old) {
                    stage['passive_max_seconds'] = v;
                  }
                }
                widget.changed();
              },
              maximum: widget.maximum,
            ),
            if (widget.completionWait)
              CheckboxListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(context.t('cook_work_finished')),
                value: stage['start_after_work'] == true,
                onChanged: (v) {
                  setState(() => stage['start_after_work'] = v == true);
                  widget.changed();
                },
              ),
          ],
          OutlinedButton.icon(
            icon: const Icon(Icons.account_tree_outlined),
            label: Text(
              '${context.t('cookDependencies')} · ${(stage['dependencies'] as List).length}',
            ),
            onPressed: () async {
              final picked = await chooseReferences(
                context,
                'cookDependencies',
                {
                  for (final other in widget.stages.where(
                    (s) => s['id'] != stage['id'],
                  ))
                    other['id'] as String: other['name'] as String,
                },
                (stage['dependencies'] as List).cast<String>(),
              );
              if (picked != null && mounted) {
                setState(() => stage['dependencies'] = picked);
                widget.changed();
              }
            },
          ),
          if (widget.allowEquipment && visible.contains('equipment'))
            EditText(
              'cookEquipment',
              (stage['equipment'] as List).join('\n'),
              (v) => update(
                'equipment',
                v
                    .split('\n')
                    .map((v) => v.trim())
                    .where((v) => v.isNotEmpty)
                    .toList(),
              ),
              lines: 3,
              max: 2400,
              hint: 'cookEquipmentHelp',
            ),
          if (widget.knownTiming && visible.contains('windows')) ...[
            DurationField(
              'cookPassiveMin',
              stage['passive_min_seconds'] as int,
              (v) => update('passive_min_seconds', v),
              maximum: widget.maximum,
            ),
            DurationField(
              'cookPassiveMax',
              stage['passive_max_seconds'] as int,
              (v) => update('passive_max_seconds', v),
              maximum: widget.maximum,
            ),
          ],
          if (visible.contains('repeat')) ...[
            EditText(
              'cookRepeat',
              '${stage['repeat']}',
              (v) {
                final n = int.tryParse(v);
                if (n != null) update('repeat', n);
              },
              required: true,
              numeric: true,
              integer: true,
              max: 2,
            ),
            DurationField(
              'cookRepeatWait',
              stage['wait_between_seconds'] as int,
              (v) => update('wait_between_seconds', v),
            ),
          ],
          if (visible.contains('uncertainty'))
            EditText(
              'cookUncertainty',
              stage['uncertainty'] as String,
              (v) => update('uncertainty', v),
              lines: 3,
              max: 1000,
              hint: 'cookUncertaintyHelp',
            ),
          if (visible.contains('anchor'))
            LocalMomentEditor('cookStartAfter', object(stage['start_at']), (v) {
              setState(() => stage['start_at'] = v);
              widget.changed();
            }),
          DurationField(
            'cookReminderLead',
            stage['lead_seconds'] as int,
            (v) => update('lead_seconds', v),
          ),
          Wrap(
            spacing: 8,
            runSpacing: 8,
            children: [
              for (final option in <String, String>{
                if (widget.allowEquipment) 'equipment': 'cookEquipment',
                if (widget.knownTiming) 'windows': 'cookPassiveMin',
                'repeat': 'cookRepeat',
                'uncertainty': 'cookUncertainty',
                if (widget.allowAnchor) 'anchor': 'cook_calendar_anchor',
              }.entries.where((e) => !visible.contains(e.key)))
                TextButton.icon(
                  icon: const Icon(Icons.add),
                  label: Text(context.t(option.value)),
                  onPressed: () => setState(() {
                    visible.add(option.key);
                    if (option.key == 'anchor') {
                      stage['start_at'] = localMoment(widget.timezone);
                    }
                    widget.changed();
                  }),
                ),
              if (visible.contains('anchor'))
                TextButton(
                  onPressed: () => setState(() {
                    stage['start_at'] = null;
                    visible.remove('anchor');
                    widget.changed();
                  }),
                  child: Text(context.t('remove')),
                ),
              if (widget.remove != null)
                IconButton(
                  tooltip: context.t('cookRemoveStage'),
                  onPressed: widget.remove,
                  icon: const Icon(Icons.delete_outline),
                ),
            ],
          ),
          const Divider(),
        ],
      ),
    );
  }
}

Json newProcessStage(String id) => {
  'id': id,
  'origin': {'kind': 'custom', 'note': ''},
  'name': '',
  'instruction': '',
  'dependencies': <String>[],
  'active_seconds': 0,
  'passive_seconds': 0,
  'passive_min_seconds': 0,
  'passive_max_seconds': 0,
  'equipment': <String>[],
  'lead_seconds': 300,
  'repeat': 1,
  'wait_between_seconds': 0,
  'uncertainty': '',
  'start_after_work': true,
};
