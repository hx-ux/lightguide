import 'package:get/get.dart';
import 'package:lightguide/Mappings/mappings_note_scales.dart';
import 'package:lightguide/Controller/MainController.dart';
import 'package:lightguide/Views/MainView/main_view.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class ColorPicker extends StatefulWidget {
  const ColorPicker({
    super.key,
    required this.viewModel,
  });

  final MainViewController viewModel;

  @override
  State<ColorPicker> createState() => _ColorPickerState();
}

class _ColorPickerState extends State<ColorPicker> {
  @override
  Widget build(BuildContext context) {
    return Obx(() => Select<String>(
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
            widget.viewModel.setColor(value!);
          },
          placeholder: Container(
            width: 20,
            height: 20,
            color: mappedKeyColors[
                colorKeysName.isNotEmpty ? colorKeysName[0] : 'blue'],
          ),
          value: widget.viewModel.selectedColor.value,
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
        ));
  }
}
