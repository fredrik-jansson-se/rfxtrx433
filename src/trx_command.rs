use crate::protocols::*;
use crate::{Result, TRXError};
use log::{error, trace};
use num_derive::FromPrimitive;
use num_traits::FromPrimitive;

pub(crate) type SequenceNumber = u8;

#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u8)]
pub enum PacketType {
    InterfaceControl = 0x00,
    InterfaceMessage = 0x01,
    RecXmitMessage = 0x02,
    Undecoded = 0x03,
    Lighting1 = 0x10,
    Lighting2 = 0x11,
    Lighting3 = 0x12,
    Lighting4 = 0x13,
    Lighting5 = 0x14,
    Lighting6 = 0x15,
    Chime = 0x16,
    Fan = 0x17,
    Curtain = 0x18,
    Blinds = 0x19,
    RFY = 0x1A,
    HomeConfort = 0x1B,
    Funkbus = 0x1E,
    Hunter = 0x1F,
    Security1 = 0x20,
    Security2 = 0x21,
    Camera = 0x28,
    Remote = 0x30,
    Thermostat1 = 0x40,
    Thermostat2 = 0x41,
    Thermostat3 = 0x42,
    Thermostat4 = 0x43,
    Radiator1 = 0x48,
    BBQ = 0x4E,
    TempRain = 0x4F,
    TEMP = 0x50,
    HUM = 0x51,
    TempHum = 0x52,
    BARO = 0x53,
    TempHumBaro = 0x54,
    RAIN = 0x55,
    WIND = 0x56,
    UV = 0x57,
    DT = 0x58,
    CURRENT = 0x59,
    ENERGY = 0x5A,
    CURRENTENERGY = 0x5B,
    POWER = 0x5C,
    WEIGHT = 0x5D,
    GAS = 0x5E,
    WATER = 0x5F,
    CARTELECTRONIC = 0x60,
    ASYNCPORT = 0x61,
    ASYNCDATA = 0x62,
    RFXSensor = 0x70,
    RFXMeter = 0x71,
    FS20 = 0x72,
    WEATHER = 0x76,
    SOLAR = 0x77,
    RAW = 0x7F,
}

#[derive(Debug)]
pub struct PacketHeader {
    packet_type: PacketType,
    sub_type: u8,
    seqnbr: u8,
}

impl PacketHeader {
    fn extend(&self, v: &mut Vec<u8>) {
        v.push(0); // placeholder for size
        v.push(self.packet_type as u8);
        v.push(self.sub_type);
        v.push(self.seqnbr);
    }

    fn parse(data: &[u8]) -> Result<(Self, &[u8])> {
        if data.len() < 3 {
            return Err(TRXError::NotEnoughData {
                received: data.len(),
                expected: 3,
            });
        }
        let packet_type =
            PacketType::from_u8(data[0]).ok_or(TRXError::UnknownPacketType(data[0]))?;
        trace!(
            "Received PacketType: {:?} sub_type: {:02X?}, seqnbr: {:02X?}",
            packet_type,
            data[1],
            data[2]
        );
        Ok((
            Self {
                packet_type,
                sub_type: data[1],
                seqnbr: data[2],
            },
            &data[3..],
        ))
    }
}

#[derive(Clone, Copy, FromPrimitive, Debug)]
#[repr(u8)]
enum InterfaceCommandCmd {
    Reset = 0,
    Status = 0x02,
    SetMode = 0x03,
    Save = 0x06,
    StartReceiver = 0x07,
}

struct InterfaceCommand {
    header: PacketHeader,
    cmd: InterfaceCommandCmd,
    frequency: u8, // Only used in SetMode
    xmitpwr: u8,   // Only used in SetMode
    extra: [u8; 7],
}

impl InterfaceCommand {
    fn to_vec(&self) -> Vec<u8> {
        let mut v = Vec::with_capacity(20);
        self.header.extend(&mut v);
        v.push(self.cmd as u8);
        v.push(self.frequency as u8);
        v.push(self.xmitpwr);
        v.extend_from_slice(&self.extra[..]);
        v[0] = v.len() as u8 - 1;
        v
    }
}

#[derive(Debug, Clone, Copy)]
pub struct EnabledProtocols {
    protos_1: Protocols1,
    protos_2: Protocols2,
    protos_3: Protocols3,
    protos_4: Protocols4,
}

impl Default for EnabledProtocols {
    fn default() -> Self {
        EnabledProtocols {
            protos_1: Protocols1::empty(),
            protos_2: Protocols2::empty(),
            protos_3: Protocols3::empty(),
            protos_4: Protocols4::empty(),
        }
    }
}

