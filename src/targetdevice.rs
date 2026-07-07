use crate::keycolor::KeyColor;

pub struct TargetDevice {
    pub name: String,
    pub keys: usize,
    pub report_size: usize,
    pub header: u8,
    pub vendorId: u16,
    pub deviceId: u16,
    pub activeColor: KeyColor,
    pub dimColor: KeyColor,
    pub responseColor: KeyColor,
}

impl TargetDevice {
    pub fn new(
        name: String,
        keys: usize,
        report_size: usize,
        header: u8,
        vendorId: u16,
        deviceId: u16,
    ) -> Self {
        Self {
            name,
            keys,
            report_size,
            header,
            vendorId,
            deviceId,
            activeColor: KeyColor { r: 0, g: 0, b: 255 },
            dimColor: KeyColor { r: 0, g: 0, b: 0 },
            responseColor: KeyColor { r: 255, g: 0, b: 0 },
        }
    }
}
