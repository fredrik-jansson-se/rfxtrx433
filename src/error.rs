use thiserror::Error;

#[derive(Error, Debug)]
/// Error type for the library
pub enum TRXError {
    /// Couldn't find a device with the given serial number
    #[error("No device with serial number {0} found")]
    DeviceWithSerialNotFound(String),
    /// Sent when the reader is shut down
    #[error("System was shutdown during operation")]
    Shutdown,
    /// Protocol error, not enough data to parse
    #[error("Expected {expected} bytes, received {received} bytes.")]
    NotEnoughData {
        /// Bytes received
        received: usize,
        /// Bytes expected by the parser
        expected: usize,
    },
    /// Unrecognized packet type received.
    #[error("Unknownn packet type: {0}")]
    UnknownPacketType(u8),
    /// Unknown subtype for the packet
    #[error("Unknown sybtype {sub_type} for packet type {packet_type:?}")]
    UnknownSubPacketType {
        /// Packet type
        packet_type: crate::trx_command::PacketType,
        /// Unknown subtype
        sub_type: u8,
    },
    /// Unknown command in an interface message
    #[error("Unknown interface message command: {0}")]
    UnknownInterfaceMessageCommand(u8),
    /// Unknown hardware type
    #[error("Unknown hardware type: {0}")]
    UnknownHardwareType(u8),
    /// Received an unexpected message
    #[error("Unknown message: {0}")]
    UnexpectedMessage(String),
    /// Serial port error
    #[error("Serial port error")]
    SerialPort(#[from] serialport::Error),
    /// IO error
    #[error("IO error")]
    IO(#[from] std::io::Error),
    /// Channel error
    #[error("Tokio send error: {0}")]
    TokioSendError(String),
}
