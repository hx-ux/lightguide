enum Testingstate { none, testing, working }
extension TestingstateExtension on Testingstate {
  String get info {
    switch (this) {
      case Testingstate.none:
        return "No testing in progress.";
      case Testingstate.testing:
        return "Currently in testing phase.";
      case Testingstate.working:
        return "working";
    }
  }
}

class ControllerProperties {
  final String name;
  final int vendorId;
  final int productId;
  final int keysCount;
  final int reportID;
  final Testingstate testingstate;
  ControllerProperties(
      {required this.reportID,
      required this.name,
      required this.vendorId,
      required this.productId,
      required this.keysCount,
      required this.testingstate});

  static final List<ControllerProperties> templates = [
    ControllerProperties(
        reportID: 0x82,
        name: "S49 MK1",
        vendorId: 0x17cc,
        productId: 0x1350,
        keysCount: 49,
        testingstate: Testingstate.working)
  ];

  toInfoString() {
    return " $name (${testingstate.info})";
  }
}
