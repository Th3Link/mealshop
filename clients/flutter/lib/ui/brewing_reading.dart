import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/controller.dart';
import 'common.dart';
import 'content_common.dart';
import 'brewing_common.dart';
import 'brewing_review.dart';
import 'messages.dart';
import 'process_common.dart';

class BrewingReadingEditor extends StatefulWidget {
  const BrewingReadingEditor(this.c, this.run, {super.key, this.original});
  final CookingController c;
  final Json run;
  final Json? original;
  @override
  State<BrewingReadingEditor> createState() => _BrewingReadingEditorState();
}

class _BrewingReadingEditorState extends State<BrewingReadingEditor> {
  final form = GlobalKey<FormState>();
  late Json reading = widget.original == null
      ? {
          'phase_id': null,
          'measurement': {'value': '', 'unit': 'celsius'},
          'measured_at': localMoment(widget.c.clock.sample?.timezone ?? 'UTC'),
          'source': {'kind': 'manual'},
          'calibration': {'kind': 'unknown'},
          'notes': '',
        }
      : clone(object(widget.original!['input']));
  String reason = '';
  bool dirty = false, saved = false, busy = false;
  String? error;
  void changed() {
    dirty = true;
  }

  @override
  void initState() {
    super.initState();
    if (widget.original != null &&
        object(reading['source'])['kind'] == 'instrument') {
      object(reading['source'])['acquisition'] = 'transcribed';
    }
  }

