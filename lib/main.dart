import 'package:desktop_window/desktop_window.dart';
import 'package:flutter/services.dart';
import 'package:lightguide/Models/app_settings.dart';
import 'package:lightguide/Theme/Theme.dart';
import 'package:lightguide/Views/MainView/main_view.dart';
import 'package:lightguide/Views/SettingsView/SettingsView.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

ThemeData? theme = themes['dark']!;

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await DesktopWindow.setWindowSize(
      Size(AppSettings.width.toDouble(), AppSettings.height.toDouble()));
  await DesktopWindow.setMinWindowSize(const Size(600, 600));
  await DesktopWindow.focus();
  await AppSettings.createSettingsStructure();

  runApp(const App());
}

class App extends StatelessWidget {
  const App({super.key});

  @override
  Widget build(BuildContext context) =>
      ShadcnApp(home: MainView(), theme: theme!);
}

