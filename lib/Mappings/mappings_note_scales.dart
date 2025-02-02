import 'package:music_notes/music_notes.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

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

final guiScaleNames = mappedScales.keys.toList();

Map<String, Color> mappedKeyColors = {
  "red": const Color.fromARGB(255, 255, 0, 0),
  "orange": const Color.fromARGB(255, 255, 145, 0),
  "yellow": const Color.fromARGB(255, 255, 230, 0),
  "green": const Color.fromARGB(255, 0, 255, 0),
  "blue": const Color.fromARGB(255, 0, 0, 255),
  "pink": const Color.fromARGB(255, 255, 0, 191),
};

final colorKeysName = mappedKeyColors.keys.toList();

String? getScaleNamebyObject(ScalePattern p) =>
    mappedScales.keys.firstWhere((k) => mappedScales[k] == p);

ScalePattern? getScaleObjectByKey(String p) =>
    mappedScales.containsKey(p) ? mappedScales[p] : null;

Map<Note, int> reverseMappedRootNotes = {
  for (var entry in mappedRootNotes.entries) entry.value: entry.key
};

int getIndexByNote(Note note) => reverseMappedRootNotes[note] ?? 0;
