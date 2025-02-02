import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:lightguide/Models/CustomPianoWidget.dart';
import 'package:lightguide/Models/MusicalScale.dart';
import 'package:lightguide/Theme/Theme.dart';
import 'package:lightguide/ViewModel.dart';
import 'package:lightguide/Widgets/ConnectionBade.dart';
import 'package:lightguide/Widgets/welcome_screen_setup_guide.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

ThemeData? theme = themes['dark']!;

void main() {
  WidgetsFlutterBinding.ensureInitialized();
  SystemChrome.setSystemUIOverlayStyle(
      const SystemUiOverlayStyle(systemNavigationBarColor: Colors.transparent));
  SystemChrome.setEnabledSystemUIMode(SystemUiMode.edgeToEdge);
  runApp(const App());
}

class App extends StatelessWidget {
  const App({super.key});

  @override
  Widget build(BuildContext context) {
    return ShadcnApp(home: const MyHomePage(), theme: theme!);
  }
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key});

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
  @override
  Widget build(BuildContext context) {
    final viewModel = Get.put(ViewModel());

    return Scaffold(
      headers: [
        AppBar(
          leading: [
            const Text('Light Guide').h1(),
            const SizedBox(width: 10),
            Obx(() => connectionBadge(
                viewModel.foundDeviceAndEtablishedConnection.value)),
          ],
          trailing: [
            Obx(() => viewModel.foundDeviceAndEtablishedConnection.value
                ? PrimaryButton(
                    onPressed: viewModel.disconnectDevice,
                    trailing: const Icon(Icons.logout),
                    child: const Text('Disconnect'),
                  )
                : PrimaryButton(
                    onPressed: viewModel.connectDevice,
                    trailing: const Icon(Icons.login),
                    child: const Text('Connect'),
                  )),
          ],
        ),
      ],
      child: Obx(() => viewModel.foundDeviceAndEtablishedConnection.value
          ? Column(
              mainAxisSize: MainAxisSize.max,
              children: [
                Row(
                  mainAxisAlignment: MainAxisAlignment.center,
                  children: [
                    PrimaryButton(
                      child: const Text('Color Picker'),
                    ),

                    /// Root Note
                    Text("Root Note: "),
                    Select<String>(
                      itemBuilder: (context, item) {
                        return Text(item);
                      },
                      popupConstraints: const BoxConstraints(
                        maxHeight: 300,
                        maxWidth: 200,
                      ),
                      onChanged: (value) {
                        viewModel.setRootKey(value!);
                      },
                      placeholder: Text(mappedRootNotes[0].toString()),
                      value: viewModel.selectedRootNote.value,
                      children: [
                        SelectGroup(
                          children: [
                            for (var key in mappedRootNotes.keys) ...[
                              SelectItemButton(
                                value: mappedRootNotes[key].toString(),
                                child: Text(mappedRootNotes[key].toString()),
                              )
                            ]
                          ],
                        ),
                      ],
                    ),
                    Text("Scale"),
                    Select<String>(
                      itemBuilder: (context, item) {
                        return Text(item);
                      },
                      searchFilter: (item, query) {
                        return item.toLowerCase().contains(query.toLowerCase())
                            ? 1
                            : 0;
                      },
                      popupConstraints: const BoxConstraints(
                        maxHeight: 300,
                        maxWidth: 200,
                      ),
                      onChanged: (value) {
                        viewModel.setScale(value!);
                      },
                      placeholder: Text(GuiScaleNames[0].toString()),
                      value: viewModel.selectedScale.value,
                      children: [
                        SelectGroup(
                          children: [
                            for (var key in GuiScaleNames) ...[
                              SelectItemButton(
                                value: key,
                                child: Text(key),
                              )
                            ]
                          ],
                        ),
                      ],
                    ),
                  ],
                ).gap(20),
                SizedBox(height: MediaQuery.sizeOf(context).height * .4),
                Obx(() => CustomPianoKeyboard(
                    mappedNotes: viewModel.globalSammlung.value.toPianokeys())),
              ],
            )
          : const WelcomeScreenSetupGuide().center()),
    );
  }
}
