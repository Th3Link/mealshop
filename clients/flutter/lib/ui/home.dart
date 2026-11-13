import 'package:material_ui/material_ui.dart';

import '../shopping/controller.dart';
import 'common.dart';
import 'messages.dart';
import 'shopping_list.dart';
import 'more.dart';

class HomeScreen extends StatefulWidget {
  const HomeScreen(this.controller, {super.key});
  final ShoppingController controller;
  @override
  State<HomeScreen> createState() => _HomeScreenState();
}

class _HomeScreenState extends State<HomeScreen> {
  int tab = 0;
  @override
  Widget build(BuildContext context) {
    final c = widget.controller;
    return Scaffold(
      appBar: AppBar(
        title: Text(context.t(tab == 0 ? 'shopping' : 'more')),
        actions: [
          IconButton(
            tooltip: context.t('sync'),
            onPressed: c.busy ? null : c.refresh,
            icon: const Icon(Icons.refresh),
          ),
        ],
      ),
      bottomNavigationBar: NavigationBar(
        selectedIndex: tab,
        onDestinationSelected: (v) => setState(() => tab = v),
        destinations: [
          NavigationDestination(
            icon: const Icon(Icons.shopping_basket_outlined),
            selectedIcon: const Icon(Icons.shopping_basket),
            label: context.t('shopping'),
          ),
          NavigationDestination(
            icon: const Icon(Icons.more_horiz),
            label: context.t('more'),
          ),
        ],
      ),
      body: SafeArea(
        child: Column(
          children: [
            Padding(
              padding: const EdgeInsets.fromLTRB(16, 0, 16, 8),
              child: OutlinedButton.icon(
                onPressed: () => showHomes(context, c),
                icon: const Icon(Icons.people_outline),
                label: Row(
                  children: [
                    Expanded(
                      child: Padding(
                        padding: const EdgeInsets.symmetric(vertical: 10),
                        child: Text(
                          c.home?.name ?? context.t('households'),
                          maxLines: 3,
                        ),
                      ),
                    ),
                    const Icon(Icons.expand_more),
                  ],
                ),
              ),
            ),
            if (c.error != null && c.error != 'connection_failed')
              Material(
                color: Theme.of(context).colorScheme.errorContainer,
                child: Padding(
                  padding: const EdgeInsets.all(12),
                  child: Row(
                    children: [
                      const Icon(Icons.info_outline),
                      const SizedBox(width: 12),
                      Expanded(child: Text(context.t('attention'))),
                      IconButton(
                        tooltip: context.t('details'),
                        onPressed: () => showDialog<void>(
                          context: context,
                          builder: (context) => AlertDialog(
                            scrollable: true,
                            title: Text(context.t('attention')),
                            content: Text(context.t(c.error!)),
                            actions: [
                              TextButton(
                                onPressed: () => Navigator.pop(context),
                                child: Text(context.t('close')),
                              ),
                            ],
                          ),
                        ),
                        icon: const Icon(Icons.info_outline),
                      ),
                      IconButton(
                        tooltip: context.t('retry'),
                        onPressed: c.busy ? null : c.refresh,
                        icon: const Icon(Icons.refresh),
                      ),
                    ],
                  ),
                ),
              ),
            if (c.busy) const LinearProgressIndicator(minHeight: 2),
            Expanded(
              child: tab == 1
                  ? MoreScreen(c)
                  : c.home == null
                  ? NoHousehold(c)
                  : ShoppingListScreen(c),
            ),
          ],
        ),
      ),
    );
  }
}

Future<void> showHomes(BuildContext context, ShoppingController c) async {
  await showModalBottomSheet<void>(
    context: context,
    isScrollControlled: true,
    useSafeArea: true,
    showDragHandle: true,
    builder: (sheetContext) => DraggableScrollableSheet(
      expand: false,
      initialChildSize: .6,
      builder: (sheetContext, scroll) => ListView(
        controller: scroll,
        children: [
          SectionTitle(context.t('households')),
          for (final home in c.homes)
            ListTile(
              leading: Icon(
                home.id == c.selected
                    ? Icons.check_circle
                    : Icons.home_outlined,
              ),
              title: Text(home.name),
              subtitle: Text(context.t(home.role)),
              onTap: () async {
                Navigator.pop(context);
                await c.select(home.id);
              },
            ),
          for (final invitation in c.invitations)
            ListTile(
              leading: const Icon(Icons.mail_outline),
              title: Text(invitation.name),
              subtitle: Text(context.t('invitations')),
              onTap: () async {
                Navigator.pop(context);
                if (await confirm(context, 'accept')) {
                  await c.invitation(invitation.id, true);
                }
              },
            ),
          ListTile(
            leading: const Icon(Icons.add),
            title: Text(context.t('new_home')),
            onTap: () async {
              Navigator.pop(context);
              final name = await askName(context, 'new_home');
              if (name != null) {
                try {
                  await c.createHome(name);
                } catch (e) {
                  if (context.mounted) await showFailure(context, e);
                }
              }
            },
          ),
        ],
      ),
    ),
  );
}

class NoHousehold extends StatelessWidget {
  const NoHousehold(this.c, {super.key});
  final ShoppingController c;
  @override
  Widget build(BuildContext context) => ListView(
    padding: const EdgeInsets.all(24),
    children: [
      const SizedBox(height: 48),
      const Icon(Icons.home_outlined, size: 56),
      const SizedBox(height: 24),
      Text(
        context.t(c.homes.isEmpty ? 'no_home' : 'households'),
        style: Theme.of(context).textTheme.headlineSmall,
      ),
      const SizedBox(height: 12),
      Text(context.t(c.homes.isEmpty ? 'no_home_body' : 'choose_home_body')),
      if (c.homes.isNotEmpty)
        for (final home in c.homes)
          ListTile(
            leading: const Icon(Icons.home_outlined),
            title: Text(home.name),
            subtitle: Text(context.t(home.role)),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => c.select(home.id),
          ),
      const SizedBox(height: 24),
      OutlinedButton.icon(
        onPressed: c.busy
            ? null
            : () async {
                final name = await askName(context, 'new_home');
                if (name != null) {
                  try {
                    await c.createHome(name);
                  } catch (e) {
                    if (context.mounted) await showFailure(context, e);
                  }
                }
              },
        icon: const Icon(Icons.add),
        label: Text(context.t('new_home')),
      ),
      for (final invitation in c.invitations)
        ListTile(
          title: Text(invitation.name),
          subtitle: Text(context.t(invitation.role)),
          trailing: Wrap(
            children: [
              IconButton(
                tooltip: context.t('decline'),
                onPressed: () async {
                  try {
                    await c.invitation(invitation.id, false);
                  } catch (e) {
                    if (context.mounted) await showFailure(context, e);
                  }
                },
                icon: const Icon(Icons.close),
              ),
              IconButton(
                tooltip: context.t('accept'),
                onPressed: () async {
                  try {
                    await c.invitation(invitation.id, true);
                  } catch (e) {
                    if (context.mounted) await showFailure(context, e);
                  }
                },
                icon: const Icon(Icons.check),
              ),
            ],
          ),
        ),
    ],
  );
}
