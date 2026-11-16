import 'package:material_ui/material_ui.dart';

import '../connection/device.dart';
import '../connection/nextcloud.dart';
import '../cooking/device.dart';
import '../shopping/controller.dart';
import 'messages.dart';

class BackupsScreen extends StatefulWidget {
  const BackupsScreen(this.c, {super.key});
  final ShoppingController c;
  @override
  State<BackupsScreen> createState() => _BackupsScreenState();
}

class _BackupsScreenState extends State<BackupsScreen> {
  late final Credential identity = widget.c.credential!;
  bool busy = false;
  String? progress, error;
  Json? status;
  DeviceGateway get device => widget.c.api.device;
  bool get current =>
      mounted && identical(widget.c.credential, identity) && identity.isDevice;
  @override
  void initState() {
    super.initState();
    load();
  }

  Future<void> load({bool keepError = false}) async {
    try {
      final value = await device.status();
      if (current && mounted) {
        setState(() {
          status = value;
          if (!keepError) error = null;
        });
      }
    } on ApiFailure catch (failure) {
      if (current && mounted) setState(() => error = failure.code);
    }
  }

  void stage(String key) {
    if (current && mounted) {
      setState(() {
        busy = true;
        progress = key;
        error = null;
      });
    }
  }

  Future<void> backup() async {
    final password = await passwordDialog(context, creating: true);
    if (password == null || !current || !mounted) return;
    String? id;
    try {
      stage('backup_encrypting');
      final result = await device.command({
        'operation': 'backup',
        'passphrase': password,
      });
      id = result['id'] as String;
      if (!current || !mounted) return;
      stage('backup_destination');
      if (await const BackupDocuments().export(id)) {
        await device.command({'operation': 'exported', 'id': id});
        id = null;
        if (current && mounted) {
          ScaffoldMessenger.of(context)
              .showSnackBar(SnackBar(content: Text(context.t('backup_saved'))));
        }
      }
    } on ApiFailure catch (failure) {
      if (current && mounted) setState(() => error = failure.code);
    } finally {
      if (id != null) {
        try {
          await device.command({'operation': 'discard_export', 'id': id});
        } on ApiFailure {
          /* Retried at next core startup. */
        }
      }
      if (current && mounted) {
        setState(() {
          busy = false;
          progress = null;
        });
        await load(keepError: true);
      }
    }
  }

  Future<void> restore() async {
    String? id;
    String? token;
    var committed = false;
    String? passwordError;
    try {
      stage('backup_select');
      await device.status();
      id = await const BackupDocuments().select();
      if (id == null || !current || !mounted) return;
      while (current) {
        if (!mounted) return;
        setState(() {
          busy = false;
          progress = null;
        });
        final password = await passwordDialog(
          context,
          creating: false,
          issue: passwordError,
        );
        if (password == null || !current || !mounted) return;
        stage('backup_verifying');
        Json preview;
        try {
          preview = await device.command({
            'operation': 'restore',
            'id': id,
            'passphrase': password,
          });
        } on ApiFailure catch (failure) {
          if (failure.code != 'device_restore_password') rethrow;
          passwordError = failure.code;
          continue;
        }
        token = preview['token'] as String;
        if (!current || !mounted) return;
        setState(() {
          busy = false;
          progress = null;
        });
        final confirmed = await showDialog<bool>(
          context: context,
          builder: (context) => AlertDialog(
            scrollable: true,
            title: Text(context.t('backup_restore_preview')),
            content: Column(
              crossAxisAlignment: CrossAxisAlignment.start,
              mainAxisSize: MainAxisSize.min,
              children: [
                Text(context.t('backup_replace_body')),
                const SizedBox(height: 20),
                Text(summary(context, object(preview['manifest']))),
              ],
            ),
            actions: [
              TextButton(
                onPressed: () => Navigator.pop(context, false),
                child: Text(context.t('cancel')),
              ),
              FilledButton(
                onPressed: () => Navigator.pop(context, true),
                child: Text(context.t('backup_replace')),
              ),
            ],
          ),
        );
        if (confirmed != true || !current || !mounted) return;
        stage('backup_restoring');
        await device.command({
          'operation': 'commit_restore',
          'token': preview['token'],
        });
        committed = true;
        // Restored timers keep their saved deadlines; device delivery needs a fresh explicit opt-in.
        try {
          await AndroidReminders().clear();
        } finally {
          await widget.c.useOnDevice();
        }
        return;
      }
    } on ApiFailure catch (failure) {
      if (current && mounted) {
        setState(
          () => error = committed ? 'backup_restored_refresh' : failure.code,
        );
      }
    } finally {
      if (id != null) {
        try {
          if (!committed && token != null) {
            await device.command({
              'operation': 'discard_restore',
              'token': token,
            });
          }
          await device.command({'operation': 'discard_import', 'id': id});
        } on ApiFailure {
          /* Encrypted transfer files are reclaimed at the next core startup. */
        }
      }
      if (current && mounted) {
        setState(() {
          busy = false;
          progress = null;
        });
        await load(keepError: true);
      }
    }
  }

