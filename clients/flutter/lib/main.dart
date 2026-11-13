import 'package:material_ui/material_ui.dart';
import 'package:image_picker/image_picker.dart';

import 'connection/nextcloud.dart';
import 'connection/vault.dart';
import 'shopping/controller.dart';
import 'ui/connect.dart';
import 'ui/home.dart';
import 'ui/messages.dart';

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  final controller = ShoppingController(Nextcloud(), DeviceVault());
  runApp(MealshopApp(controller));
  await controller.initialize(
    deviceLanguage:
        WidgetsBinding.instance.platformDispatcher.locale.languageCode,
  );
  // Consume abandoned gallery results without attaching them to a different account/draft.
  try {
    final lost = await ImagePicker().retrieveLostData();
    if (!lost.isEmpty) controller.photoInterrupted();
  } catch (_) {
    controller.photoInterrupted();
  }
}

class MealshopApp extends StatefulWidget {
  const MealshopApp(this.controller, {super.key});
  final ShoppingController controller;
  @override
  State<MealshopApp> createState() => _MealshopAppState();
}

class _MealshopAppState extends State<MealshopApp> with WidgetsBindingObserver {
  @override
  void initState() {
    super.initState();
    WidgetsBinding.instance.addObserver(this);
  }

  @override
  void dispose() {
    WidgetsBinding.instance.removeObserver(this);
    super.dispose();
  }

  @override
  void didChangeAppLifecycleState(AppLifecycleState state) =>
      widget.controller.foreground(state == AppLifecycleState.resumed);
  ThemeData theme(Brightness brightness) => ThemeData(
    colorScheme: ColorScheme.fromSeed(
      seedColor: const Color(0xff00679e),
      brightness: brightness,
    ),
    scaffoldBackgroundColor: brightness == Brightness.light
        ? const Color(0xfffafbfc)
        : const Color(0xff11171c),
    inputDecorationTheme: const InputDecorationTheme(
      border: OutlineInputBorder(),
    ),
    visualDensity: VisualDensity.standard,
  );
  @override
  Widget build(BuildContext context) => ListenableBuilder(
    listenable: widget.controller,
    builder: (context, _) {
      final c = widget.controller;
      return MaterialApp(
        // A household/account change disposes every open draft and detail route.
        key: ValueKey((
          c.credential?.server.root,
          c.credential?.user,
          c.selected,
        )),
        debugShowCheckedModeBanner: false,
        title: 'Mealshop',
        theme: theme(Brightness.light),
        darkTheme: theme(Brightness.dark),
        locale: Locale(c.language),
        supportedLocales: const [Locale('en'), Locale('de')],
        localizationsDelegates: GlobalMaterialLocalizations.delegates,
        home: !c.initialized
            ? const Scaffold(body: Center(child: CircularProgressIndicator()))
            : c.storageBlocked
            ? Scaffold(
                appBar: AppBar(title: const Text('Mealshop')),
                body: Padding(
                  padding: const EdgeInsets.all(24),
                  child: Column(
                    mainAxisAlignment: MainAxisAlignment.center,
                    children: [
                      Text(message('storage_failed', c.language)),
                      const SizedBox(height: 24),
                      FilledButton(
                        onPressed: () async {
                          await c.signOut();
                        },
                        child: Text(message('clear_local', c.language)),
                      ),
                    ],
                  ),
                ),
              )
            : c.credential == null
            ? ConnectScreen(c)
            : HomeScreen(c),
      );
    },
  );
}
