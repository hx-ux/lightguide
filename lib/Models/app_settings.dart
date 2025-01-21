import 'dart:io';
import 'package:path/path.dart' as p;

class AppSettings {
  static int width = 400;
  static int height = 400;

  static const String _settingsFolderName = "lightguide";
  static final String settingsFolderPath =
      '${Platform.environment['USERPROFILE']}\\Desktop\\$_settingsFolderName';

  static get logFilePath => p.join(settingsFolderPath, "log.txt");
  static get settingsfilePath => p.join(settingsFolderPath, "settings.yaml");
  static get favouritesFilePath => p.join(settingsFolderPath, "favourites.txt");

  static Future<void> createSettingsStructure() async {
    await Directory(settingsFolderPath).create();
    await File(logFilePath).create();
    await File(settingsfilePath).create();
    await File(favouritesFilePath).create();
  }
}
