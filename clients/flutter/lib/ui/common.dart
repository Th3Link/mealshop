import 'package:material_ui/material_ui.dart';

import 'messages.dart';
import '../connection/nextcloud.dart';

Future<void> showFailure(BuildContext context, Object failure) async {
  if (!context.mounted) return;
  final code = failure is ApiFailure ? failure.code : 'request_failed';
  ScaffoldMessenger.of(context)
      .showSnackBar(SnackBar(content: Text(context.t(code))));
}

Future<bool> confirm(
  BuildContext context,
  String title, {
  String? body,
}) async =>
    await showDialog<bool>(
      context: context,
      builder: (context) => AlertDialog(
        title: Text(context.t(title)),
        content: body == null ? null : Text(context.t(body)),
        actions: [
          TextButton(
            onPressed: () => Navigator.pop(context, false),
            child: Text(context.t('cancel')),
          ),
          FilledButton(
            onPressed: () => Navigator.pop(context, true),
            child: Text(
              context.t(switch (title) {
                'confirm_delete' ||
                'delete_all' ||
                'delete_person' ||
                'revoke' => 'delete',
                'discard_draft' => 'discard_changes',
                _ => title,
              }),
            ),
          ),
        ],
      ),
    ) ??
    false;
Future<String?> askName(
  BuildContext context,
  String title, {
  String initial = '',
  String field = 'name',
  bool integer = false,
}) async {
  final input = TextEditingController(text: initial);
  final form = GlobalKey<FormState>();
  final result = await showDialog<String>(
    context: context,
    builder: (context) => AlertDialog(
      title: Text(context.t(title)),
      content: Form(
        key: form,
        child: TextFormField(
          controller: input,
          autofocus: true,
          maxLength: integer ? 7 : 120,
          keyboardType: integer ? TextInputType.number : TextInputType.text,
          decoration: InputDecoration(labelText: context.t(field)),
          validator: (v) {
            if (v == null || v.trim().isEmpty) {
              return context.t('name_required');
            }
            if (integer) {
              final count = int.tryParse(v);
              if (count == null || count < 1 || count > 1000000) {
                return context.t('quantity_invalid');
              }
            }
            return null;
          },
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
              Navigator.pop(context, input.text.trim());
            }
          },
          child: Text(context.t('save')),
        ),
      ],
    ),
  );
  // Dialog exit transitions may still refer to its text controller.
  await Future<void>.delayed(const Duration(milliseconds: 300));
  input.dispose();
  return result;
}

class SectionTitle extends StatelessWidget {
  const SectionTitle(this.text, {super.key, this.trailing});
  final String text;
  final Widget? trailing;
  @override
  Widget build(BuildContext context) => Padding(
    padding: const EdgeInsets.fromLTRB(20, 24, 12, 8),
    child: Row(
      children: [
        Expanded(
          child: Text(text, style: Theme.of(context).textTheme.titleMedium),
        ),
        ?trailing,
      ],
    ),
  );
}

class DraftGuard extends StatefulWidget {
  const DraftGuard({super.key, required this.dirty, required this.child});
  final bool dirty;
  final Widget child;
  @override
  State<DraftGuard> createState() => _DraftGuardState();
}

class _DraftGuardState extends State<DraftGuard> {
  bool discarded = false;
  @override
  Widget build(BuildContext context) => PopScope<Object?>(
    canPop: discarded || !widget.dirty,
    onPopInvokedWithResult: (didPop, _) async {
      if (didPop || !await confirm(context, 'discard_draft') || !mounted) {
        return;
      }
      setState(() => discarded = true);
      WidgetsBinding.instance.addPostFrameCallback((_) {
        if (mounted) Navigator.pop(context);
      });
    },
    child: widget.child,
  );
}
