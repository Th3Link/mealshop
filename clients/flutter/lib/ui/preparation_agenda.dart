import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../planning/data.dart';
import 'content_common.dart';
import 'common.dart';
import 'cooking_scope.dart';
import 'cooking.dart';
import 'brewing_detail.dart';
import 'messages.dart';

class PreparationAgenda extends StatefulWidget {
  const PreparationAgenda(this.api, this.day, {super.key});
  final HouseholdApi api;
  final DateTime day;
  @override
  State<PreparationAgenda> createState() => _PreparationAgendaState();
}

class _PreparationAgendaState extends State<PreparationAgenda> {
  int offset = 0;
  @override
  void didUpdateWidget(PreparationAgenda oldWidget) {
    super.didUpdateWidget(oldWidget);
    if (oldWidget.day != widget.day) offset = 0;
  }

  Future<Json> load() {
    final from =
        widget.day.subtract(const Duration(hours: 14)).millisecondsSinceEpoch ~/
        1000;
    final until =
        widget.day.add(const Duration(hours: 38)).millisecondsSinceEpoch ~/
        1000;
    return widget.api.household(
      '/cooking/calendar?from=$from&until=$until&offset=$offset&limit=100',
    );
  }

  @override
  Widget build(BuildContext context) => Resource<Json>(
    key: ValueKey('${isoDay(widget.day)}-$offset'),
    sensitive: true,
    inline: true,
    pollInterval: null,
    load: load,
    builder: (context, data, reload) {
      final entries = objects(data['entries'])
          .where((e) => (e['local'] as String).startsWith(isoDay(widget.day)))
          .toList();
      final c = CookingScope.maybeOf(context);
      return Column(
        crossAxisAlignment: CrossAxisAlignment.start,
        children: [
          SectionTitle(context.t('cook_agenda')),
          Note(context.t('cook_agenda_zone')),
          if (entries.isEmpty) Text(context.t('cook_agenda_empty')),
          for (final e in entries)
            ListTile(
              contentPadding: EdgeInsets.zero,
              leading: Icon(
                e['event'] == 'deadline'
                    ? Icons.timer_outlined
                    : Icons.schedule,
              ),
              title: Text(
                '${(e['local'] as String).substring(11)} · ${e['step_name']}',
              ),
              subtitle: Text(
                '${e['run_name']} · ${e['timezone']} · ${context.t(e['event'] == 'deadline' ? 'cookDeadline' : 'cookPlannedStart')}',
              ),
              trailing: const Icon(Icons.chevron_right),
              onTap: c == null
                  ? null
                  : () async {
                      await openPage<void>(
                        context,
                        ['beer', 'cider', 'wine', 'mead'].contains(e['kind'])
                            ? BrewingDetailScreen(c, e['run_id'] as String)
                            : CookingDetailScreen(c, e['run_id'] as String),
                      );
                      await reload();
                    },
            ),
          Wrap(
            children: [
              if (offset > 0)
                TextButton(
                  onPressed: () => setState(() => offset -= 100),
                  child: Text(context.t('previous_page')),
                ),
              if (offset + (data['limit'] as int) < (data['total'] as int))
                TextButton(
                  onPressed: () => setState(() => offset += 100),
                  child: Text(context.t('next_page')),
                ),
            ],
          ),
          const Divider(),
        ],
      );
    },
  );
}
