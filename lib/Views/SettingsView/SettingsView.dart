import 'package:lightguide/Models/app_settings.dart';
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
      height: AppSettings.height.toDouble()*0.9,
      alignment: Alignment.center,
      child: Column(
        mainAxisSize: MainAxisSize.max,
        children: [
          Text('Settings').h4(), 
        //  SettingsView()
        ],
      ),
    );
  }
}