  @override
  Widget build(BuildContext context) => DraftGuard(
    dirty: () => dirty && !saved,
    child: Scaffold(
      appBar: AppBar(
        title: Text(
          context.t(
            widget.original == null ? 'brewAddReading' : 'brewCorrectReading',
          ),
        ),
      ),
      body: Form(
        key: form,
        child: ListView(
          padding: const EdgeInsets.all(16),
          children: [
            if (widget.original != null) ...[
              SectionTitle(context.t('brew_before')),
              BrewReadingSummary(object(widget.original!['input'])),
              const Divider(),
            ],
            BrewMeasurementField(
              'brewReadingValue',
              object(reading['measurement']),
              changed,
            ),
            Note(context.t('brewOriginalUnits')),
            LocalMomentEditor(
              'brewMeasuredAt',
              object(reading['measured_at']),
              (v) => setState(() {
                reading['measured_at'] = v;
                changed();
              }),
            ),
            Choice(
              'brewPhase',
              reading['phase_id'] as String? ?? '',
              {
                '': context.t('brewNone'),
                for (final step in objects(widget.run['steps']))
                  object(step['stage'])['id'] as String:
                      object(step['stage'])['name'] as String,
              },
              (v) => setState(() {
                reading['phase_id'] = v.isEmpty ? null : v;
                changed();
              }),
            ),
            Choice(
              'brewReadingSource',
              object(reading['source'])['kind'] as String,
              {
                'manual': context.t('brewManual'),
                'instrument': context.t('brewInstrument'),
              },
              (v) => setState(() {
                reading['source'] = v == 'manual'
                    ? {'kind': 'manual'}
                    : {
                        'kind': 'instrument',
                        'device_id': '',
                        'name': '',
                        'acquisition': 'transcribed',
                      };
                changed();
              }),
            ),
            if (object(reading['source'])['kind'] == 'instrument') ...[
              Note(context.t('brewDeviceHelp')),
              EditText(
                'brewDeviceName',
                object(reading['source'])['name'] as String,
                (v) {
                  object(reading['source'])['name'] = v;
                  changed();
                },
                required: true,
              ),
              EditText(
                'brewDeviceId',
                object(reading['source'])['device_id'] as String,
                (v) {
                  object(reading['source'])['device_id'] = v;
                  changed();
                },
                required: true,
                max: 120,
              ),
            ],
            Choice(
              'brewCalibration',
              object(reading['calibration'])['kind'] as String,
              {
                'unknown': context.t('brewCalibrationUnknown'),
                'declared': context.t('brewCalibrationDeclared'),
              },
              (v) => setState(() {
                reading['calibration'] = v == 'unknown'
                    ? {'kind': 'unknown'}
                    : {
                        'kind': 'declared',
                        'details': '',
                        'calibrated_at': null,
                        'reference_temperature': null,
                        'source_url': null,
                      };
                changed();
              }),
            ),
            if (object(reading['calibration'])['kind'] == 'declared') ...[
              EditText(
                'brewCalibrationDetails',
                object(reading['calibration'])['details'] as String,
                (v) {
                  object(reading['calibration'])['details'] = v;
                  changed();
                },
                required: true,
                lines: 3,
                max: 2000,
              ),
              if (object(reading['calibration'])['calibrated_at'] == null)
                TextButton.icon(
                  icon: const Icon(Icons.add),
                  label: Text(context.t('brewCalibrationAt')),
                  onPressed: () => setState(() {
                    object(reading['calibration'])['calibrated_at'] = clone(
                      object(reading['measured_at']),
                    );
                    changed();
                  }),
                )
              else ...[
                LocalMomentEditor(
                  'brewCalibrationAt',
                  object(object(reading['calibration'])['calibrated_at']),
                  (v) => setState(() {
                    object(reading['calibration'])['calibrated_at'] = v;
                    changed();
                  }),
                ),
                TextButton(
                  onPressed: () => setState(() {
                    object(reading['calibration'])['calibrated_at'] = null;
                    changed();
                  }),
                  child: Text(context.t('remove')),
                ),
              ],
              if (object(reading['calibration'])['reference_temperature'] ==
                  null)
                TextButton.icon(
                  icon: const Icon(Icons.add),
                  label: Text(context.t('brewReferenceTemperature')),
                  onPressed: () => setState(() {
                    object(reading['calibration'])['reference_temperature'] = {
                      'value': '',
                      'unit': 'celsius',
                    };
                    changed();
                  }),
                )
              else ...[
                BrewMeasurementField(
                  'brewReferenceTemperature',
                  object(
                    object(reading['calibration'])['reference_temperature'],
                  ),
                  changed,
                  temperature: true,
                ),
                TextButton(
                  onPressed: () => setState(() {
                    object(reading['calibration'])['reference_temperature'] =
                        null;
                    changed();
                  }),
                  child: Text(context.t('remove')),
                ),
              ],
              EditText(
                'source_url',
                object(reading['calibration'])['source_url'] as String? ?? '',
                (v) {
                  object(reading['calibration'])['source_url'] =
                      v.trim().isEmpty ? null : v.trim();
                  changed();
                },
                max: 2000,
              ),
            ],
            EditText(
              'brewNotes',
              reading['notes'] as String,
              (v) {
                reading['notes'] = v;
                changed();
              },
              lines: 3,
              max: 2000,
            ),
            if (widget.original != null)
              EditText(
                'brewCorrectionReason',
                reason,
                (v) {
                  reason = v;
                  changed();
                },
                required: true,
                max: 1000,
                lines: 2,
              ),
            BrewingPending(widget.c),
            if (error != null) Note(context.t(error!), warning: true),
            FilledButton(
              onPressed:
                  busy || !widget.c.writable || widget.c.pending.isNotEmpty
                  ? null
                  : () async {
                      if (!form.currentState!.validate()) return;
                      setState(() => busy = true);
                      try {
                        final result = await reviewBrewing(context, widget.c, {
                          'operation': 'add_reading',
                          'run_id': widget.run['id'],
                          'reading': clone(reading),
                          'corrects': widget.original?['id'],
                          'reason': widget.original == null ? null : reason,
                        });
                        if (result != null && context.mounted) {
                          saved = true;
                          Navigator.pop(context, result['result_id']);
                        }
                      } catch (e) {
                        if (mounted) {
                          setState(
                            () => error = e is ApiFailure
                                ? e.code
                                : 'request_failed',
                          );
                        }
                      } finally {
                        if (mounted) setState(() => busy = false);
                      }
                    },
              child: Text(context.t('brewReview')),
            ),
            if (busy) const LinearProgressIndicator(),
          ],
        ),
      ),
    ),
  );
}