impl From<&[u8]> for EnabledProtocols {
    fn from(bytes: &[u8]) -> Self {
        assert!(bytes.len() == 4);
        EnabledProtocols {
            protos_1: Protocols1::from_bits_truncate(bytes[0]),
            protos_2: Protocols2::from_bits_truncate(bytes[1]),
            protos_3: Protocols3::from_bits_truncate(bytes[2]),
            protos_4: Protocols4::from_bits_truncate(bytes[3]),
        }
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u8)]
/// Type to specify the receiver/tranceiver frequency
pub enum Frequency {
    ///
    TrxType310 = 0x50,
    ///
    TrxType315 = 0x51,
    /// Receiver 433.92 Mhz
    RecType43392 = 0x52,
    /// Tranceiver 433.92 Mhz (default)
    TrxType43392 = 0x53,
    /// 433.32 Mhz
    RecType43342 = 0x54,
    ///
    TrxType868 = 0x55,
    /// 434.5 Mhz
    RecType43450 = 0x5f,
}

impl Default for Frequency {
    fn default() -> Self {
        Frequency::TrxType43392
    }
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
#[repr(u8)]
pub enum FWType {
    TypeRec = 0x0,
    Type1 = 0x1,
    Type2 = 0x2,
    TypeExt = 0x3,
    TypeExt2 = 0x4,
    TypePro1 = 0x5,
    TypePro2 = 0x6,
    TypeProXL1 = 0x10,
}

#[derive(Debug)]
pub enum InterfaceMessage {
    Status {
        frequency: Frequency,
        fw_version: u8,
        enabled_protocols: EnabledProtocols,
    },
    SetMode,
    ReceiverStarted,
    Save,
}

impl InterfaceMessage {
    fn parse(header: PacketHeader, data: &[u8]) -> Result<Self> {
        let sub_type = InterfaceMessageSubType::from_u8(header.sub_type).ok_or(
            TRXError::UnknownSubPacketType {
                packet_type: PacketType::InterfaceMessage,
                sub_type: header.sub_type,
            },
        )?;
        let cmd = InterfaceCommandCmd::from_u8(data[0])
            .ok_or(TRXError::UnknownInterfaceMessageCommand(data[0]))?;
        trace!(
            "Received InterfaceMeessage sub_type: {:?} cmd: {:?}",
            sub_type,
            cmd
        );
        match sub_type {
            InterfaceMessageSubType::InterfaceResponse => match cmd {
                InterfaceCommandCmd::Status => Ok(InterfaceMessage::Status {
                    frequency:
                        Frequency::from_u8(data[1]) //.unwrap_or(HWType::Unknown),
                            .ok_or(TRXError::UnknownHardwareType(data[1]))?,
                    fw_version: data[2],
                    enabled_protocols: data[3..7].into(),
                }),
                InterfaceCommandCmd::SetMode => Ok(InterfaceMessage::SetMode),
                InterfaceCommandCmd::Save => Ok(InterfaceMessage::Save),

                cmd => {
                    error!("No code to handle {:?}", cmd);
                    unreachable!();
                }
            },
            InterfaceMessageSubType::RecStarted => Ok(InterfaceMessage::ReceiverStarted),
            _ => unreachable!(),
        }
    }
}

#[derive(Clone, Copy, FromPrimitive, Debug)]
#[repr(u8)]
enum InterfaceMessageSubType {
    InterfaceResponse = 0x00,
    UnknownRFYremote = 0x01,
    ExtError = 0x02,
    RFYremoteList = 0x03,
    ASAremoteList = 0x04,
    RecStarted = 0x07,
    InterfaceWrongCommand = 0xFF,
}

#[repr(u8)]
enum InterfaceControlSubType {
    InterfaceCommand = 0x00,
}

#[derive(Debug)]
pub(crate) enum ReceivedCommand {
    InterfaceMessage(InterfaceMessage),
    ProtocolMessage(ProtocolMessage),
}

#[derive(Debug)]
/// Returned value from reading protocol messages
pub enum ProtocolMessage {
    /// Temperature & humidity
    TempHum(TempHum),
    /// Raw data
    NotParsed {
        /// Packet header
        header: PacketHeader,
        /// Remaining data
        data: Vec<u8>,
    },
}

#[derive(Clone, Copy, Debug)]
/// Temperature and humidity
pub struct TempHum {
    pub id: u16,
    pub temp: f32,
    pub humidity: u8,
    pub humidity_status: u8,
    pub battery_level: u8,
    pub rssi: u8,
}

impl TempHum {
    fn parse(_header: PacketHeader, data: &[u8]) -> Result<Self> {
        if data.len() < 7 {
            return Err(TRXError::NotEnoughData {
                received: data.len(),
                expected: 7,
            });
        }
        let id = ((data[0] as u16) << 8) | data[1] as u16;

        let temp_sign = data[2] & 0x80;
        let temp_high = data[2] & 0x7f;
        let temp_low = data[3];

        let temp = if temp_sign != 0 {
            -((temp_high as i16) << 8 | temp_low as i16)
        } else {
            (temp_high as i16) << 8 | temp_low as i16
        };

        let temp = temp as f32 / 10.0;

        let humidity = data[4];
        let humidity_status = data[5];

        let battery_level = data[6] >> 4;
        let rssi = data[6] & 0x0f;

        Ok(Self {
            id,
            temp,
            humidity,
            humidity_status,
            battery_level,
            rssi,
        })
    }
}

pub(crate) fn reset(seqnbr: SequenceNumber) -> Vec<u8> {
    InterfaceCommand {
        header: PacketHeader {
            packet_type: PacketType::InterfaceControl,
            sub_type: InterfaceControlSubType::InterfaceCommand as u8,
            seqnbr,
        },
        cmd: InterfaceCommandCmd::Reset,
        frequency: 0,
        xmitpwr: 0,
        extra: [0; 7],
    }
    .to_vec()
}

pub(crate) fn get_status(seqnbr: SequenceNumber) -> Vec<u8> {
    InterfaceCommand {
        header: PacketHeader {
            packet_type: PacketType::InterfaceControl,
            sub_type: InterfaceControlSubType::InterfaceCommand as u8,
            seqnbr,
        },
        cmd: InterfaceCommandCmd::Status,
        frequency: 0,
        xmitpwr: 0,
        extra: [0; 7],
    }
    .to_vec()
}

pub(crate) fn start_receiver(seqnbr: SequenceNumber) -> Vec<u8> {
    InterfaceCommand {
        header: PacketHeader {
            packet_type: PacketType::InterfaceControl,
            sub_type: InterfaceControlSubType::InterfaceCommand as u8,
            seqnbr,
        },
        cmd: InterfaceCommandCmd::StartReceiver,
        frequency: 0,
        xmitpwr: 0,
        extra: [0; 7],
    }
    .to_vec()
}

pub(crate) fn parse_message(data: &[u8]) -> Result<ReceivedCommand> {
    let (header, data) = PacketHeader::parse(data)?;

    match header.packet_type {
        PacketType::InterfaceMessage => Ok(ReceivedCommand::InterfaceMessage(
            InterfaceMessage::parse(header, data)?,
        )),
        PacketType::TempHum => Ok(ReceivedCommand::ProtocolMessage(ProtocolMessage::TempHum(
            TempHum::parse(header, data)?,
        ))),

        // Catch all if we receive a command we don't know how to handle
        _ => Ok(ReceivedCommand::ProtocolMessage(
            ProtocolMessage::NotParsed {
                header,
                data: data.to_vec(),
            },
        )),
    }
}

pub(crate) fn set_mode(
    seqnbr: SequenceNumber,
    frequency: Frequency,
    protos_1: Protocols1,
    protos_2: Protocols2,
    protos_3: Protocols3,
    protos_4: Protocols4,
) -> Vec<u8> {
    InterfaceCommand {
        header: PacketHeader {
            packet_type: PacketType::InterfaceControl,
            sub_type: InterfaceControlSubType::InterfaceCommand as u8,
            seqnbr,
        },
        cmd: InterfaceCommandCmd::SetMode,
        frequency: frequency as u8,
        xmitpwr: 0,
        extra: [
            protos_1.bits(),
            protos_2.bits(),
            protos_3.bits(),
            protos_4.bits(),
            0,
            0,
            0,
        ],
    }
    .to_vec()
}

pub(crate) fn save(seqnbr: SequenceNumber) -> Vec<u8> {
    InterfaceCommand {
        header: PacketHeader {
            packet_type: PacketType::InterfaceControl,
            sub_type: InterfaceControlSubType::InterfaceCommand as u8,
            seqnbr,
        },
        cmd: InterfaceCommandCmd::Save,
        frequency: 0,
        xmitpwr: 0,
        extra: [0; 7],
    }
    .to_vec()
}

#[cfg(test)]
mod test {
    use super::*;
    #[test]
    fn create_reset() {
        let cmd = reset(1);
        let cmd = cmd.to_vec();
        assert_eq!(vec![0xd, 0, 0, 1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0], cmd);
    }

