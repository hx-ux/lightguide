import 'package:get/get.dart';
import 'package:lightguide/Mappings/mappings_note_scales.dart';
import 'package:lightguide/ViewModel.dart';
import 'package:shadcn_flutter/shadcn_flutter.dart';

class RootNotePicker extends StatefulWidget {
  const RootNotePicker({super.key, required this.viewModel});

  final MainViewModel viewModel;
  @override
  State<RootNotePicker> createState() => _RootNotePickerState();
}

class _RootNotePickerState extends State<RootNotePicker> {
  @override
  Widget build(BuildContext context) {
    return Obx(() => Select<String>(
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
            widget.viewModel.setRootKey(value!);
          },
          placeholder: Text(mappedRootNotes[0].toString()),
          value: widget.viewModel.selectedRootNote.value,
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
        ));
  }
}
