import 'package:material_ui/material_ui.dart';

import '../content/api.dart';
import '../connection/nextcloud.dart';
import '../planning/data.dart';
import 'messages.dart';
import 'content_common.dart';

class DayButton extends StatelessWidget {
  const DayButton(this.label, this.value, this.changed, {super.key});
  final String label, value;
  final ValueChanged<String> changed;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.symmetric(vertical: 6),
    child: OutlinedButton.icon(
      icon: const Icon(Icons.calendar_today_outlined),
      label: Padding(
        padding: const EdgeInsets.symmetric(vertical: 12),
        child: Text(
          '${context.t(label)} · ${MaterialLocalizations.of(context).formatMediumDate(civilDay(value))}',
        ),
      ),
      onPressed: () async {
        final date = await showDatePicker(
          context: context,
          initialDate: civilDay(value),
          firstDate: DateTime(1900),
          lastDate: DateTime(2200),
        );
        if (date != null) changed(isoDay(date));
      },
    ),
  );
}

class AttendeeChoices extends StatelessWidget {
  const AttendeeChoices(this.data, this.selected, this.changed, {super.key});
  final PlanningData data;
  final List<String> selected;
  final ValueChanged<List<String>> changed;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        context.t('attendees'),
        style: Theme.of(context).textTheme.titleMedium,
      ),
      for (final p in data.people)
        CheckboxListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(p['name'] as String),
          value: selected.contains(p['id']),
          onChanged: (v) => changed(
            v == true
                ? [...selected, p['id'] as String]
                : selected.where((id) => id != p['id']).toList(),
          ),
        ),
    ],
  );
}

class ShoppingDelta extends StatelessWidget {
  const ShoppingDelta(this.rows, this.data, {super.key});
  final List<Json> rows;
  final PlanningData data;
  @override
  Widget build(BuildContext context) => Column(
    crossAxisAlignment: CrossAxisAlignment.start,
    children: [
      Text(
        context.t('shopping_delta'),
        style: Theme.of(context).textTheme.titleMedium,
      ),
      if (rows.isEmpty)
        Padding(
          padding: const EdgeInsets.symmetric(vertical: 12),
          child: Text(context.t('no_quantity_change')),
        ),
      for (final r in rows)
        ListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(
            data.catalogue.find(r['ingredient_id'] as String) != null
                ? data.catalogue.name(
                    data.catalogue.find(r['ingredient_id'] as String)!,
                    Localizations.localeOf(context).languageCode,
                  )
                : r['ingredient_id'] as String,
          ),
          subtitle: Text(
            '${decimal(r['grams_delta'] as String, Localizations.localeOf(context).languageCode)} g',
          ),
        ),
      Note(context.t('plan_delta_note')),
    ],
  );
}
