import 'dart:io';

import 'package:lightguide/Models/app_settings.dart';
import 'package:music_notes/music_notes.dart';

class Favourites {
  String? displayName;
  Note? note;
  String? scale;

  Favourites(
      {required this.displayName, required this.note, required this.scale});

  Favourites.fromFile(String fromFile) {
    final values = fromFile.split('|');
    note = Note.parse(values[0]);
    scale = values[1];
    displayName = values[2];
  }

  @override
  String toString() => "${note.toString()}|$scale|$displayName";
  String get toFile => "${toString()}\r\n";

  void saveAsFavourite(Favourites f) {
    if (!File(AppSettings.favouritesFilePath).existsSync()) {
      File(AppSettings.favouritesFilePath)
          .writeAsString(f.toFile, mode: FileMode.append);
    }
  }
}
