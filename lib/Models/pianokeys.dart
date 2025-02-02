import 'package:lightguide/Mappings/mappings_note_scales.dart';
import 'package:music_notes/music_notes.dart';

class Pianokeys {
  final int position;
  final Note noteName;
  final bool isActive;

  bool isFlat() => _isKeyFlat(noteName);

  Pianokeys.singlePianoKey(
      {required this.position, required this.noteName, required this.isActive});

  bool _isKeyFlat(Note noteName) => [1, 3, 6, 8, 10].contains(getIndexByNote(noteName));
}