    #[test]
    fn create_save_settings() {
        let cmd = save(0x11).to_vec();
        assert_eq!(vec![0x0d, 00, 00, 0x11, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0], cmd);
    }

    #[test]
    fn set_mode_x10() {
        // X10
        let cmd = set_mode(
            0x12,
            Default::default(),
            Protocols1::empty(),
            Protocols2::empty(),
            Protocols3::X10,
            Protocols4::empty(),
        )
        .to_vec();
        assert_eq!(
            vec![0x0d, 00, 00, 0x12, 03, 0x53, 00, 00, 00, 01, 00, 00, 00, 00],
            cmd
        );
    }

    #[test]
    fn set_mode_mixed() {
        let cmd = set_mode(
            0x12,
            Default::default(),
            Protocols1::IMAGINTRONIX | Protocols1::RUBICSON,
            Protocols2::LEGRAND | Protocols2::MERTIK,
            Protocols3::X10 | Protocols3::ATI,
            Protocols4::KEELOQ,
        )
        .to_vec();
        assert_eq!(
            vec![0x0d, 00, 00, 0x12, 03, 0x53, 00, 0x42, 0x11, 0x41, 0x01, 00, 00, 00],
            cmd
        );
    }

    #[test]
    fn save_cmd() {
        let cmd = super::save(3).to_vec();
        assert_eq!(vec![0x0d, 0, 0, 3, 6, 0, 0, 0, 0, 0, 0, 0, 0, 0], cmd);
    }
}
