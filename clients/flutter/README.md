# Mealshop for Android

Native Flutter client for Nextcloud households, recipes, personal activities and shared shopping. See the repository
[mobile guide](../../docs/guides/mobile.md), [F35 contract](../../specs/0035-flutter-mobile-client/specification.md)
and [toolchain pin](toolchain.json).

Use `python3 scripts/mobile.py bootstrap`, `emulator`, `build` and `install` from the repository root.
Shopping, Recipes and More are active destinations. P2 adds people/rules, online ingredients,
collections/sharing and optional person activities. P3–P5 remain tracked in the feature plan.
Instrumentation runs as `org.mealshop.pocket.verification`; the Flutter driver can uninstall that
package without touching the development app or its local account data.
No server URL, user password or ExApp secret is built into the application.
