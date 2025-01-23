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
  @override
  Widget build(BuildContext context) {
    return const SizedBox(
      width: 300,
      child: Column(
        crossAxisAlignment: CrossAxisAlignment.stretch,
        children: [
          Text('Connect your Controller'),
          Divider(),
          Text('Select your Controller'),
        ],
      ),
    );
  }
}
