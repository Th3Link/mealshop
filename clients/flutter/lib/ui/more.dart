import 'package:material_ui/material_ui.dart';

import '../shopping/controller.dart';
import 'common.dart';
import 'home.dart';
import 'messages.dart';
import 'pending.dart';
import 'products.dart';
import 'shops.dart';
import '../content/api.dart';
import 'content_common.dart';
import 'people.dart';
import 'ingredients.dart';
import 'collections.dart';
import 'sharing.dart';

class MoreScreen extends StatelessWidget {
  const MoreScreen(this.c, {super.key});
  final ShoppingController c;
  @override
  Widget build(BuildContext context) => ListView(
    children: [
      SectionTitle(context.t('account')),
      ListTile(
        leading: const Icon(Icons.account_circle_outlined),
        title: Text(c.credential!.user),
        subtitle: Text(c.credential!.server.root.toString()),
      ),
      ListTile(
        leading: const Icon(Icons.people_outline),
        title: Text(context.t('households')),
        subtitle: Text(c.home?.name ?? context.t('no_home')),
        trailing: const Icon(Icons.chevron_right),
        onTap: () => showHomes(context, c),
      ),
      ListTile(
        leading: const Icon(Icons.folder_shared_outlined),
        title: Text(context.t('shared')),
        trailing: const Icon(Icons.chevron_right),
        onTap: () => openPage<void>(context, SharedScreen(HouseholdApi(c))),
      ),
      if (c.home != null) ...[
        SectionTitle(context.t('library')),
        for (final item in <(String, IconData, Widget)>[
          ('people', Icons.people_outline, PeopleScreen(HouseholdApi(c))),
          (
            'activities',
            Icons.directions_run,
            PeopleScreen(HouseholdApi(c), activity: true),
          ),
          (
            'ingredients',
            Icons.eco_outlined,
            IngredientsScreen(HouseholdApi(c)),
          ),
          (
            'collections',
            Icons.collections_bookmark_outlined,
            CollectionsScreen(HouseholdApi(c)),
          ),

          if (c.home!.role == 'owner')
            ('shares', Icons.share_outlined, SharesScreen(HouseholdApi(c))),
        ])
          ListTile(
            leading: Icon(item.$2),
            title: Text(context.t(item.$1)),
            trailing: const Icon(Icons.chevron_right),
            onTap: () => openPage<void>(context, item.$3),
          ),
        SectionTitle(context.t('shopping')),
        ListTile(
          leading: const Icon(Icons.storefront_outlined),
          title: Text(context.t('shops')),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => Navigator.push<void>(
            context,
            MaterialPageRoute(builder: (_) => ShopsScreen(c)),
          ),
        ),
        ListTile(
          leading: const Icon(Icons.qr_code_outlined),
          title: Text(context.t('products')),
          trailing: const Icon(Icons.chevron_right),
          onTap: () => Navigator.push<void>(
            context,
            MaterialPageRoute(builder: (_) => ProductsScreen(c)),
          ),
        ),
        ListTile(
          leading: const Icon(Icons.sync),
          title: Text(context.t('pending')),
          trailing: Text('${c.pending.length}'),
          onTap: () => Navigator.push<void>(
            context,
            MaterialPageRoute(builder: (_) => PendingScreen(c)),
          ),
        ),
      ],
      SectionTitle(context.t('settings')),
      Padding(
        padding: const EdgeInsets.symmetric(horizontal: 20, vertical: 8),
        child: DropdownButtonFormField<String>(
          initialValue: c.language,
          decoration: InputDecoration(labelText: context.t('language')),
          items: const [
            DropdownMenuItem(value: 'de', child: Text('Deutsch')),
            DropdownMenuItem(value: 'en', child: Text('English')),
          ],
          onChanged: (v) {
            if (v != null) c.setLanguage(v);
          },
        ),
      ),
      ListTile(
        leading: const Icon(Icons.lock_outline),
        title: Text(context.t('privacy')),
        subtitle: Text(context.t('privacy_body')),
      ),
      Padding(
        padding: const EdgeInsets.all(20),
        child: OutlinedButton.icon(
          onPressed: () async {
            if (await confirm(context, 'sign_out', body: 'sign_out_body')) {
              await c.signOut();
            }
          },
          icon: const Icon(Icons.logout),
          label: Text(context.t('sign_out')),
        ),
      ),
      ListTile(
        leading: const Icon(Icons.info_outline),
        title: Text(context.t('licences')),
        onTap: () => showLicensePage(
          context: context,
          applicationName: 'Mealshop',
          applicationVersion: '0.2.0',
          applicationLegalese: 'AGPL-3.0-only',
        ),
      ),
      const Padding(
        padding: EdgeInsets.fromLTRB(20, 0, 20, 24),
        child: Text('Mealshop · 0.2.0 · AGPL-3.0-only'),
      ),
    ],
  );
}
