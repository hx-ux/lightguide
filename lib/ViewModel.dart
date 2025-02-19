import 'dart:async';
import 'dart:typed_data';
import 'package:get/get.dart';
import 'package:lightguide/Alert/StandartAlertDialog.dart';
import 'package:lightguide/Models/hardware_device.dart';
import 'package:lightguide/Models/collection_scale.dart';
import 'package:lightguide/Models/controller_Properties.dart';
import 'package:lightguide/Mappings/mappings_note_scales.dart';
import 'package:lightguide/Models/pianokeys.dart';
import 'package:music_notes/music_notes.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class MainViewModel extends GetxController {
  var connectedToDevice = false.obs;
  var selectedDevice = 0.obs;
  var autoConnect = false.obs;
  late ControllerProperties controllerProperties;
  var collectionScale = CollectionScale(ControllerProperties.templates[0]).obs;
  late HardwareDevice hardwareDevice;
  var vmPianoKeys = <Pianokeys>[].obs;

  @override
  Future<void> onInit() async {
    super.onInit();

    controllerProperties = ControllerProperties.templates[selectedDevice.value];
    collectionScale.value = CollectionScale(controllerProperties);
    hardwareDevice = HardwareDevice(controllerProperties);
    if (autoConnect.value) {
      await connectDevice();
    }
    updateDeviceAndView();
  }

  var selectedRootNote = mappedRootNotes[0].toString().obs;
  RxString selectedScale = RxString(guiScaleNames[0]);
  var selectedColor = colorKeysName[0].obs;

  void setColor(String col) {
    selectedColor.value = col;
    updateDeviceAndView();
  }

  Future<void> connectDevice() async {
    try {
      hardwareDevice = HardwareDevice(controllerProperties);
      final result = await hardwareDevice?.connectDevice() ?? false;
      connectedToDevice.value = result;
    } catch (e) {
      connectedToDevice.value = false;
      print("Error connecting to device: $e");
      // showErrorDialog("Connection Error", "Failed to connect to the device. Please try again.");
    }
  }

  Future<bool> disconnectDevice() async {
    final r = await hardwareDevice?.disconnectDevice();
    connectedToDevice.value = !(r ?? false);
    return r ?? false;
  }

  void updateDeviceAndView() async {
    Note? rootNote =
        mappedRootNotes[getIndexByNote(Note.parse(selectedRootNote.value))];
    ScalePattern? scale = getScaleObjectByKey(selectedScale.value);

    if (rootNote != null && scale != null) {
      var scaled = scale.on(rootNote);

      update();
      if (scaled.degrees.isNotEmpty && scaled.degrees.isNotEmpty) {
        collectionScale.value.setNotesByScale(scaled.degrees);
        vmPianoKeys.value = collectionScale.value.intKeysToNotes();

        print("root note ${rootNote}");
        print("Scale ${scaled.toString()}");
        if (hardwareDevice != null) {
          await hardwareDevice!.sendFullReport(collectionScale.value
              .allKeyToBytes(colIn: mappedKeyColors[selectedColor.value]!));
        }
      }
    }
  }

  void setRootKey(String root) {
    selectedRootNote.value = root;
    updateDeviceAndView();
  }

  void setScale(String scale) {
    selectedScale.value = scale;
    updateDeviceAndView();
  }
}
