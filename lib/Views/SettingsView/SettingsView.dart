import 'package:get/get.dart';
import 'package:get/get_core/src/get_main.dart';
import 'package:lightguide/Models/app_settings.dart';
import 'package:lightguide/Views/SettingsView/Widgets/deviceSelector.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class SettingsView extends StatefulWidget {
  const SettingsView({
    super.key,
  });

  @override
  State<SettingsView> createState() => _SettingsViewState();
}

class _SettingsViewState extends State<SettingsView> {
  @override
  Widget build(BuildContext context) {
    return Container(
      width: 400,
      height: AppSettings.height.toDouble() * 0.9,
      alignment: Alignment.center,
      child: Column(
        mainAxisSize: MainAxisSize.max,
        children: [
          Row(
            children: [
              IconButton(
                  icon: Icon(Icons.exit_to_app),
                  variance: ButtonVariance.destructive,
                  onPressed: () => Get.back()),
              Text('Settings').h4(),
            ],
          ),
          DeviceSelector(),
          //  SettingsView()
        ],
      ),
    );
  }
}
