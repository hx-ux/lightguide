import 'package:shadcn_flutter/shadcn_flutter.dart';

Widget connectionBadge(bool isConnected) {
  return isConnected
      ? const PrimaryBadge(child: Text('Connected'))
      : const DestructiveBadge(
          child: Text('Not Connected'),
        );
}
