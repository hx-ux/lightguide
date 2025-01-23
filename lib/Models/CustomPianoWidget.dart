import 'package:flutter/material.dart';
import 'package:lightguide/Models/MusicalScale.dart';

class CustomPianoKeyboard extends StatefulWidget {
  final List<Pianokeys> mappedNotes;

  const CustomPianoKeyboard({super.key, required this.mappedNotes});

  @override
  _CustomPianoKeyboardState createState() => _CustomPianoKeyboardState();
}

class _CustomPianoKeyboardState extends State<CustomPianoKeyboard> {
  @override
  Widget build(BuildContext context) {
    var screenSize = MediaQuery.of(context).size;
  
    double singlekeyWidth = screenSize.width/ 13;
    double globalHeight =  screenSize.height*0.1;

    return Column(
      children: [
        Row(
          mainAxisAlignment: MainAxisAlignment.center,
          children: [
            for (var key in widget.mappedNotes)
              Padding(
                padding: const EdgeInsets.all(0.5),
                child: Column(
                  children: [
                    CustomPaint(
                      painter: PianoKeyPainter(
                          isActive: key.isActive,
                          isFlat: key.isFlat(),
                          Colors.blue),
                      size: Size(singlekeyWidth, globalHeight),
                    ),
                    Text(key.noteName.toString()),
                  ],
                ),
              ),
          ],
        )
      ],
    );
  }
}

class PianoKeyPainter extends CustomPainter {
  final bool isActive;
  final bool isFlat;
  final Color isActivecolor;
  PianoKeyPainter(this.isActivecolor,
      {required this.isActive, required this.isFlat});
  @override
  void paint(Canvas canvas, Size size) {
    Offset basePosition = Offset(size.width / 2, size.height / 2);

    Offset basePositionShifted = Offset(size.width / 2, size.height / 4);

    Rect base = Rect.fromCenter(
        center: !isFlat ? basePosition : basePositionShifted,
        width: size.width,
        height: size.height);
    canvas.drawRect(base, Paint()..color = isFlat ? Colors.grey : Colors.white);

    Rect indictaor = Rect.fromCenter(
        center: basePosition, width: size.width, height: size.height / 4);
    canvas.drawRect(base, Paint()..color = isFlat ? Colors.grey : Colors.white);

    canvas.drawRect(indictaor,
        Paint()..color = isActive ? isActivecolor : Colors.transparent);
  }

  @override
  bool shouldRepaint(CustomPainter oldDelegate) {
    return true;
  }
}
