import 'dart:typed_data';

import 'package:hid4flutter/hid4flutter.dart';
import 'package:lightguide/Models/controller_Properties.dart';

class HardwareDevice {
  HidDevice? device;
  final ControllerProperties controller;
  HardwareDevice(this.controller);

  get isOpen {
    if (device is HidDevice) {
      return device!.isOpen;
    }
    return false;
  }

  Future<bool> disconnectDevice() async {
    if (device == null) return false;

    if (device!.isOpen) {
      try {
        await device!.close().timeout(const Duration(seconds: 3));
      } catch (e) {
        return false;
      }
    }
    return device?.isOpen ?? false;
  }

  Future<bool> sendFullReport(Uint8List buffer) async {
    if (device != null && device!.isOpen) {
      await device!.sendReport(reportId: controller.reportID, buffer);
      return true;
    }
    return false;
  }

  Future<bool> connectDevice() async {
    final List<HidDevice> foundDevices = await Hid.getDevices(
        vendorId: controller.vendorId, productId: controller.productId);
    if (foundDevices.isEmpty) return false;
    device = foundDevices.first;
    try {
      await device!.open();
      Uint8List bufferA = Uint8List(0);
      await device!.sendReport(reportId: 0xa0, bufferA);

      return device!.isOpen;
    } catch (e) {
      device = null;
      return false;
    }
  }
}
