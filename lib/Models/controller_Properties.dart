enum Testingstate { none, testing, working }

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
    return "Name: $name TestingState: $testingstate";
  }
}