  @override
  Widget build(BuildContext context) => PopScope(
    canPop: !busy,
    child: Scaffold(
      appBar: AppBar(title: Text(context.t('backups'))),
      body: SafeArea(
        child: ListView(
          padding: const EdgeInsets.all(20),
          children: [
            Text(context.t('backup_intro')),
            const SizedBox(height: 24),
            if (status != null) ...[
              Text(
                context.t('backup_last'),
                style: Theme.of(context).textTheme.titleMedium,
              ),
              const SizedBox(height: 8),
              Text(
                status!['last_backup'] == null
                    ? context.t('backup_never')
                    : summary(context, object(status!['last_backup'])),
              ),
              const SizedBox(height: 24),
            ],
            if (status?['storage'] == 'recovery') ...[
              Text(context.t('device_recovery_required')),
              const SizedBox(height: 20),
            ],
            if (status?['cleanup_complete'] == false) ...[
              Text(context.t('device_cleanup_failed')),
              const SizedBox(height: 20),
            ],
            if (busy) ...[
              const LinearProgressIndicator(),
              const SizedBox(height: 12),
              Semantics(liveRegion: true, child: Text(context.t(progress!))),
              const SizedBox(height: 20),
            ],
            if (error != null) ...[
              Semantics(
                liveRegion: true,
                child: Text(
                  context.t(error!),
                  style: TextStyle(color: Theme.of(context).colorScheme.error),
                ),
              ),
              const SizedBox(height: 12),
              TextButton(
                onPressed: busy ? null : load,
                child: Text(context.t('retry')),
              ),
            ],
            FilledButton.icon(
              onPressed: busy || status?['storage'] != 'ready' ? null : backup,
              icon: const Icon(Icons.save_alt),
              label: Text(context.t('backup_create')),
            ),
            const SizedBox(height: 12),
            OutlinedButton.icon(
              onPressed: busy ? null : restore,
              icon: const Icon(Icons.restore),
              label: Text(context.t('backup_restore')),
            ),
            const SizedBox(height: 24),
            Text(
              context.t('backup_passphrase_notice'),
              style: Theme.of(context).textTheme.bodySmall,
            ),
          ],
        ),
      ),
    ),
  );
}

String summary(BuildContext context, Json manifest) {
  final date = DateTime.fromMillisecondsSinceEpoch(
    (manifest['created_at'] as int) * 1000,
  ).toLocal();
  final locale = MaterialLocalizations.of(context);
  final bytes = manifest['bytes'] as int? ?? 0;
  final size = bytes < 1024
      ? '$bytes B'
      : bytes < 1048576
      ? '${(bytes + 1023) ~/ 1024} KiB'
      : '${(bytes + 1048575) ~/ 1048576} MiB';
  return '${locale.formatCompactDate(date)} · ${locale.formatTimeOfDay(TimeOfDay.fromDateTime(date))}\n${context.t('households')}: ${manifest['households']} · ${context.t('backup_size')}: $size';
}

Future<String?> passwordDialog(
  BuildContext context, {
  required bool creating,
  String? issue,
}) => showDialog<String>(
  context: context,
  builder: (_) => _PasswordDialog(creating: creating, issue: issue),
);

class _PasswordDialog extends StatefulWidget {
  const _PasswordDialog({required this.creating, this.issue});
  final bool creating;
  final String? issue;
  @override
  State<_PasswordDialog> createState() => _PasswordDialogState();
}

class _PasswordDialogState extends State<_PasswordDialog> {
  final form = GlobalKey<FormState>();
  final password = TextEditingController(),
      confirmation = TextEditingController();
  bool hidden = true;
  @override
  void dispose() {
    password.clear();
    confirmation.clear();
    password.dispose();
    confirmation.dispose();
    super.dispose();
  }

  @override
  Widget build(BuildContext context) => AlertDialog(
    scrollable: true,
    title: Text(
      context.t(
        widget.creating ? 'backup_password_create' : 'backup_password_enter',
      ),
    ),
    content: Form(
      key: form,
      child: Column(
        mainAxisSize: MainAxisSize.min,
        children: [
          Text(
            context.t(
              widget.creating
                  ? 'backup_password_help'
                  : 'backup_password_restore_help',
            ),
          ),
          if (widget.issue != null) ...[
            const SizedBox(height: 12),
            Semantics(
              liveRegion: true,
              child: Text(
                context.t(widget.issue!),
                style: TextStyle(color: Theme.of(context).colorScheme.error),
              ),
            ),
          ],
          const SizedBox(height: 20),
          TextFormField(
            controller: password,
            autofocus: true,
            obscureText: hidden,
            autocorrect: false,
            enableSuggestions: false,
            decoration: InputDecoration(
              labelText: context.t('backup_password'),
              suffixIcon: IconButton(
                tooltip: context.t(
                  hidden ? 'backup_password_show' : 'backup_password_hide',
                ),
                onPressed: () => setState(() => hidden = !hidden),
                icon: Icon(
                  hidden
                      ? Icons.visibility_outlined
                      : Icons.visibility_off_outlined,
                ),
              ),
            ),
            validator: (value) =>
                value == null ||
                    value.runes.length < 12 ||
                    value.runes.length > 256
                ? context.t('device_passphrase_invalid')
                : null,
          ),
          if (widget.creating) ...[
            const SizedBox(height: 16),
            TextFormField(
              controller: confirmation,
              obscureText: hidden,
              autocorrect: false,
              enableSuggestions: false,
              decoration: InputDecoration(
                labelText: context.t('backup_password_confirm'),
              ),
              validator: (value) => value != password.text
                  ? context.t('backup_password_mismatch')
                  : null,
            ),
          ],
        ],
      ),
    ),
    actions: [
      TextButton(
        onPressed: () => Navigator.pop(context),
        child: Text(context.t('cancel')),
      ),
      FilledButton(
        onPressed: () {
          if (form.currentState!.validate()) {
            Navigator.pop(context, password.text);
          }
        },
        child: Text(context.t('backup_continue')),
      ),
    ],
  );
}
