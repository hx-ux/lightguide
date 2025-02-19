import 'dart:io';
import 'dart:typed_data';
import 'package:lightguide/Mappings/mappings_note_scales.dart';
import 'package:lightguide/Models/color_converter.dart';
import 'package:lightguide/Models/controller_Properties.dart';
import 'package:lightguide/Models/pianokeys.dart';
import 'package:music_notes/music_notes.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class CollectionScale {
  final ControllerProperties controllerproperties;
  List<int> activeKeys = [];

  CollectionScale(this.controllerproperties);
  bool isControllerConnected = false;

  List<Pianokeys> toOctave() {
    List<Pianokeys> p = [];
    for (int i = 0; i <= 11; i++) {
      p.add(Pianokeys.singlePianoKey(
          position: i,
          noteName: mappedRootNotes[i]!,
          isActive: activeKeys.contains(i)));
    }
    return p;
  }

  Uint8List allKeyToBytes({Color colIn = Colors.blue}) {
    var temp = toOctave();

    int multiplyBy = (controllerproperties.keysCount / 11).ceil();
    for (int i = 0; i < multiplyBy; i++) {
      temp += temp;
    }

    var toSendBytes = BytesBuilder();
    for (int x = 0; x < temp.length; x++) {
      var col = temp[x].isActive
          ? ColorConverter.rgbToUint8(colIn)
          : ColorConverter.rgbToUint8(Colors.black);
      toSendBytes.add(col);
    }

    return toSendBytes.toBytes();
  }

  void setNotesByScale(List<Note> primaryList) {
    var cuttedScale = List.from(primaryList)..removeLast();
    List<int> d = [];
    for (var element in cuttedScale) {
      d.add(getIndexByNote(element));
    }
    activeKeys = d;
  }
}
