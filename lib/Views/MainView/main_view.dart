import 'package:get/get.dart';
import 'package:get/get_core/src/get_main.dart';
import 'package:lightguide/Mappings/mappings_note_scales.dart';
import 'package:lightguide/Models/app_settings.dart';
import 'package:lightguide/Views/MainView/Widgets/custom_piano_widget.dart';
import 'package:lightguide/ViewModel.dart';
import 'package:lightguide/Views/MainView/Widgets/RootNotePicker.dart';
import 'package:lightguide/Views/MainView/Widgets/color_picker.dart';
import 'package:lightguide/Views/MainView/Widgets/scale_picker.dart';
import 'package:lightguide/Widgets/connection_bade.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class MainView extends StatelessWidget {
  const MainView({super.key});

  @override
  Widget build(BuildContext context) {
    final controller = Get.put(MainViewModel());
    return Scaffold(
        headers: [
          AppBar(
            leading: [
              const Text('Light Guide').h3(),
              const SizedBox(width: 10),
              Obx(() => connectionBadge(controller.connectedToDevice.value)),
            ],
            trailing: [
              IconButton(
                onPressed: () {},
                icon: const Icon(Icons.settings),
                variance: ButtonVariance.menu,
              ),
              Obx(() => controller.connectedToDevice.value
                  ? IconButton(
                      onPressed: controller.disconnectDevice,
                      icon: const Icon(Icons.logout),
                      variance: ButtonVariance.destructive,
                    )
                  : IconButton(
                      onPressed: controller.connectDevice,
                      icon: const Icon(Icons.login),
                      variance: ButtonVariance.primary,
                    )),
            ],
          ),
        ],
        child: Column(
          mainAxisSize: MainAxisSize.min,
          children: [
            Row(
              mainAxisAlignment: MainAxisAlignment.center,
              children: [
                const Text("Color ").withPadding(horizontal: 10),
                ColorPicker(viewModel: controller).withPadding(horizontal: 10),
                const Text("Root Note: ").withPadding(horizontal: 10),
                RootNotePicker(viewModel: controller)
                    .withPadding(horizontal: 10),
                const Text("Scale").withPadding(horizontal: 10),
                ScalePicker(
                  viewModel: controller,
                ).withPadding(horizontal: 10),
              ],
            ),
            SizedBox(height: MediaQuery.sizeOf(context).height * .2),
            CustomPianoKeyboard(
              controller: controller,
            ).paddingOnly(left: 50)
          ],
        ));
  }
}
