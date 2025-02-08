import 'package:flutter/material.dart';
import 'package:lightguide/Models/pianokeys.dart';

double singleKeywidth = 60;
double singleKeyHeight = 40;
double paddingbetweenKeys = 2;
double globalHeight = singleKeyHeight + 20;

class CustomPianoKeyboard extends StatefulWidget {
  final List<Pianokeys> mappedNotes;
  final Color keysColor;

  const CustomPianoKeyboard(
      {super.key, required this.mappedNotes, required this.keysColor});

  @override
  _CustomPianoKeyboardState createState() => _CustomPianoKeyboardState();
}

class _CustomPianoKeyboardState extends State<CustomPianoKeyboard> {
  @override
  Widget build(BuildContext context) {
    var screenSize = MediaQuery.of(context).size;

    return SizedBox(
      child: CustomPaint(
        painter:
            BaseKeyPainter(keys: widget.mappedNotes, color: widget.keysColor),
        size: Size(screenSize.width, globalHeight),
      ),
    );
  }
}

class BaseKeyPainter extends CustomPainter {
  final List<Pianokeys> keys;
  final Color color;
  BaseKeyPainter({
    required this.color,
    required this.keys,
  });
  @override
  void paint(Canvas canvas, Size size) {
    var black = keys.where((element) => element.isFlat()).toList();
    var white = keys.where((element) => !element.isFlat()).toList();

    double currentPos = 0;

    for (var key in white) {
      canvas.drawRect(
          Rect.fromCenter(
              center: Offset(currentPos, 0),
              width: singleKeywidth,
              height: singleKeyHeight),
          Paint()..color = Colors.white);

      if (key.isActive) {
        canvas.drawRect(
            Rect.fromCenter(
                center: Offset(currentPos, 0),
                width: singleKeywidth * 0.9,
                height: singleKeyHeight * 0.2),
            Paint()..color = color);
      }

      currentPos = currentPos + singleKeywidth + paddingbetweenKeys;
    }

    currentPos = singleKeywidth * 0.5 + paddingbetweenKeys;

    for (var key in black) {
      canvas.drawRect(
          Rect.fromCenter(
              center: Offset(currentPos, (singleKeyHeight / 8) * -1),
              width: singleKeywidth * 0.5,
              height: singleKeyHeight * 0.8),
          Paint()..color = Colors.grey);

      if (black.elementAt(1) == key) {
        currentPos = currentPos + (singleKeywidth * 0.5) * 2;
      }
      currentPos =
          currentPos + ((singleKeywidth * 0.5) * 2) + paddingbetweenKeys;

      if (key.isActive) {
        canvas.drawRect(
            Rect.fromCenter(
                center: Offset(currentPos, 0),
                width: singleKeywidth * 0.4,
                height: singleKeyHeight * 0.8),
            Paint()..color = color);
      }
    }
  }

  @override
  bool shouldRepaint(CustomPainter oldDelegate) {
    return true;
  }
}
