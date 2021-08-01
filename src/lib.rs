//! Library for interacting with RFXtrx433 devices.
//!
//! # Example
//! ```no_run
//! use rfxtrx433::*;
//!
//! #[tokio::main]
//! async fn main() -> crate::Result<()> {
//!     let mut rfx = RFXtrx433::new_from_serial_number("123ABC").await?;
//!
//!     // Send reset signal to the device
//!     rfx.reset().await?;
//!
//!     // Start the receiver
//!     rfx.start_receiver().await?;
//!
//!     // Configure what protocols we're interested in
//!     rfx.set_mode(
//!         Default::default(),
//!         Protocols1::FINEOFFSET,
//!         Protocols2::empty(),
//!         Protocols3::empty(),
//!         Protocols4::empty(),
//!     )
//!     .await?;
//!
//!     // After connecting and configuring the device we will receive messages
//!     loop {
//!        let msg = rfx.read_message().await?;
//!        println!("Received message {:?}", msg);
//!     }
//!     Ok(())
//! }
//! ```

#![warn(missing_docs)]

use log::{debug, error, trace};

/// Result type used by the library
pub type Result<T> = std::result::Result<T, TRXError>;

mod error;
mod protocols;
mod trx_command;

pub use error::TRXError;
pub use protocols::{Protocols1, Protocols2, Protocols3, Protocols4};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    select,
    sync::mpsc::{
        channel as bounded_channel, unbounded_channel, Receiver as BoundedReceiver,
        Sender as BoundedSender, UnboundedReceiver, UnboundedSender,
    },
};
use tokio_serial::SerialPortBuilderExt;
use trx_command::ReceivedCommand;
pub use trx_command::{Frequency, ProtocolMessage};

const MESSAGE_QUEUE_LEN: usize = 100;

///
/// Tries to read a message from the serial port, if a message with size=0 is received,
/// None is returned.
async fn read_message(sp: &mut tokio_serial::SerialStream) -> Result<Option<Vec<u8>>> {
    let mut buffer = Vec::with_capacity(255);

    // First byte is the size
    buffer.resize(1, 0);
    sp.read_exact(&mut buffer).await?;

    let size = buffer[0] as usize;

    if size == 0 {
        return Ok(None);
    }

    buffer.resize(size, 0);

    sp.read_exact(&mut buffer).await?;

    trace!("Received {} bytes, {:02X?}", size, buffer);

    Ok(Some(buffer))
}

///
/// Listens for serial port messages
async fn serial_port(
    mut sp: tokio_serial::SerialStream,
    mut to_serial_rx: UnboundedReceiver<Vec<u8>>,
    interface_msg_tx: BoundedSender<trx_command::InterfaceMessage>,
    protocol_msg_tx: BoundedSender<trx_command::ProtocolMessage>,
) -> Result<()> {
    loop {
        select! {
            msg = to_serial_rx.recv() => match msg {
                // Shutdown if the channel is closed
                None => return Ok(()),
                Some(msg) => {
                    trace!("Sending {:02X?}", msg);
                    sp.write_all(&msg).await?;
                },
            },
            msg = read_message(&mut sp) => match msg {
                Ok(Some(msg)) => {
                    match trx_command::parse_message(&msg) {
                        Ok(ReceivedCommand::InterfaceMessage(msg)) => {
                            interface_msg_tx.send(msg).await
                                .map_err(|e| TRXError::TokioSendError(format!("{}", e)))?;
                            }
                        Ok(ReceivedCommand::ProtocolMessage(msg)) => {
                            protocol_msg_tx.send(msg).await
                                .map_err(|e| TRXError::TokioSendError(format!("{}", e)))?;
                            },

                        Err(e) => {
                            error!("Parsing error {}", e);
                        }
                    };
                },
                // Ignore empty messages read
                Ok(None) => {},
                Err(e) => return Err(e),
            }
        }
    }
}

/// This structs owns the serial port and provides the functions to configure the RFXtrx433 device.
pub struct RFXtrx433 {
    seqnbr: trx_command::SequenceNumber,
    to_serial_tx: UnboundedSender<Vec<u8>>,
    interface_msg_rx: BoundedReceiver<trx_command::InterfaceMessage>,
    protocol_msg_rx: BoundedReceiver<trx_command::ProtocolMessage>,
}

impl RFXtrx433 {
    /// Try to create an instance from a serial number.
    /// The function iterates over the available serial ports and tries to match the serial number.
    pub async fn new_from_serial_number(serial: &str) -> Result<Self> {
        let serialports = serialport::available_ports()?;
        trace!("Searching for serial {} in serialports", serial);

        for sp in serialports {
            trace!("Checking for serial ({}) in {:?}", serial, sp);
            if let serialport::SerialPortType::UsbPort(type_info) = sp.port_type {
                if Some(serial) == type_info.serial_number.as_deref() {
                    return Self::new_from_serial_port(&sp.port_name).await;
                }
            }
        }
        Err(TRXError::DeviceWithSerialNotFound(format!(
            "Serial number {}",
            serial
        )))
    }

