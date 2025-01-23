import 'dart:typed_data';
import 'package:get/get.dart';
import 'package:hid4flutter/hid4flutter.dart';
import 'package:lightguide/Models/ControllerProperties.dart';
import 'package:lightguide/Models/MusicalScale.dart';
import 'package:music_notes/music_notes.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class ViewModel extends GetxController {
  var foundDeviceAndEtablishedConnection = false.obs;
  var devices = <HidDevice>[].obs;
  var globalSammlung = CollectionScale(ControllerProperties.templates[0]).obs;
  ControllerProperties controller = ControllerProperties.templates[0];

  var selectedRootNote = mappedRootNotes[0].toString().obs;
  var selectedScale = guiScaleNames[0].toString().obs;

  String selectedColor = ColorKeysName[0];
  late final HidDevice device;

  void setColor(String col) {
    selectedColor = col;
    
    print(col); 
    globalSammlung.value.activekeysToBytes(colIn: mappedKeyColors[col] ?? Colors.blue);
  }

  Future<void> disconnectDevice() async {
    if (device.isOpen) {
      device.close();
      foundDeviceAndEtablishedConnection.value = false;
    }
  }

  List<int> scaleToNotesPostiions(List<Note> primaryList) {
    var cuttedScale = List.from(primaryList)..removeLast();
    List<int> d = [];
    for (var element in cuttedScale) {
      d.add(getIndexByNote(element));
    }
    return d;
  }

  void applySammlungToDeviceAndGui() async {
    Note? rootNote =
        mappedRootNotes[getIndexByNote(Note.parse(selectedRootNote.value))];

    ScalePattern? scale = getScaleObjectByKey(selectedScale.value);

    if (rootNote != null && scale != null) {
      var s = scale.on(rootNote);

      print("root note ${rootNote}");
      print("Scale ${s.toString()}");

      globalSammlung.value.activeKeys = scaleToNotesPostiions(s.degrees);
      await sendNewColorReport(globalSammlung.value.activekeysToBytes());
    }
  }

  Future<bool> sendNewColorReport(Uint8List buffer) async {
    if (device.isOpen) {
      await device.sendReport(reportId: controller.reportID, buffer);
      print("send data to device");
      return true;
    }
    print("send data error");
    return false;
  }

  void connectDevice() async {
    final List<HidDevice> founddevices = await Hid.getDevices(
        vendorId: controller.vendorId, productId: controller.productId);
    device = founddevices.first;

    try {
      await device.open();
      foundDeviceAndEtablishedConnection.value = true;
      update();
      Uint8List bufferA = Uint8List(0);
      await device.sendReport(reportId: 0xa0, bufferA);

      // init cmajor
      globalSammlung.value.activeKeys =
          scaleToNotesPostiions(ScalePattern.major.on(Note.c).degrees);
      await sendNewColorReport(globalSammlung.value.activekeysToBytes());
    } finally {}
  }

  void setRootKey(String root) {
    selectedRootNote.value = root;
    applySammlungToDeviceAndGui();
  }

  void setScale(String scale) {
    selectedScale.value = scale;
    applySammlungToDeviceAndGui();
  }
}
