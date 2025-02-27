import 'package:flutter/material.dart';
import 'package:get/get.dart';
import 'package:lightguide/Mappings/mappings_note_scales.dart';
import 'package:lightguide/Models/pianokeys.dart';
import 'package:lightguide/Controller/MainController.dart';

double singleKeywidth = 60;
double singleKeyHeight = 40;
double paddingbetweenKeys = 2;
double globalHeight = singleKeyHeight + 20;

class CustomPianoKeyboard extends StatefulWidget {
  final MainViewController controller;
  const CustomPianoKeyboard({super.key, required this.controller});

  @override
  CustomPianoKeyboardState createState() => CustomPianoKeyboardState();
}

class CustomPianoKeyboardState extends State<CustomPianoKeyboard> {
  final controller = Get.put(MainViewController());
  @override
  Widget build(BuildContext context) {
    var screenSize = MediaQuery.of(context).size;

    return SizedBox(
      child: Obx(() {
        return CustomPaint(
          painter: BaseKeyPainter(
              keys: controller.vmPianoKeys.value!,
              color: mappedKeyColors[controller.selectedColor.value]!),
          size: Size(screenSize.width, globalHeight),
        );
      }),
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

      TextPainter textPainter;
      textPainter = TextPainter(
        text: TextSpan(
          text: key.noteName.toString(),
          style: TextStyle(
            color: Colors.yellow,
            fontSize: 12,
          ),
        ),
        textDirection: TextDirection.ltr,
      );
      textPainter.layout(
        minWidth: 0,
        maxWidth: singleKeywidth,
      );
      textPainter.paint(
        canvas,
        Offset(currentPos - (singleKeywidth / 2), singleKeyHeight / 2),
      );
      

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

      TextPainter textPainter;
      textPainter = TextPainter(
      text: TextSpan(
        text: key.noteName.toString(),
        style: TextStyle(
        color: Colors.yellow,
        fontSize: 12,
        ),
      ),
      textDirection: TextDirection.ltr,
      );
      textPainter.layout(
      minWidth: 0,
      maxWidth: singleKeywidth * 0.5,
      );
      textPainter.paint(
      canvas,
      Offset(currentPos - (singleKeywidth * 0.25), (singleKeyHeight * 0.4) * -1),
      );

      if (black.length > 1 && black.elementAt(1) == key) {
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
  bool shouldRepaint(BaseKeyPainter oldDelegate) {
    return oldDelegate.color != color || oldDelegate.keys != keys;
  }
}
