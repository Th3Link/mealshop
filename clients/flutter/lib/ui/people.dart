import 'package:material_ui/material_ui.dart';

import '../connection/nextcloud.dart';
import '../content/api.dart';
import 'content_common.dart';
import 'person_tour.dart';
import 'activity.dart';
import 'messages.dart';

class PeopleScreen extends StatefulWidget {
  const PeopleScreen(this.api, {super.key, this.activity = false});
  final HouseholdApi api;
  final bool activity;
  @override
  State<PeopleScreen> createState() => _PeopleScreenState();
}

class _PeopleScreenState extends State<PeopleScreen> {
  int version = 0;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(
      title: Text(context.t(widget.activity ? 'activities' : 'people')),
    ),
    floatingActionButton: !widget.activity && widget.api.writable
        ? FloatingActionButton.extended(
            onPressed: () async {
              await openPage<void>(context, PersonTour(widget.api));
              if (mounted) setState(() => version++);
            },
            icon: const Icon(Icons.person_add_outlined),
            label: Text(
              context.t(
                widget.api.controller.draft('person:new') == null
                    ? 'new_person'
                    : 'resume',
              ),
            ),
          )
        : null,
    body: Resource<Json>(
      key: ValueKey(version),
      sensitive: widget.activity,
      load: () async {
        final profiles = await widget.api.household('/profiles');
        profiles['access'] = await widget.api.household('/people/access');
        return profiles;
      },
      builder: (context, data, reload) {
        final members = objects(data['members']);
        final caps = objects(object(data['access'])['people']);
        if (members.isEmpty) {
          return EmptyContent(
            icon: Icons.people_outline,
            title: 'nothing_yet',
            body: 'tour_intro',
          );
        }
        return ListView(
          padding: const EdgeInsets.only(bottom: 100),
          children: [
            Padding(
              padding: const EdgeInsets.symmetric(horizontal: 16),
              child: Note(
                context.t(
                  widget.activity ? 'activity_access_note' : 'exceptions_note',
                ),
              ),
            ),
            for (final member in members)
              Builder(
                builder: (context) {
                  final cap = caps
                      .where((c) => c['person_id'] == member['id'])
                      .first;
                  final rules = object(member['rules'] ?? <String, dynamic>{});
                  final exceptions = rules['exceptions'] as List? ?? [];
                  return ListTile(
                    leading: CircleAvatar(
                      child: Text((member['name'] as String).characters.first),
                    ),
                    title: Text(member['name'] as String),
                    subtitle: Text(
                      widget.activity
                          ? context.t(
                              cap['can_edit'] == true
                                  ? 'edit_access'
                                  : cap['can_read'] == true
                                  ? 'read'
                                  : 'no_access',
                            )
                          : '${context.t(rules['preset'] as String? ?? 'unrestricted')}${exceptions.isEmpty ? '' : ' · ${context.t('exceptions')}: ${exceptions.length}'}',
                    ),
                    trailing: const Icon(Icons.chevron_right),
                    onTap: () async {
                      if (widget.activity) {
                        await openPage<void>(
                          context,
                          ActivityScreen(
                            widget.api,
                            member,
                            data['revision'] as int,
                          ),
                        );
                      } else {
                        await openPage<void>(
                          context,
                          widget.api.writable
                              ? PersonTour(
                                  widget.api,
                                  profiles: data,
                                  person: member,
                                )
                              : PublicPersonDetail(
                                  widget.api,
                                  member['id'] as String,
                                ),
                        );
                      }
                      await reload();
                    },
                  );
                },
              ),
          ],
        );
      },
    ),
  );
}

class PublicPersonDetail extends StatelessWidget {
  const PublicPersonDetail(this.api, this.id, {super.key});
  final HouseholdApi api;
  final String id;
  @override
  Widget build(BuildContext context) => Scaffold(
    appBar: AppBar(title: Text(context.t('people'))),
    body: Resource<Json>(
      load: () async {
        final p = await api.household('/profiles');
        final c = await api.household('/ingredients');
        return {
          'person': objects(p['members']).where((m) => m['id'] == id).first,
          'catalogue': c,
        };
      },
      builder: (context, data, reload) => ListView(
        padding: const EdgeInsets.all(20),
        children: [
          Note(context.t('reader')),
          Text(
            personDescription(
              object(data['person']),
              Catalogue(object(data['catalogue'])),
              api.controller.language,
            ),
          ),
          Note(context.t('allergen_review'), warning: true),
        ],
      ),
    ),
  );
}
