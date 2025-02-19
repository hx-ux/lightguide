import 'package:get/get_state_manager/src/rx_flutter/rx_obx_widget.dart';
import 'package:lightguide/Mappings/mappings_note_scales.dart';
import 'package:lightguide/ViewModel.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class ScalePicker extends StatefulWidget {
  final MainViewModel viewModel;

  const ScalePicker({Key? key, required this.viewModel}) : super(key: key);

  @override
  State<ScalePicker> createState() => _ScalePickerState();
}

class _ScalePickerState extends State<ScalePicker> {
  @override
  Widget build(BuildContext context) {
    return Obx(() {
      return Select<String>(
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
          widget.viewModel.setScale(value!);
        },
        placeholder: Text(guiScaleNames[0].toString()),
        value: widget.viewModel.selectedScale.value,
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
    });
  }
}
