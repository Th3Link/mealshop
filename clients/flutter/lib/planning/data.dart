import '../content/api.dart';
import '../connection/nextcloud.dart';

const mealKeys = ['breakfast', 'lunch', 'snack', 'dinner'];
String isoDay(DateTime day) =>
    '${day.year.toString().padLeft(4, '0')}-${day.month.toString().padLeft(2, '0')}-${day.day.toString().padLeft(2, '0')}';
DateTime civilDay(String value) {
  final parts = value.split('-').map(int.parse).toList();
  if (parts.length != 3) throw const ApiFailure('invalid_meal_plan');
  final day = DateTime.utc(parts[0], parts[1], parts[2]);
  if (isoDay(day) != value) throw const ApiFailure('invalid_meal_plan');
  return day;
}

DateTime addDays(DateTime day, int count) =>
    DateTime.utc(day.year, day.month, day.day + count);
DateTime weekStart(DateTime day) => addDays(day, 1 - day.weekday);
List<Json> eatingSlots(Json plan) => plan.containsKey('allocations')
    ? objects(plan['allocations'])
    : [
        {
          'date': plan['date'],
          'meal': plan['meal'],
          'servings': plan['servings'],
        },
      ];

class PlanningData {
  PlanningData(
    this.plan,
    this.profiles,
    this.recipes,
    this.catalogue,
    this.stock,
  );
  final Json plan, profiles, stock;
  final List<Json> recipes;
  final Catalogue catalogue;
  List<Json> get meals => objects(plan['meals']);
  List<Json> get people => objects(profiles['members']);
  List<Json> get prepared => objects(stock['prepared'] ?? []);
  static Future<PlanningData> load(HouseholdApi api) async {
    final values = await Future.wait([
      api.household('/mealplan'),
      api.household('/profiles'),
      api.recipes(),
      api.household('/ingredients'),
      api.household('/inventory?limit=500'),
    ]);
    return PlanningData(
      values[0],
      values[1],
      objects(values[2]['recipes']),
      Catalogue(values[3]),
      values[4],
    );
  }

  String person(Object? id) =>
      people.where((p) => p['id'] == id).firstOrNull?['name'] as String? ??
      id.toString();
  String title(Json reference) =>
      objects(plan['recipe_revisions'])
              .where(
                (r) =>
                    r['id'] == reference['id'] &&
                    r['revision'] == reference['revision'],
              )
              .firstOrNull?['title']
          as String? ??
      recipes.where((r) => r['id'] == reference['id']).firstOrNull?['title']
          as String? ??
      reference['id'] as String;
  List<String> attendees(String date, String meal) => people
      .where((p) {
        final schedule = object(
          plan['attendance'] ?? {'recurring': <Json>[], 'overrides': <Json>[]},
        );
        final override = objects(schedule['overrides'])
            .where((o) => o['date'] == date && o['meal'] == meal)
            .firstOrNull;
        final recurring = objects(schedule['recurring'])
            .where((r) => r['member_id'] == p['id'])
            .firstOrNull;
        final weekday = civilDay(date).weekday - 1;
        final allowed = override != null
            ? (override['member_ids'] as List).contains(p['id'])
            : recurring == null ||
                  (recurring['weekdays'] as List).contains(weekday) &&
                      (recurring['meals'] as List).contains(meal);
        final windows = objects(
          object(p['rules'] ?? <String, dynamic>{})['availability'] ?? [],
        );
        return allowed &&
            (windows.isEmpty ||
                windows.any(
                  (w) =>
                      w['meal'] == meal &&
                      (w['weekdays'] as List).contains(weekday),
                ));
      })
      .map((p) => p['id'] as String)
      .toList();
}
