use std::time::Duration;

pub struct Ssm2 {
    port_name: String,
    serial_port: Option<Box<dyn serialport::SerialPort>>,
}

impl Ssm2 {
    const BAUD_RATE: u32 = 9600;

    pub fn new(port_name: &String) -> Self {
        Ssm2 { port_name: port_name.clone(), serial_port: None}
    }

    pub fn open(mut self) {
        // Connect to the serial port
        if self.serial_port.is_some() {
            println!("Serial port is already open.");
            return;
        }

        let port = serialport::new(&self.port_name, Ssm2::BAUD_RATE)
            .timeout(Duration::from_millis(100))
            .open()
            .unwrap();
        self.serial_port = Some(port);
    }

    pub fn close(mut self) {
        // Close the serial port
        drop(self.serial_port.unwrap());
        self.serial_port = None;
    }
}