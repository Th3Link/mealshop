import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../brewing/data.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';
import 'process_common.dart';
import 'ingredients.dart' show openSource;

String brewQuantityText(BuildContext context, Object? value) => value == null
    ? context.t('unknown')
    : '${exactNumber(context, object(value)['amount'] as String)} ${object(value)['unit']}';
Map<String, String> brewOptions(
  BuildContext context,
  Map<String, String> options,
) => options.map((id, key) => MapEntry(id, context.t(key)));

class BrewQuantityField extends StatefulWidget {
  const BrewQuantityField(
    this.label,
    this.value,
    this.changed, {
    super.key,
    this.volume = false,
  });
  final String label;
  final Json value;
  final VoidCallback changed;
  final bool volume;
  @override
  State<BrewQuantityField> createState() => _BrewQuantityFieldState();
}

class _BrewQuantityFieldState extends State<BrewQuantityField> {
  TextEditingController? text;
  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    text ??= TextEditingController(
      text: inputDecimal(
        widget.value['amount'] as String,
        Localizations.localeOf(context).languageCode,
      ),
    );
  }

  @override
  void dispose() {
    text?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 8),
    child: Column(
      children: [
        TextFormField(
          key: ValueKey(widget.label),
          controller: text,
          keyboardType: const TextInputType.numberWithOptions(decimal: true),
          decoration: InputDecoration(labelText: context.t(widget.label)),
          validator: (v) {
            try {
              final n = exactInput(
                v ?? '',
                Localizations.localeOf(context).languageCode,
              );
              return RegExp(r'^0+(?:\.0+)?$').hasMatch(n)
                  ? context.t('quantity_invalid')
                  : null;
            } on ApiFailure {
              return context.t('quantity_invalid');
            }
          },
          onChanged: (v) {
            try {
              widget.value['amount'] = exactInput(
                v,
                Localizations.localeOf(context).languageCode,
              );
            } on ApiFailure {
              /* The form retains invalid text and blocks submission. */
            }
            widget.changed();
          },
        ),
        Choice(
          'unit',
          widget.value['unit'] as String,
          {
            for (final unit
                in widget.volume
                    ? ['ml', 'l']
                    : ['g', 'kg', 'ml', 'l', 'piece'])
              unit: unit == 'piece' ? context.t('piece') : unit,
          },
          (v) {
            setState(() => widget.value['unit'] = v);
            widget.changed();
          },
        ),
      ],
    ),
  );
}

class BrewMeasurementField extends StatefulWidget {
  const BrewMeasurementField(
    this.label,
    this.value,
    this.changed, {
    super.key,
    this.temperature = false,
  });
  final String label;
  final Json value;
  final VoidCallback changed;
  final bool temperature;
  @override
  State<BrewMeasurementField> createState() => _BrewMeasurementFieldState();
}

class _BrewMeasurementFieldState extends State<BrewMeasurementField> {
  TextEditingController? text;
  bool get signed => ['celsius', 'fahrenheit'].contains(widget.value['unit']);
  @override
  void didChangeDependencies() {
    super.didChangeDependencies();
    text ??= TextEditingController(
      text: inputDecimal(
        widget.value['value'] as String,
        Localizations.localeOf(context).languageCode,
      ),
    );
  }

  @override
  void dispose() {
    text?.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 8),
    child: Column(
      children: [
        TextFormField(
          key: ValueKey(widget.label),
          controller: text,
          keyboardType: TextInputType.numberWithOptions(
            decimal: true,
            signed: signed,
          ),
          decoration: InputDecoration(labelText: context.t(widget.label)),
          validator: (v) {
            try {
              exactInput(
                v ?? '',
                Localizations.localeOf(context).languageCode,
                signed: signed,
              );
              return null;
            } on ApiFailure {
              return context.t('quantity_invalid');
            }
          },
          onChanged: (v) {
            try {
              widget.value['value'] = exactInput(
                v,
                Localizations.localeOf(context).languageCode,
                signed: signed,
              );
            } on ApiFailure {
              /* Keep the user's incomplete original input. */
            }
            widget.changed();
          },
        ),
        Choice(
          'unit',
          widget.value['unit'] as String,
          brewOptions(
            context,
            widget.temperature
                ? {'celsius': 'brewCelsius', 'fahrenheit': 'brewFahrenheit'}
                : brewReadingUnits,
          ),
          (v) {
            setState(() => widget.value['unit'] = v);
            widget.changed();
          },
        ),
      ],
    ),
  );
}

