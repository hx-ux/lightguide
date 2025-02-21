import 'package:get/get.dart';
import 'package:lightguide/Models/controller_Properties.dart';
import 'package:lightguide/Controller/MainController.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class DeviceSelector extends StatefulWidget {
  var controller = Get.put(MainViewController());
  DeviceSelector({
    super.key,
  });

  @override
  State<DeviceSelector> createState() => _DeviceSelectorState();
}

class _DeviceSelectorState extends State<DeviceSelector> {
  @override
  Widget build(BuildContext context) {
    var selection =
        ControllerProperties.templates.map((e) => e.toInfoString()).toList();

    return IntrinsicWidth(
        child: Column(
      mainAxisSize: MainAxisSize.max,
      crossAxisAlignment: CrossAxisAlignment.stretch,
      children: [
        const Text('Select your Controller').marginOnly(bottom: 8),
        Column(
          crossAxisAlignment: CrossAxisAlignment.start,
          children: [
            RadioGroup<int>(
              value: widget.controller.selectedDevice.value,
              onChanged: (value) =>
                  widget.controller.selectedDevice.value = value,
              child: Column(
                crossAxisAlignment: CrossAxisAlignment.start,
                children: [
                  RadioItem(
                    value: 0,
                    trailing: Text(selection[0]),
                  ).paddingOnly(top: 8),
                ],
              ),
            ),
          ],
        ),
      ],
    ));
  }
}