    /// Create an instance from a serial port tty, e.g. /dev/ttyUSB0
    pub async fn new_from_serial_port(port: &str) -> Result<Self> {
        // let s = serialport::SerialPortSettings {
        //     baud_rate: 38400,
        //     ..Default::default()
        // };
        let sp = tokio_serial::new(port, 38400).open_native_async()?;
        let (to_serial_tx, to_serial_rx) = unbounded_channel();
        let (interface_msg_tx, interface_msg_rx) = bounded_channel(MESSAGE_QUEUE_LEN);
        let (protocol_msg_tx, protocol_msg_rx) = bounded_channel(MESSAGE_QUEUE_LEN);
        tokio::spawn(async move {
            serial_port(sp, to_serial_rx, interface_msg_tx, protocol_msg_tx).await
        });
        Ok(Self {
            seqnbr: 0,
            to_serial_tx,
            interface_msg_rx,
            protocol_msg_rx,
        })
    }

    fn next_seqnbr(&mut self) -> trx_command::SequenceNumber {
        let n = self.seqnbr;
        self.seqnbr = self.seqnbr.wrapping_add(1);
        n
    }

    /// Sends a reset signal to the device
    pub async fn reset(&mut self) -> Result<()> {
        let cmd = trx_command::reset(self.next_seqnbr()).to_vec();
        self.to_serial_tx
            .send(cmd)
            .map_err(|e| TRXError::TokioSendError(format!("{}", e)))?;
        // Need to sleep at least 500 ms after reset
        debug!("Sleeping after sending reset");
        tokio::time::sleep(std::time::Duration::from_millis(1000)).await;
        Ok(())
    }

    /// Sends a get status signal to the device and waits for a response
    pub async fn get_status(&mut self) -> Result<RFXtrx433Info> {
        let msg = trx_command::get_status(self.next_seqnbr()).to_vec();
        debug!("sending get status");
        self.to_serial_tx
            .send(msg)
            .map_err(|e| TRXError::TokioSendError(format!("{}", e)))?;

        let cmd = self
            .interface_msg_rx
            .recv()
            .await
            .ok_or(TRXError::Shutdown)?;
        debug!("Received get_status response");
        trace!("Received command: {:02X?}", cmd);
        if let trx_command::InterfaceMessage::Status {
            enabled_protocols,
            frequency,
            ..
        } = cmd
        {
            Ok(RFXtrx433Info {
                frequency,
                enabled_protocols,
            })
        } else {
            Err(TRXError::UnexpectedMessage(format!(
                "Expected status response, received {:?}",
                cmd
            )))
        }
    }

    /// Starts the receiver and waits for confirmation.
    pub async fn start_receiver(&mut self) -> Result<()> {
        let msg = trx_command::start_receiver(self.next_seqnbr()).to_vec();
        debug!("Sending start_receiver");
        self.to_serial_tx
            .send(msg)
            .map_err(|e| TRXError::TokioSendError(format!("{}", e)))?;

        let cmd = self
            .interface_msg_rx
            .recv()
            .await
            .ok_or(TRXError::Shutdown)?;
        debug!("Received start_receiver response");
        trace!("Received command: {:02X?}", cmd);

        Ok(())
    }

    /// Sets the mode of the receiver, then calls save.
    pub async fn set_mode(
        &mut self,
        frequency: trx_command::Frequency,
        protos_1: Protocols1,
        protos_2: Protocols2,
        protos_3: Protocols3,
        protos_4: Protocols4,
    ) -> Result<()> {
        let msg = trx_command::set_mode(
            self.next_seqnbr(),
            frequency,
            protos_1,
            protos_2,
            protos_3,
            protos_4,
        )
        .to_vec();
        debug!("Sending set_mode");
        self.to_serial_tx
            .send(msg)
            .map_err(|e| TRXError::TokioSendError(format!("{}", e)))?;

        let cmd = self
            .interface_msg_rx
            .recv()
            .await
            .ok_or(TRXError::Shutdown)?;
        trace!("Received command: {:02X?}", cmd);

        let msg = trx_command::save(self.next_seqnbr()).to_vec();

        debug!("Sending save");
        self.to_serial_tx
            .send(msg)
            .map_err(|e| TRXError::TokioSendError(format!("{}", e)))?;

        let cmd = self
            .interface_msg_rx
            .recv()
            .await
            .ok_or(TRXError::Shutdown)?;

        debug!("Received save response");
        trace!("Received command: {:02X?}", cmd);

        Ok(())
    }

    /// This function will wait for protocol messages from the device
    pub async fn read_message(&mut self) -> Result<trx_command::ProtocolMessage> {
        let cmd = self
            .protocol_msg_rx
            .recv()
            .await
            .ok_or(TRXError::Shutdown)?;
        trace!("read_command: received {:?}", cmd);

        Ok(cmd)
    }
}

#[derive(Debug)]
/// Information about the hardware
pub struct RFXtrx433Info {
    /// Currently set frequency
    pub frequency: trx_command::Frequency,
    /// Currently enabled protocols
    pub enabled_protocols: trx_command::EnabledProtocols,
}
