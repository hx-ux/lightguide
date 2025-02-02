import 'dart:typed_data';

import 'package:hid4flutter/hid4flutter.dart';
import 'package:lightguide/Models/controller_Properties.dart';

class HardwareDevice {
  late HidDevice device;
  final ControllerProperties controller;
  HardwareDevice(this.controller);

  Future<bool> disconnectDevice() async {
    if (device.isOpen) {
      await device.close();
    }
    return device.isOpen;
  }

  Future<bool> sendFullReport(Uint8List buffer) async {
    if (device.isOpen) {
      await device.sendReport(reportId: controller.reportID, buffer);
      return true;
    }
    return false;
  }

  Future<bool> connectDevice() async {
    final List<HidDevice> founddevices = await Hid.getDevices(
        vendorId: controller.vendorId, productId: controller.productId);
    if (founddevices.isEmpty) return false;
    device = founddevices.first;
    try {
      await device.open();
      Uint8List bufferA = Uint8List(0);
      await device.sendReport(reportId: 0xa0, bufferA);

      return device.isOpen;
    } finally {}
  }
}
