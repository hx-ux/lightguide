import 'dart:async';
import 'dart:typed_data';
import 'package:get/get.dart';
import 'package:lightguide/Alert/StandartAlertDialog.dart';
import 'package:lightguide/Models/hardware_device.dart';
import 'package:lightguide/Models/collection_scale.dart';
import 'package:lightguide/Models/controller_Properties.dart';
import 'package:lightguide/Mappings/mappings_note_scales.dart';
import 'package:music_notes/music_notes.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class MainViewModel extends GetxController {
  var connectedToDevice = false.obs;

  var selectedDevice = 0.obs;

  late ControllerProperties controller;
  var collectionScale = CollectionScale(ControllerProperties.templates[0]).obs;
  HardwareDevice? hardwareDevice;


  @override
  void onInit() {
    super.onInit();
    
    controller = ControllerProperties.templates[selectedDevice.value];
    collectionScale.value = CollectionScale(controller);
  }

  var selectedRootNote = mappedRootNotes[0].toString().obs;
  var selectedScale = guiScaleNames[0].toString().obs;

  var selectedColor = colorKeysName[0].obs;
  Color get selectedColorAsColor => stringtoColor(selectedColor.value);

  void setColor(String col) {
    selectedColor.value = col;
    applyToDeviceAndGui();
  }

  Color stringtoColor(String col) => mappedKeyColors[col] ?? Colors.blue;

  Future<void> connectDevice() async {
    hardwareDevice = HardwareDevice(controller);

    final result = await hardwareDevice?.connectDevice() ?? false;
    connectedToDevice.value = result;

    if (!connectedToDevice.value) {
      if (Get.context != null) {
        standartDialog(Get.context!, "Error", "Could not connect to device");
      } else {
        print("Could not connect to device");
      }
      return;
    }

    // init cmajor
    collectionScale.value
        .setNotesByScale(ScalePattern.major.on(Note.c).degrees);
  }

  Future<bool> disconnectDevice() async {
    final r = await hardwareDevice?.disconnectDevice();
    connectedToDevice.value = !(r ?? false);
    return r ?? false;
  }

  void applyToDeviceAndGui() async {
    Note? rootNote =
        mappedRootNotes[getIndexByNote(Note.parse(selectedRootNote.value))];
    ScalePattern? scale = getScaleObjectByKey(selectedScale.value);

    if (rootNote != null && scale != null) {
      var scaled = scale.on(rootNote);

      print("root note ${rootNote}");
      print("Scale ${scaled.toString()}");

      collectionScale.value.setNotesByScale(scaled.degrees);
      print(collectionScale.value.activeKeys);
      await hardwareDevice?.sendFullReport(
          collectionScale.value.allKeyToBytes(colIn: selectedColorAsColor));
    }
  }

  void setRootKey(String root) {
    selectedRootNote.value = root;
    applyToDeviceAndGui();
  }

  void setScale(String scale) {
    selectedScale.value = scale;
    applyToDeviceAndGui();
  }
}