class BrewRecipeSummary extends StatelessWidget {
  const BrewRecipeSummary(this.recipe, {super.key});
  final Json recipe;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        recipe['title'] as String,
        style: Theme.of(context).textTheme.titleLarge,
      ),
      Text(
        '${context.t(brewBeverages[recipe['beverage']]!)} · ${brewQuantityText(context, recipe['volume'])}',
      ),
      for (final method in object(recipe['method']).entries)
        Text(
          context.t(
            method.key == 'process'
                ? brewProcesses[method.value]!
                : brewMethods[method.key]![method.value]!,
          ),
        ),
      for (final line in objects(recipe['ingredients']))
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 4),
          child: Text(
            '${line['name']} · ${brewQuantityText(context, line['quantity'])}${line['note'] == '' ? '' : ' · ${line['note']}'}',
          ),
        ),
      for (final phase in objects(recipe['phases']))
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 12),
          child: Column(
            crossAxisAlignment: CrossAxisAlignment.start,
            children: [
              Text(
                object(phase['stage'])['name'] as String,
                style: Theme.of(context).textTheme.titleMedium,
              ),
              SelectableText(object(phase['stage'])['instruction'] as String),
              Text(
                phase['timing_known'] == true
                    ? '${context.t('cookActiveDuration')}: ${durationText(object(phase['stage'])['active_seconds'] as int)} · ${context.t('cookPassiveDuration')}: ${durationText(object(phase['stage'])['passive_seconds'] as int)}'
                    : context.t('brewUnknownTiming'),
              ),
              if (phase['temperature'] != null)
                Text(
                  '${exactNumber(context, object(phase['temperature'])['value'] as String)} ${context.t(brewReadingUnits[object(phase['temperature'])['unit']]!)}',
                ),
              if (phase['volume'] != null)
                Text(
                  '${context.t('brewPhaseVolume')}: ${brewQuantityText(context, phase['volume'])}',
                ),
              if (object(phase['stage'])['uncertainty'] != '')
                Note(object(phase['stage'])['uncertainty'] as String),
            ],
          ),
        ),
      if (recipe['yeast_preparation'] != null) ...[
        SectionTitle(context.t('brewYeast')),
        Text(
          '${object(recipe['yeast_preparation'])['days_before']} ${context.t('cookDays')} · ${object(recipe['yeast_preparation'])['local_time']}',
        ),
        Text(object(recipe['yeast_preparation'])['instruction'] as String),
      ],
      if (recipe['notes'] != '') Note(recipe['notes'] as String),
      SectionTitle(context.t('brewSource')),
      for (final source in objects(object(recipe['template'])['sources']))
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(source['title'] as String),
          subtitle: Text(
            '${source['publisher']} · ${source['document_version'] ?? source['retrieved_on']}',
          ),
          trailing: const Icon(Icons.open_in_new),
          onTap: () =>
              runAction(context, () => openSource(source['url'] as String)),
        ),
    ],
  );
}

class BrewReadingSummary extends StatelessWidget {
  const BrewReadingSummary(this.input, {super.key, this.reason, this.corrects});
  final Json input;
  final String? reason, corrects;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        '${exactNumber(context, object(input['measurement'])['value'] as String)} ${context.t(brewReadingUnits[object(input['measurement'])['unit']]!)}',
        style: Theme.of(context).textTheme.titleLarge,
      ),
      Text(
        '${object(input['measured_at'])['local']} · ${object(input['measured_at'])['timezone']} · ${context.t(object(input['measured_at'])['fold'] == 'earlier'
            ? 'cookFoldEarlier'
            : object(input['measured_at'])['fold'] == 'later'
            ? 'cookFoldLater'
            : 'cookFoldReject')}',
      ),
      Text(
        context.t(
          object(input['source'])['kind'] == 'manual'
              ? 'brewManual'
              : 'brewInstrumentReading',
        ),
      ),
      if (object(input['source'])['kind'] == 'instrument')
        Text(
          '${object(input['source'])['name']} · ${object(input['source'])['device_id']} · ${context.t(object(input['source'])['acquisition'] == 'reported' ? 'brewReported' : 'brewTranscribed')}',
        ),
      if (object(input['calibration'])['kind'] == 'unknown')
        Note(context.t('brewCalibrationUnknown'))
      else ...[
        Text(object(input['calibration'])['details'] as String),
        if (object(input['calibration'])['calibrated_at'] != null)
          Text(
            '${context.t('brewCalibrationAt')}: ${object(object(input['calibration'])['calibrated_at'])['local']} · ${object(object(input['calibration'])['calibrated_at'])['timezone']}',
          ),
        if (object(input['calibration'])['reference_temperature'] != null)
          Text(
            '${context.t('brewReferenceTemperature')}: ${exactNumber(context, object(object(input['calibration'])['reference_temperature'])['value'] as String)} ${context.t(brewReadingUnits[object(object(input['calibration'])['reference_temperature'])['unit']]!)}',
          ),
        if (object(input['calibration'])['source_url'] != null)
          TextButton.icon(
            icon: const Icon(Icons.open_in_new),
            label: Text(context.t('source')),
            onPressed: () => runAction(
              context,
              () => openSource(
                object(input['calibration'])['source_url'] as String,
              ),
            ),
          ),
      ],
      if (input['notes'] != '') Text(input['notes'] as String),
      if (corrects != null)
        Text('${context.t('brewCorrection')}: $corrects · $reason'),
    ],
  );
}
