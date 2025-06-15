use std::time::Duration;

use crate::vprintln;

const BAUD_RATE: u32 = 4800;
// 0x80 as header byte
// 0x10 as destination (ECU) and 0xF0 as source (program)
const HEADER_BYTE: u8 = 0x80;
const ECU_BYTE: u8 = 0x10;
const TOOL_BYTE: u8 = 0xF0;

// TODO consider creating packet struct
const DST_I: usize = 1;
const SRC_I: usize = 2;
const NUM_I: usize = 3;

pub enum EcuParam {
    EngineSpeed,
    EngineLoad,
    IntakeTemp,
    ThrottleAngle,
    AirFuelSensor1,
}

impl EcuParam {
    // Where the params are located in the ECU memory
    fn addr(&self) -> (u32, Option<u32>) {
        match self {
            EcuParam::EngineSpeed => (0x00000E, Some(0x00000F)),
            EcuParam::EngineLoad => (0x000007, None),
            EcuParam::IntakeTemp => (0x000012, None),
            EcuParam::ThrottleAngle => (0x000015, None),
            EcuParam::AirFuelSensor1 => (0x000042, None),
        }
    }

    // Where the param value is found in the response data
    fn mask(&self) -> (usize, Option<usize>) {
        match self {
            EcuParam::EngineSpeed => (4, Some(5)),
            EcuParam::EngineLoad => (5, None),
            EcuParam::IntakeTemp => (5, None),
            EcuParam::ThrottleAngle => (5, None),
            EcuParam::AirFuelSensor1 => (5, None),
        }
    }
}

pub struct Ssm2 {
    port_name: String,
    serial_port: Option<Box<dyn serialport::SerialPort>>,
}

impl Ssm2 {
    pub fn new(port_name: &String) -> Self {
        Ssm2 {
            port_name: port_name.clone(),
            serial_port: None,
        }
    }

    pub fn open(&mut self) {
        // Connect to the serial port
        if self.serial_port.is_some() {
            println!("Serial port is already open.");
            return;
        }

        let port = serialport::new(&self.port_name, BAUD_RATE)
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

    pub fn ecu_init(&mut self, buf: &mut Vec<u8>) {
        self.send_read_packet(&[0xBF], buf);
    }

    pub fn ecu_read(&mut self, value_type: EcuParam, buf: &mut Vec<u8>) {
        let (addr1, addr2) = value_type.addr();
        let a1 = ((addr1 & 0xFF0000) >> 16) as u8;
        let a2 = ((addr1 & 0x00FF00) >> 8) as u8;
        let a3 = (addr1 & 0x0000FF) as u8;

        match addr2 {
            // Single byte read
            None => {
                self.send_read_packet(&[0xA8, 0x00, a1, a2, a3], buf);
            }
            // Block read
            Some(addr2) => {
                self.send_read_packet(&[0xA0, 0x00, a1, a2, a3, (addr2 - addr1) as u8], buf);
            }
        }
    }

    // Sends packet to ECU by appending the packet header 0x80 0x10 0xF0
    // followed by the data length, the data bytes themselves, and a checksum byte.
    // Assume data.len() <= 255
    fn send_packet(&mut self, data: &[u8]) {
        match data.first().unwrap() {
            0xB0 | 0xB8 => {
                println!(
                    "Warning! Attempting to send packet with write instructions, aborting instruction"
                );
                return;
            }
            _ => {
                // no-op
            }
        }

        let mut packet: Vec<u8> = vec![
            HEADER_BYTE,
            ECU_BYTE,  // To ECU
            TOOL_BYTE, // From tool
        ];
        packet.push(data.len() as u8); // Len byte
        data.iter().for_each(|&byte| packet.push(byte)); // Data bytes
        packet.push(Ssm2::calculate_checksum(&packet)); // Checksum byte

        vprintln!("Sending packet: {:02X?}", packet);
        self.serial_port
            .as_mut()
            .unwrap()
            .write_all(&packet)
            .unwrap();
    }

    fn read_packet(&mut self, response: &mut Vec<u8>) {
        response.clear();

        let mut buffer: Vec<u8> = Vec::new();

        let port = self.serial_port.as_mut().unwrap();

        // TODO Temporary hard code until a better check for invalid case is found
        for _ in 0..20 {
            let mut buf = [0; 64];
            let n = port.read(&mut buf).unwrap_or_default();
            if n == usize::default() {
                break;
            }
            vprintln!("Received {} bytes: {:02X?}", n, &buf[..n]);

            buf[..n].iter().for_each(|&byte| buffer.push(byte));
        }

        for (index, &byte) in buffer.iter().enumerate() {
            if byte == HEADER_BYTE {
                let length = Ssm2::check_packet(&buffer[index..]);
                if length != 0 {
                    vprintln!(
                        "Valid packet found: {:02X?}",
                        &buffer[index..index + length]
                    );
                    *response = buffer[index..index + length].to_vec();
                    return;
                }
            }
        }
        response.clear();
    }

    // Checks if the start of the byte array is a valid incoming packet from ECU
    // If valid, returns packet length
    // Otherwise 0 if invalid
    fn check_packet(packet: &[u8]) -> usize {
        if packet.len() < 5 {
            // At least HDR, DST, SRC, NUM, CHK
            vprintln!("Invalid packet! Length {} is less than 5", packet.len());
            return 0;
        }

        let dst_byte = packet[DST_I];
        let src_byte = packet[SRC_I];

        if dst_byte != TOOL_BYTE || src_byte != ECU_BYTE {
            vprintln!(
                "Invalid packet! DST/SRC not matched: actual DST {:02X} SRC {:02X}",
                dst_byte,
                src_byte
            );
            return 0;
        }

        let packet_len = packet[NUM_I] as usize + 5;

        if packet_len > packet.len() {
            vprintln!(
                "Invalid packet! Expected packet length {} is greater than actual {}",
                packet_len,
                packet.len()
            );
            return 0;
        }

        // TODO better way to sum and ignore overflow?
        let actual_sum = Ssm2::calculate_checksum(&packet[..packet_len - 1]);
        // TODO better way to get last element?
        let expect_sum = packet[packet_len - 1];

        if actual_sum != expect_sum {
            vprintln!(
                "Invalid packet! Checksum failed: {:02X}, {:02X}",
                actual_sum,
                expect_sum
            );
            return 0;
        }

        return packet_len;
    }

    // Sends packet with provided data bytes and immediately reads the response packet from the ECU.
    fn send_read_packet(&mut self, data: &[u8], buf: &mut Vec<u8>) {
        self.send_packet(data);
        self.read_packet(buf);
    }

    fn calculate_checksum(packet: &[u8]) -> u8 {
        (packet.iter().fold(0u64, |sum, &val| sum + val as u64) & 0xFF) as u8
    }
}

#[cfg(test)]
mod ssm2_tests {
    use super::*;

