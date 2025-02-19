import 'package:get/get.dart';
import 'package:lightguide/Models/controller_Properties.dart';
import 'package:lightguide/ViewModel.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class DeviceSelector extends StatefulWidget {
  MainViewModel get viewModel => Get.find();
  const DeviceSelector({
    super.key,
  });

  @override
  State<DeviceSelector> createState() =>
      _DeviceSelectorState();
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
              value: widget.viewModel.selectedDevice.value,
              onChanged: (value) =>
                  widget.viewModel.selectedDevice.value = value,
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
        // AutoConnectOnStartCheckbox(widget: widget).paddingOnly(top: 20),
      ],
    ));
  }
}
