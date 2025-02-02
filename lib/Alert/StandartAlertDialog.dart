import 'package:shadcn_flutter/shadcn_flutter.dart';

void standartDialog(BuildContext? context, String title, String content,
    {bool canCancel = false}) {
  if (context == null) return;
  showDialog(
    context: context,
    builder: (context) {
      return AlertDialog(
        title: Text(title),
        content: Text(content),
        actions: [
          if (canCancel)
            OutlineButton(
              child: const Text('Cancel'),
              onPressed: () {
                Navigator.pop(context);
              },
            ),
          PrimaryButton(
            child: const Text('OK'),
            onPressed: () {
              Navigator.pop(context);
            },
          ),
        ],
      );
    },
  );
}
