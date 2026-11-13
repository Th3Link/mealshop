import 'package:material_ui/material_ui.dart';

import '../shopping/controller.dart';
import '../shopping/models.dart';
import '../shopping/quantity.dart';
import '../connection/nextcloud.dart';
import 'common.dart';
import 'add_item.dart';
import 'messages.dart';

class PendingScreen extends StatelessWidget {
  const PendingScreen(this.c, {super.key});
  final ShoppingController c;
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: c,
    builder: (context, _) => Scaffold(
      appBar: AppBar(
        title: Text(context.t('pending')),
        actions: [
          IconButton(
            tooltip: context.t('retry'),
            onPressed: c.busy ? null : c.refresh,
            icon: const Icon(Icons.refresh),
          ),
        ],
      ),
      body: ListView(
        padding: const EdgeInsets.all(16),
        children: [
          Text(context.t('pending_body')),
          const SizedBox(height: 16),
          for (final entry in c.pending)
            Card.outlined(
              child: Padding(
                padding: const EdgeInsets.all(16),
                child: Column(
                  crossAxisAlignment: CrossAxisAlignment.start,
                  children: [
                    Text(
                      entry.command['label'] as String? ??
                          entry.command['name'] as String? ??
                          (entry.command['product'] as Json?)?['name']
                              as String? ??
                          entry.command['barcode'] as String? ??
                          c.shopping?.items
                              .where((i) => i.key == entry.command['key'])
                              .firstOrNull
                              ?.label(c.language) ??
                          context.t('review'),
                      style: Theme.of(context).textTheme.titleMedium,
                    ),
                    if (entry.command['operation'] == 'check')
                      Text(
                        context.t(
                          entry.command['checked'] == true ? 'check' : 'undo',
                        ),
                      ),
                    if (entry.command['product'] is Json)
                      Builder(
                        builder: (context) {
                          final p = entry.command['product'] as Json;
                          final q = object(p['quantity']);
                          return Text(
                            '${p['brand']} · ${Quantity.display(q['amount'] as String, c.language)} ${q['unit']}',
                          );
                        },
                      ),
                    const SizedBox(height: 8),
                    Text(
                      entry.uncertain
                          ? context.t('unknown')
                          : entry.error == null
                          ? context.t('pending')
                          : context.t(entry.error!),
                    ),
                    if (entry.error != null && !entry.uncertain) ...[
                      const SizedBox(height: 12),
                      Text(context.t('review_body')),
                      if (entry.command['key'] != null)
                        for (final item
                            in c.shopping?.items
                                    .where((i) => i.key == entry.command['key'])
                                    .toList() ??
                                const <ShoppingItem>[])
                          Text(
                            '${context.t('required')}: ${Quantity.display(item.remaining, c.language)} ${item.unit}',
                          ),
                      Wrap(
                        spacing: 8,
                        children: [
                          if (entry.command['operation'] == 'add_manual')
                            TextButton(
                              onPressed: c.busy
                                  ? null
                                  : () => Navigator.push<void>(
                                      context,
                                      MaterialPageRoute(
                                        builder: (_) =>
                                            AddItemScreen(c, pending: entry),
                                      ),
                                    ),
                              child: Text(context.t('edit')),
                            ),
                          TextButton(
                            onPressed: c.busy || !c.online
                                ? null
                                : () async {
                                    final observedRevision =
                                        c.shopping?.revision;
                                    if (await confirm(
                                      context,
                                      'apply_reviewed',
                                      body: 'review_body',
                                    )) {
                                      try {
                                        await c.retryReviewed(
                                          entry.id,
                                          expectedRevision: observedRevision,
                                        );
                                      } catch (e) {
                                        if (context.mounted) {
                                          await showFailure(context, e);
                                        }
                                      }
                                    }
                                  },
                            child: Text(context.t('apply_reviewed')),
                          ),
                          TextButton(
                            onPressed: c.busy
                                ? null
                                : () async {
                                    try {
                                      await c.discard(entry.id);
                                    } catch (e) {
                                      if (context.mounted) {
                                        await showFailure(context, e);
                                      }
                                    }
                                  },
                            child: Text(context.t('discard')),
                          ),
                        ],
                      ),
                    ] else if (!entry.attempted)
                      TextButton(
                        onPressed: c.busy ? null : () => c.discard(entry.id),
                        child: Text(context.t('discard')),
                      ),
                  ],
                ),
              ),
            ),
        ],
      ),
    ),
  );
}
