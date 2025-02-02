import 'dart:io';
import 'package:path/path.dart' as p;

class AppSettings {
  static const String version="1.0.0"; 
  static int width = 800;
  static int height = 600;
  static const String _settingsFolderName = "lightguide";
  static final String settingsFolderPath = Platform.isWindows
      ? '${Platform.environment['USERPROFILE']}\\Desktop\\$_settingsFolderName'
      : '${Platform.environment['HOME']}/Desktop/$_settingsFolderName';
  
  static String get logFilePath => p.join(settingsFolderPath, "log.txt");
  static String get settingsfilePath => p.join(settingsFolderPath, "settings.yaml");
  static String get favouritesFilePath => p.join(settingsFolderPath, "favourites.txt");

  static Future<void> createSettingsStructure() async {
    await Directory(settingsFolderPath).create(recursive: true);
    await File(logFilePath).create();
    await File(settingsfilePath).create();
    await File(favouritesFilePath).create();
  }
}
