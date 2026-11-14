import 'package:material_ui/material_ui.dart';

import '../content/api.dart';
import '../connection/nextcloud.dart';
import 'content_common.dart';
import 'common.dart';
import 'messages.dart';

class LearningScreen extends StatelessWidget {
  const LearningScreen(this.api, {super.key});
  final HouseholdApi api;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('historyLearning'))),
    body: Resource<Json>(
      sensitive: true,
      pollInterval: null,
      load: () async {
        final v = await Future.wait([
          api.household('/learning'),
          api.household('/profiles'),
        ]);
        return {'learning': v[0], 'profiles': v[1]};
      },
      builder: (context, data, reload) => LearningForm(
        api,
        data,
        reload,
        key: ValueKey(object(data['learning'])['revision']),
      ),
    ),
  );
}

class LearningForm extends StatefulWidget {
  const LearningForm(this.api, this.data, this.reload, {super.key});
  final HouseholdApi api;
  final Json data;
  final Future<void> Function() reload;
  @override
  State<LearningForm> createState() => _LearningFormState();
}

class _LearningFormState extends State<LearningForm> {
  late bool enabled =
      object(object(widget.data['learning'])['policy'])['enabled'] as bool;
  late final Set<String> scope =
      (object(object(widget.data['learning'])['policy'])['member_ids'] as List)
          .cast<String>()
          .toSet();
  bool busy = false;
  String? error;
  Future<void> apply({bool reset = false, bool clear = false}) async {
    if (!await confirm(
          context,
          clear
              ? 'historyClear'
              : reset
              ? 'historyLearningReset'
              : 'historyLearningSave',
          body: clear
              ? 'historyClearConfirm'
              : reset
              ? 'historyLearningResetConfirm'
              : 'historyLearningNew',
        ) ||
        !mounted) {
      return;
    }
    setState(() => busy = true);
    try {
      await widget.api.household(
        clear ? '/history' : '/learning',
        method: clear ? 'DELETE' : 'PUT',
        body: {
          'expected_revision': object(widget.data['learning'])['revision'],
          if (clear) 'confirmed': true,
          if (!clear) ...{
            'expected_profile_revision': object(
              widget.data['profiles'],
            )['revision'],
            'policy': {
              'enabled': reset ? false : enabled,
              'member_ids': reset || !enabled ? <String>[] : scope.toList(),
            },
          },
        },
      );
      await widget.reload();
    } catch (e) {
      if (mounted) {
        setState(() => error = e is ApiFailure ? e.code : 'request_failed');
      }
    } finally {
      if (mounted) setState(() => busy = false);
    }
  }

  @override
  Widget build(BuildContext context) {
    final learning = object(widget.data['learning']);
    return ListView(
      padding: const EdgeInsets.all(16),
      children: [
        Note(context.t('historyLearningHelp')),
        Note(context.t('historyLearningNew')),
        if (error != null) ...[
          Note(context.t(error!), warning: true),
          TextButton(
            onPressed: () async {
              if (await confirm(
                context,
                'history_rebase',
                body: 'history_rebase_help',
              )) {
                await widget.reload();
              }
            },
            child: Text(context.t('historyReload')),
          ),
        ],
        Text(
          context.t(
            object(learning['policy'])['enabled'] == true
                ? 'historyLearningOn'
                : 'historyLearningOff',
          ),
          style: Theme.of(context).textTheme.titleLarge,
        ),
        for (final pair in {
          'historyRecordedCooks': 'history_records',
          'historyLearnedCooks': 'learned_cooks',
          'historyLearnedRecipes': 'learned_recipes',
        }.entries)
          Text('${context.t(pair.key)}: ${learning[pair.value]}'),
        Text(
          '${context.t('history_epoch')}: ${learning['epoch']} · ${learning['model_version']}',
        ),
        if (!widget.api.owner) Note(context.t('historyOwnerOnly')),
        if (widget.api.owner) ...[
          SwitchListTile(
            contentPadding: EdgeInsets.zero,
            title: Text(context.t('historyLearningStart')),
            value: enabled,
            onChanged: busy ? null : (v) => setState(() => enabled = v),
          ),
          if (enabled) ...[
            SectionTitle(context.t('historyLearningScope')),
            for (final p in objects(object(widget.data['profiles'])['members']))
              CheckboxListTile(
                contentPadding: EdgeInsets.zero,
                title: Text(p['name'] as String),
                value: scope.contains(p['id']),
                onChanged: busy
                    ? null
                    : (v) => setState(() {
                        if (v == true) {
                          scope.add(p['id'] as String);
                        } else {
                          scope.remove(p['id']);
                        }
                      }),
              ),
          ],
          FilledButton(
            onPressed: busy || !widget.api.controller.online ? null : apply,
            child: Text(context.t('historyLearningSave')),
          ),
          TextButton(
            onPressed: busy || !widget.api.controller.online
                ? null
                : () => apply(reset: true),
            child: Text(context.t('historyLearningReset')),
          ),
          const Divider(),
          TextButton(
            onPressed: busy || !widget.api.controller.online
                ? null
                : () => apply(clear: true),
            child: Text(context.t('historyClear')),
          ),
        ],
      ],
    );
  }
}
