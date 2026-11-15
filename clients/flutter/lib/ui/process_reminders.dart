import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import '../cooking/controller.dart';
import 'common.dart';
import 'content_common.dart';
import 'messages.dart';

class ProcessReminders extends StatefulWidget {
  const ProcessReminders(this.c, this.data, {super.key, required this.command});
  final CookingController c;
  final Json data;
  final Future<void> Function(Json) command;
  @override
  State<ProcessReminders> createState() => _ProcessRemindersState();
}

class _ProcessRemindersState extends State<ProcessReminders> {
  bool busy = false;
  Future<void> act(Future<void> Function() action) async {
    setState(() => busy = true);
    await runAction(context, action);
    if (mounted) setState(() => busy = false);
  }

  @override
  Widget build(BuildContext context) {
    final c = widget.c,
        data = widget.data,
        run = object(data['run']),
        id = run['id'] as String;
    return Column(
      crossAxisAlignment: CrossAxisAlignment.start,
      children: [
        SectionTitle(context.t('cookNotifications')),
        SwitchListTile(
          contentPadding: EdgeInsets.zero,
          title: Text(context.t('cook_local_reminders')),
          subtitle: Text(
            context.t(
              c.deviceStatus['precision'] == 'exact'
                  ? 'cook_local_exact'
                  : 'cook_local_inexact',
            ),
          ),
          value: c.enabled(id),
          onChanged: busy ? null : (v) => act(() => c.setReminders(id, v)),
        ),
        Note(context.t('cook_local_help')),
        Note(context.t('cook_local_horizon')),
        if (c.enabled(id) &&
            c.deviceStatus['permission'] == 'granted' &&
            c.deviceStatus['precision'] == 'inexact')
          TextButton.icon(
            icon: const Icon(Icons.alarm),
            label: Text(context.t('cook_allow_precise')),
            onPressed: busy
                ? null
                : () => act(() async {
                    await c.device.requestPrecision();
                    await c.reconcileReminders();
                  }),
          ),
        if (c.deviceStatus['permission'] != 'granted')
          Note(
            context.t(
              c.deviceStatus['permission'] == 'denied'
                  ? 'cookPermissionDeniedShort'
                  : 'cookPermissionUnsupportedShort',
            ),
            warning: true,
          ),
        if (c.limitedReminders)
          Note(context.t('cook_local_limited'), warning: true),
        if (data['notification_transport'] == true && run['status'] == 'active')
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(context.t('cookSubscribe')),
            subtitle: Text(context.t('cookNotificationHelp')),
            value: (run['subscription'] as Json?)?['enabled'] == true,
            onChanged:
                !c.writable ||
                    busy ||
                    c.pending.isNotEmpty ||
                    !c.api.controller.online
                ? null
                : (v) => act(
                    () => widget.command({
                      'operation': 'subscribe',
                      'run_id': id,
                      'enabled': v,
                      'language': c.api.controller.language,
                      'permission': 'default',
                    }),
                  ),
          ),
        for (final notice in objects(data['notices']))
          ListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(
              context.t(
                {
                  'prepare': 'cookEventPrepare',
                  'start': 'cookEventStart',
                  'active_ended': 'cookEventActive',
                  'timer_ended': 'cookEventEnd',
                }[notice['event']]!,
              ),
            ),
            subtitle: Text(
              '${context.t({'queued': 'cookDeliveryQueued', 'sending': 'cookDeliverySending', 'accepted': 'cookDeliveryAccepted', 'failed': 'cookDeliveryFailed', 'unknown': 'cookDeliveryUnknown', 'missed': 'cookDeliveryMissed', 'cancelled': 'cookCancelled'}[object(notice['delivery'])['state']]!)}${notice['acknowledged_at'] == null ? '' : ' · ${context.t('cookAcknowledged')}'}',
            ),
            trailing: PopupMenuButton<String>(
              tooltip: context.t('details'),
              enabled:
                  c.writable &&
                  !busy &&
                  c.pending.isEmpty &&
                  c.api.controller.online,
              itemBuilder: (context) => [
                PopupMenuItem(
                  value: 'acknowledge',
                  child: Text(context.t('cookAcknowledge')),
                ),
                if ([
                  'failed',
                  'unknown',
                ].contains(object(notice['delivery'])['state']))
                  PopupMenuItem(
                    value: 'retry',
                    child: Text(context.t('retry')),
                  ),
              ],
              onSelected: (action) async {
                final risk = object(notice['delivery'])['state'] == 'unknown';
                if (action == 'retry' &&
                    risk &&
                    !await confirm(context, 'retry', body: 'cookRetryRisk')) {
                  return;
                }
                if (mounted) {
                  await act(
                    () => widget.command({
                      'operation': action,
                      'notice_id': notice['id'],
                      if (action == 'retry') 'duplicate_risk_accepted': risk,
                    }),
                  );
                }
              },
            ),
          ),
      ],
    );
  }
}
