use std::fmt;
use std::error;

pub const MAXIMUM_PAYLOAD_LENGTH: usize = 1460;

pub enum RconPacketType {
    Login,
    Command,
}

fn packet_type_id(packet_type: RconPacketType) -> u32 {
    match packet_type {
        RconPacketType::Login => 3,
        RconPacketType::Command => 2,
    }
}

struct RconPacket {
    request_id: i32,
    packet_type: RconPacketType,
    payload: str,
}

impl RconPacket {
    pub fn new<S: Into<str>>(
        request_id: i32,
        packet_type: RconPacketType,
        payload: S) -> Result<::Self, RCONPacketError> {
        if !payload.is_ascii() {
            return Err(RCONPacketError::Ascii)
        }

        // Convert string to ascii bytes
        let payload_bytes = payload.as_bytes();
        let payload_length = payload_bytes.len();

        // From https://wiki.vg/RCON


        if payload_length > MAXIMUM_PAYLOAD_LENGTH {
            return Err(RCONPacketError::TooLong(PayloadTooLongError{
                max_payload_length: MAXIMUM_PAYLOAD_LENGTH,
                actual_payload_length: payload_length,
            }))
        }

        Ok(RconPacket{
            request_id: request_id,
            packet_type: packet_type,
            payload: payload.into(),
        })
    }

    pub fn serialise(&self, data: &mut [u8]) -> usize {
        // The plus 14 is because of three 32-bit ints, and two bytes of padding.
        let total_packet_length = payload_length + 14;
        // For the length we put at the start, don't count the first i32
        let remainder_of_packet_length = payload_length + 10;

        // First four bytes will be the length.
        packet[0..4].copy_from_slice(&(remainder_of_packet_length as u32).to_le_bytes());
        // The request ID
        packet[4..8].copy_from_slice(&(request_id as u32).to_le_bytes());
        // The packet type
        packet[8..12].copy_from_slice(&(packet_type_id(packet_type) as u32).to_le_bytes());
        // Payload
        packet[12..(12 + payload_length)].copy_from_slice(payload_bytes);
        // Two nil bytes of padding
        packet[(12 + payload_length)] = 0 as u8;
        packet[(13 + payload_length)] = 0 as u8;

        total_packet_length
    }

    pub fn deserialise(data: &mut [u8]) -> Result<::Self, > {
        // The plus 14 is because of three 32-bit ints, and two bytes of padding.
        let actual_packet_length = data.len();

        // First four bytes will be the declared remainder length.
        let remainder_of_packet_length = u32::from_le_bytes(data[0..4]);
        // The declared length doesn't include the four bytes of integer we just parsed.
        let declared_packet_length = remainder_of_packet_length + 4;
        if declared_packet_length > actual_packet_length {

        }

        data[0..4].copy_from_slice(&(remainder_of_packet_length as i32).to_le_bytes());
        // The request ID
        data[4..8].copy_from_slice(&(request_id as i32).to_le_bytes());
        // The packet type
        data[8..12].copy_from_slice(&(packet_type_id(packet_type) as i32).to_le_bytes());
        // Payload
        data[12..(12 + payload_length)].copy_from_slice(payload_bytes);
        // Two nil bytes of padding
        data[(12 + payload_length)] = 0 as u8;
        data[(13 + payload_length)] = 0 as u8;

        total_packet_length

    }
}

#[cfg(test)]
mod tests {
    // Note this useful idiom: importing names from outer (for mod tests) scope.
    use super::*;

    #[test]
    fn test_packet_type_id() {
        assert_eq!(packet_type_id(RconPacketType::Command), 2);
        assert_eq!(packet_type_id(RconPacketType::Login), 3);
    }

    #[test]
    fn test_write_rcon_packet() -> Result<(), RCONPacketError> {
        let mut packet = [0 as u8; MAXIMUM_PAYLOAD_LENGTH];
        let packet_size = write_rcon_packet(1337, RconPacketType::Command, "list uuids", &mut packet)?;

        assert_eq!(packet_size, 24);
        assert_eq!(packet[0..packet_size],
            [
                // Length of 24
                20, 0x0, 0x0, 0x0,
                // Request ID of 1337 is 0x539 in Hexidecimal. Little endian though.
                0x39, 0x5, 0x0, 0x0,
                // Packet type of 2
                2, 0x0, 0x0, 0x0,
                // String, everything is in hex.
                0x6c, // l
                0x69, // i
                0x73, // s
                0x74, // t
                0x20, // SPACE
                0x75, // u
                0x75, // u
                0x69, // i
                0x64, // d
                0x73, // s
                // Two nil bytes padding
                0x0, 0x0
            ]);
        Ok(())
    }

    #[test]
    fn test_write_rcon_packet_unicode() {
        let mut packet = [0 as u8; MAXIMUM_PAYLOAD_LENGTH];

        match write_rcon_packet(1337, RconPacketType::Command, "(╯°□°)╯︵ ┻━┻", &mut packet) {
            Ok(_) => assert!(false),
            Err(e) => match e {
                RCONPacketError::Ascii => (),
                _ => assert!(false),
            },
        }
    }
}

pub fn write_rcon_packet<S: AsRef<str>>(
    request_id: i32,
    packet_type: RconPacketType,
    payload: S,
    packet: &mut [u8]) -> Result<usize, RCONPacketError> {



    return Ok(total_packet_length)
}

pub enum RCONPacketDeserialiseError {
    InvalidLength(InvalidLengthError),
    InvalidType(InvalidTypeError),

}

#[derive(Debug, Clone)]
pub enum RCONPacketError {
    Ascii,
    TooLong(PayloadTooLongError),
}

impl fmt::Display for RCONPacketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            RCONPacketError::Ascii => write!(f, "Payload was not ASCII"),
            RCONPacketError::TooLong(ref err) => {
                write!(f, "Payload too long. Maximum size is {} bytes (ASCII), message was {} bytes.",
                       err.max_payload_length,
                       err.actual_payload_length)
            },
        }
    }
}

impl error::Error for RCONPacketError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        match *self {
            RCONPacketError::Ascii => None,
            RCONPacketError::TooLong(_) => None,
        }
    }
}

#[derive(Debug, Clone)]
pub struct PayloadTooLongError {
    actual_payload_length: usize,
    max_payload_length: usize,
}

