import 'package:desktop_window/desktop_window.dart';
import 'package:flutter/services.dart';
import 'package:get/get.dart';
import 'package:lightguide/Models/custom_piano_widget.dart';
import 'package:lightguide/Mappings/mappings_note_scales.dart';
import 'package:lightguide/Models/app_settings.dart';
import 'package:lightguide/Theme/Theme.dart';
import 'package:lightguide/ViewModel.dart';
import 'package:lightguide/Widgets/connection_bade.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

ThemeData? theme = themes['dark']!;

Future<void> main() async {
  WidgetsFlutterBinding.ensureInitialized();
  await DesktopWindow.setWindowSize(
      Size(AppSettings.width.toDouble(), AppSettings.height.toDouble()));
  await DesktopWindow.setMinWindowSize(const Size(600, 600));
  await DesktopWindow.focus();
  await AppSettings.createSettingsStructure();

  runApp(const App());
}

class App extends StatelessWidget {
  const App({super.key});

  @override
  Widget build(BuildContext context) =>
      ShadcnApp(home: const MyHomePage(), theme: theme!);
}

class MyHomePage extends StatefulWidget {
  const MyHomePage({super.key});

  @override
  State<MyHomePage> createState() => _MyHomePageState();
}

class _MyHomePageState extends State<MyHomePage> {
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
              onPressed: controller.disconnectDevice,
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
      child: Obx(() => Column(
            mainAxisSize: MainAxisSize.min,
            children: [
              Row(
                mainAxisAlignment: MainAxisAlignment.center,
                children: [
                  const Text("Color ").withPadding(horizontal: 10),
                  colorPicker(controller).withPadding(horizontal: 10),
                  const Text("Root Note: ").withPadding(horizontal: 10),
                  rootNotePicker(controller).withPadding(horizontal: 10),
                  const Text("Scale").withPadding(horizontal: 10),
                  scalePicker(controller).withPadding(horizontal: 10),
                ],
              ),
              SizedBox(height: MediaQuery.sizeOf(context).height * .2),
              Obx(() => 
              
              CustomPianoKeyboard(
                    mappedNotes: controller.collectionScale.value.toOctave(),
                    keysColor: controller.selectedColorAsColor,
                  )).paddingOnly(left: 50)
            ],
          )),
    );
  }

  Select<String> colorPicker(MainViewModel viewModel) => Select<String>(
        autoClosePopover: true,
        orderSelectedFirst: false,
        itemBuilder: (context, item) {
          return Container(
            width: 20,
            height: 20,
            color: mappedKeyColors[item],
          );
        },
        popupConstraints: const BoxConstraints(
          maxHeight: 300,
          maxWidth: 10,
        ),
        onChanged: (value) {
          viewModel.setColor(value!);
        },
        placeholder: Container(
          width: 20,
          height: 20,
          color: mappedKeyColors[
              colorKeysName.isNotEmpty ? colorKeysName[0] : 'blue'],
        ),
        value: viewModel.selectedColor.value,
        children: [
          SelectGroup(
            children: [
              for (var col in mappedKeyColors.entries) ...[
                SelectItemButton(
                  value: col.key,
                  child: Container(
                    width: 20,
                    height: 20,
                    color: col.value,
                  ),
                )
              ]
            ],
          ),
        ],
      );

  Select<String> scalePicker(MainViewModel viewModel) => Select<String>(
        // important
        autoClosePopover: true,
        orderSelectedFirst: false,
        itemBuilder: (context, item) {
          return Text(item);
        },
        searchFilter: (item, query) {
          return item.toLowerCase().contains(query.toLowerCase()) ? 1 : 0;
        },
        popupConstraints: const BoxConstraints(
          maxHeight: 300,
          maxWidth: 200,
        ),
        onChanged: (value) {
          viewModel.setScale(value!);
        },
        placeholder: Text(guiScaleNames[0].toString()),
        value: viewModel.selectedScale.value,
        children: [
          SelectGroup(
            children: [
              for (var key in guiScaleNames) ...[
                SelectItemButton(
                  value: key,
                  child: Text(key),
                )
              ]
            ],
          ),
        ],
      );

  Select<String> rootNotePicker(MainViewModel viewModel) => Select<String>(
        autoClosePopover: true,
        orderSelectedFirst: false,
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
      );
}
