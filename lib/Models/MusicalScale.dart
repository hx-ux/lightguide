import 'dart:convert';
import 'dart:io';
import 'dart:typed_data';

import 'package:lightguide/Models/ControllerProperties.dart';
import 'package:lightguide/Models/app_settings.dart';
import 'package:lightguide/Models/favourites.dart';
import 'package:music_notes/music_notes.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class Pianokeys {
  final int position;
  final Note noteName;
  final bool isActive;

  bool isFlat() => _isKeyFlat(noteName);

  Pianokeys.singlePianoKey(
      {required this.position, required this.noteName, required this.isActive});
}

class CollectionScale {
  final ControllerProperties controllerproperties;
  List<int> activeKeys = [];

  CollectionScale(this.controllerproperties);
  bool isControllerConnected=false;

  List<Pianokeys> toPianokeys() {
    List<Pianokeys> p = [];
    for (int i = 0; i <= 11; i++) {
      p.add(Pianokeys.singlePianoKey(
          position: i,
          noteName: mappedRootNotes[i]!,
          isActive: activeKeys.contains(i)));
    }
    return p;
  }

  Uint8List activekeysToBytes({Color colIn = Colors.blue}) {
    var temp = toPianokeys();
    temp += temp;
    temp += temp;

    var toSendBytes = BytesBuilder();
    for (int x = 0; x < temp.length; x++) {
      var col = temp[x].isActive ? rgbToUint8(colIn) : rgbToUint8(Colors.black);

      toSendBytes.add(col);
    }
    print(toSendBytes);

    return toSendBytes.toBytes();
  }
}

Uint8List rgbToUint8(Color selcol) {
  int red = (selcol.red >> 16) & 0xFF;
  int green = (selcol.green >> 8) & 0xFF;
  int blue = selcol.blue & 0xFF;
  return Uint8List.fromList([red, green, blue]);
}

Map<int, Note> mappedRootNotes = {
  0: Note.c,
  1: const Note(BaseNote.c, Accidental.sharp),
  2: Note.d,
  3: const Note(BaseNote.d, Accidental.sharp),
  4: Note.e,
  5: Note.f,
  6: const Note(BaseNote.f, Accidental.sharp),
  7: Note.g,
  8: const Note(BaseNote.g, Accidental.sharp),
  9: Note.a,
  10: const Note(BaseNote.a, Accidental.sharp),
  11: Note.b,
};

Map<String, ScalePattern> mappedScales = {
  "major": ScalePattern.major,
  "minor": ScalePattern.naturalMinor,
  "harmonic minor": ScalePattern.harmonicMinor,
  "melodic minor": ScalePattern.melodicMinor,
  "dorian": ScalePattern.dorian,
  "phrygian": ScalePattern.phrygian,
  "lydian": ScalePattern.lydian,
  "mixolydian": ScalePattern.mixolydian,
  "locrian": ScalePattern.locrian,
  "whole tone": ScalePattern.wholeTone,
  "chromatic": ScalePattern.chromatic,
  "major pentatonic": ScalePattern.majorPentatonic,
};

List<String> guiScaleNames = mappedScales.keys.toList();


Map<String, Color> mappedKeyColors = {
  "red": Colors.red,
  "orange": Colors.orange,
  "yellow": Colors.yellow,
  "green": Colors.green,
  "blue": Colors.blue,
  "pink": Colors.pink,
};

List<String> ColorKeysName = mappedKeyColors.keys.toList();


String? getScaleNamebyObject(ScalePattern p) => mappedScales.keys.firstWhere((k) => mappedScales[k] == p);

ScalePattern? getScaleObjectByKey(String p) => mappedScales.containsKey(p) ? mappedScales[p] : null;



Future<List<Favourites>> getFavourites() async {
  File f = File(AppSettings.favouritesFilePath);
  if (!f.existsSync()) return [];

  final p = await f.readAsString();
  var splitted = LineSplitter().convert(p);
  return splitted.map((element) => Favourites.fromFile(element)).toList();
}



int getIndexByNote(Note note) {
  for (var entry in mappedRootNotes.entries) {
    if (entry.value == note) {
      return entry.key;
    }
  }
  return 0;
}

bool _isKeyFlat(Note noteName) {
  int index = getIndexByNote(noteName);
  switch (index) {
    case 1:
    case 3:
    case 6:
    case 8:
    case 10:
      return true;
    default:
      return false;
  }
}
