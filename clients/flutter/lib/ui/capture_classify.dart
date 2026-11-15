import 'package:material_ui/material_ui.dart';

import '../capture/import.dart';
import '../connection/nextcloud.dart';
import 'messages.dart';

/// Users classify the exact OCR lines. No ingredient, amount or semantic inference.
class CaptureClassify extends StatefulWidget {
  const CaptureClassify(this.text, {super.key});
  final String text;
  @override
  State<CaptureClassify> createState() => _CaptureClassifyState();
}

class _CaptureClassifyState extends State<CaptureClassify> {
  late final List<String> lines;
  final Map<int, String> roles = {};
  @override
  void initState() {
    super.initState();
    lines = widget.text.split('\n').where((s) => s.trim().isNotEmpty).toList();
  }

  Json get candidate => {
    ...blankCandidate(),
    'title': roles.entries
        .where((e) => e.value == 'title')
        .map((e) => lines[e.key])
        .firstOrNull,
    'ingredients': [
      for (var i = 0; i < lines.length; i++)
        if (roles[i] == 'ingredients') lines[i],
    ],
    'steps': [
      for (var i = 0; i < lines.length; i++)
        if (roles[i] == 'steps') {'text': lines[i], 'section': null},
    ],
  };
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('capture_assign'))),
    body: Column(
      children: [
        Padding(
          padding: const EdgeInsets.all(16),
          child: Text(context.t('capture_assign_help')),
        ),
        Expanded(
          child: ListView.builder(
            itemCount: lines.length,
            itemBuilder: (context, i) => ListTile(
              title: Text(lines[i]),
              subtitle: Text(context.t(roles[i] ?? 'capture_unused')),
              trailing: PopupMenuButton<String>(
                tooltip: context.t('capture_assign'),
                icon: Icon(
                  roles.containsKey(i) ? Icons.check_circle_outline : Icons.add,
                ),
                itemBuilder: (context) => [
                  for (final role in [
                    'title',
                    'ingredients',
                    'steps',
                    'capture_unused',
                  ])
                    PopupMenuItem(value: role, child: Text(context.t(role))),
                ],
                onSelected: (role) => setState(() {
                  if (role == 'title') {
                    roles.removeWhere((_, v) => v == 'title');
                  }
                  if (role == 'capture_unused') {
                    roles.remove(i);
                  } else {
                    roles[i] = role;
                  }
                }),
              ),
            ),
          ),
        ),
        SafeArea(
          top: false,
          child: Padding(
            padding: const EdgeInsets.all(16),
            child: FilledButton(
              onPressed:
                  (candidate['ingredients'] as List).length > 100 ||
                      (candidate['steps'] as List).length > 100
                  ? null
                  : () => Navigator.pop(context, candidate),
              child: Text(context.t('done')),
            ),
          ),
        ),
      ],
    ),
  );
}
