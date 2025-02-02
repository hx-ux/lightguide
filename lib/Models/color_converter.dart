import 'dart:typed_data';
import 'dart:ui';

class ColorConverter {
  static Uint8List rgbToUint8(Color selcol) {
    int red = selcol.red & 0xFF;
    int green = selcol.green & 0xFF;
    int blue = selcol.blue & 0xFF;

    return Uint8List.fromList([red, green, blue]);
  }
}
