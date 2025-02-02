import 'dart:convert';
import 'dart:io';

import 'package:lightguide/Models/app_settings.dart';
import 'package:music_notes/music_notes.dart';

class Favourites {
  String? displayName;
  Note? note;
  String? scale;

  Favourites({
    required this.displayName,
    required this.note,
    required this.scale,
  });

  Favourites.fromFile(String fromFile) {
    try {
      final values = fromFile.split('|');
      if (values.length != 3) {
        throw FormatException("Invalid format");
      }
      note = Note.parse(values[0]);
      scale = values[1];
      displayName = values[2];
    } catch (e) {
      print("Error parsing Favourites: $e");
    }
  }
  @override
  String toString() => "${note.toString()}|$scale|$displayName";
  String get toFile => "${toString()}\r\n";

  void saveAsFavourite(Favourites f) {
    File(AppSettings.favouritesFilePath)
        .writeAsString(f.toFile, mode: FileMode.append);
    Future<List<Favourites>> getAllFavourites() async {
      File f = File(AppSettings.favouritesFilePath);

      if (!f.existsSync()) return [];

      List<Favourites> favouritesList = [];
      await f
          .openRead()
          .transform(utf8.decoder)
          .transform(const LineSplitter())
          .forEach((line) {
        favouritesList.add(Favourites.fromFile(line));
      });
      return favouritesList;
    }
  }
}
