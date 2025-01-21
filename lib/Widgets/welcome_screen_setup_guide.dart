import 'package:shadcn_flutter/shadcn_flutter.dart';

class WelcomeScreenSetupGuide extends StatefulWidget {
  const WelcomeScreenSetupGuide({
    super.key,
  });

  @override
  State<WelcomeScreenSetupGuide> createState() =>
      _WelcomeScreenSetupGuideState();
}

class _WelcomeScreenSetupGuideState extends State<WelcomeScreenSetupGuide> {
  bool isAutoConnectChecked = false;
  CheckboxState _state = CheckboxState.unchecked;

  void toggleAutoConnect(bool? value) {
    setState(() {
      isAutoConnectChecked = value ?? false;
    });
  }

  @override
  Widget build(BuildContext context) {
    return Steps(
      children: [
        StepItem(
          title: const Text('Connect to the controller'),
          content: [
            const Text('Press the login button in the navbar'),
          ],
        ),
        StepItem(
          title: const Text('Auto Connection'),
          content: [
            Checkbox(
              state: _state,
              onChanged: (value) {
                setState(() {
                  // _state = value;
                });
              },
              trailing: const Text('Remember me'),
            )
          ],
        ),
      ],
    );
  }
}