    // calculate_checksum
    #[test]
    fn test_calculate_checksum() {
        let packet = [0x80, 0xF0, 0x10, 0x03, 0xFF, 0xFF, 0x00];
        assert_eq!(Ssm2::calculate_checksum(&packet), 0x81);
    }

    // check_packet
    #[test]
    fn test_check_packet_valid() {
        let packet = [
            0x80, 0xF0, 0x10, 0x29, 0xFF, 0xA1, 0x10, 0x0D, 0x16, 0x04, 0x69, 0x05, 0x05, 0x61,
            0xE4, 0xEB, 0x80, 0x0A, 0x0A, 0x00, 0x00, 0x00, 0x00, 0x00, 0x70, 0xDE, 0x64, 0xF8,
            0xBC, 0x08, 0x00, 0x00, 0xE0, 0x00, 0x00, 0x00, 0x00, 0x00, 0xDC, 0x00, 0x00, 0x00,
            0x80, 0x00, 0x00, 0x61,
        ];
        assert_eq!(Ssm2::check_packet(&packet), packet.len());
    }

    #[test]
    fn test_check_packet_wrong_src_dst() {
        let packet = [0x80, 0x10, 0xF0, 0x01, 0x01, 0x02];
        assert_eq!(Ssm2::check_packet(&packet), 0);
    }

    #[test]
    fn test_check_packet_wrong_length() {
        let packet = [0x80, 0x10, 0xF0, 0x00];
        assert_eq!(Ssm2::check_packet(&packet), 0);
    }

    #[test]
    fn test_check_packet_wrong_data() {
        let packet = [0x80, 0x10, 0xF0, 0x01];
        assert_eq!(Ssm2::check_packet(&packet), 0);
    }

    #[test]
    fn test_check_packet_wrong_check_sum() {
        let packet = [0x80, 0x10, 0xF0, 0x03, 0x01, 0x02, 0x03, 0x00];
        assert_eq!(Ssm2::check_packet(&packet), 0);
    }
}
