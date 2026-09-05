use std::{pin::Pin, time::Duration};

use crate::{Acknowledge, OpenRgbError::ProtocolError, protocol::PacketId};
use crate::{DeserFromBuf, OpenRgbError, OpenRgbResult, ReceivedMessage, SerToBuf, WriteMessage};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

#[derive(Debug)]
pub(crate) struct RecvPacket {
    pub header: OpenRgbMessageHeader,
    data: Vec<u8>,
}

impl std::fmt::Display for RecvPacket {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "{:?} {:?} ({})",
            self.header.packet_id,
            self.data,
            String::from_utf8_lossy(&self.data)
        )
    }
}

impl RecvPacket {
    pub async fn read(stream: &mut TcpStream) -> OpenRgbResult<Self> {
        let header = OpenRgbMessageHeader::read(stream).await?;
        tracing::debug!("Read header {header:?}");
        Ok(Self {
            data: Self::read_data(stream, &header).await?,
            header,
        })
    }

    async fn read_data(
        stream: &mut TcpStream,
        header: &OpenRgbMessageHeader,
    ) -> OpenRgbResult<Vec<u8>> {
        // the header tells us exactly how long the packet is, so we might as well read it all at once
        let mut data = vec![0; header.packet_size as usize];
        stream.read_exact(&mut data).await?;
        Ok(data)
    }

    pub fn deser<T: DeserFromBuf>(self, protocol_version: u32) -> OpenRgbResult<T> {
        let mut recv = ReceivedMessage::new(&self.data, protocol_version);
        T::deserialize(&mut recv)
    }
}

/// Utility struct to write packets.
/// Some packets need to be prepended by their length.
/// This struct serializes the contents and prepends the length to the buffer.
#[derive(Debug)]
pub(crate) struct OpenRgbWritePacket<T: SerToBuf> {
    pub contents: T,
}

impl<T: SerToBuf> OpenRgbWritePacket<T> {
    pub fn new(contents: T) -> OpenRgbWritePacket<T> {
        Self { contents }
    }
}

impl<T: SerToBuf> SerToBuf for OpenRgbWritePacket<T> {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        let mut inner_buf = WriteMessage::new(buf.protocol_version());
        self.contents.serialize(&mut inner_buf)?;
        let len = inner_buf.len() + size_of::<u32>(); // + u32 to account for the length field itself
        buf.write_u32(len as u32);
        buf.write_slice(inner_buf.bytes());
        Ok(())
    }
}

#[derive(Debug)]
pub(crate) struct OpenRgbMessageHeader {
    pub packet_id: PacketId,
    pub device_id: u32,
    pub packet_size: u32,
}

impl DeserFromBuf for OpenRgbMessageHeader {
    fn deserialize(buf: &mut ReceivedMessage<'_>) -> OpenRgbResult<Self>
    where
        Self: Sized,
    {
        let magic = buf.read_value::<[u8; 4]>()?;
        if magic != Self::MAGIC {
            return Err(OpenRgbError::ProtocolError(format!(
                "expected OpenRGB magic value, got {magic:?}"
            )));
        }

        let device_id = buf.read_value::<u32>()?;
        let packet_id = buf.read_value::<PacketId>()?;
        let packet_size = buf.read_value::<u32>()?;
        Ok(Self {
            device_id,
            packet_id,
            packet_size,
        })
    }
}

impl OpenRgbMessageHeader {
    pub(crate) const MAGIC: [u8; 4] = *b"ORGB";

    async fn read(stream: &mut TcpStream) -> OpenRgbResult<Self> {
        // header is always 16 bytes long
        let mut buf = [0u8; 16];
        stream.read_exact(&mut buf).await?;
        let mut recv = ReceivedMessage::new(&buf, 0); // header is constant across protocol versions
        Self::deserialize(&mut recv)
    }

    async fn write(&self, stream: &mut TcpStream) -> OpenRgbResult<()> {
        let mut buf = WriteMessage::with_capacity(0, 16);
        buf.write_slice(&Self::MAGIC);
        buf.write_u32(self.device_id);
        buf.write_value(self.packet_id)?;
        buf.write_u32(self.packet_size);
        tracing::trace!("Writing header: {:?}", self);
        tracing::trace!("Writing header: {}", buf);
        stream.write_all(buf.bytes()).await?;
        Ok(())
    }
}

/// `tokio TcpStream` with an `OpenRGB` protocol version.
/// The version is tagged to all received and written packets, since packet format depends on protocol version.
pub(crate) struct ProtocolStream {
    stream: TcpStream,
    protocol_version: u32,
}

impl ProtocolStream {
    pub async fn connect<A: ToSocketAddrs>(
        addr: A,
        protocol_version: u32,
    ) -> std::io::Result<Self> {
        let stream = TcpStream::connect(addr).await?;
        Ok(Self {
            stream,
            protocol_version,
        })
    }

    pub fn protocol_version(&self) -> u32 {
        self.protocol_version
    }

    pub fn set_protocol_version(&mut self, version: u32) {
        self.protocol_version = version;
    }

    /// Writes and receives a packet. Use this for packets that expect a response that is not an ACK.
    pub async fn request<I: SerToBuf, O: DeserFromBuf>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data: I,
    ) -> OpenRgbResult<O> {
        self.write_packet(device_id, packet_id, data).await?;
        let read = self.read_packet(device_id, packet_id).await?;
        self.recv_ack(device_id, packet_id).await?;
        Ok(read)
    }

    /// Writes and receives a packet. Use this for packets that don't expect a response (excluding ACK).
    pub async fn write<I: SerToBuf>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data: I,
    ) -> OpenRgbResult<()> {
        self.write_packet(device_id, packet_id, data).await?;
        self.recv_ack(device_id, packet_id).await?;
        Ok(())
    }

    pub async fn write_multiple<I: SerToBuf>(
        &mut self,
        packets: impl IntoIterator<Item = (u32, PacketId, I)>,
    ) -> OpenRgbResult<()> {
        let mut ids = Vec::new();
        for (device_id, packet_id, d) in packets {
            self.write_packet(device_id, packet_id, d).await?;
            ids.push((device_id, packet_id))
        }

        while !ids.is_empty() {
            let packet = self.recv_packet().await?;

            if packet.header.packet_id.is_server_only() {
                continue;
            }

            if packet.header.packet_id != PacketId::Acknowledge {
                return Err(ProtocolError(format!(
                    "Expected acknowledge but received {}",
                    packet.header.packet_id
                )));
            }

            if !ids.iter().any(|(d_id, _)| *d_id == packet.header.device_id) {
                return Err(ProtocolError(format!(
                    "Received packet has invalid device id, received {} but expected one of {:?}",
                    packet.header.device_id, ids
                )));
            }

            ids.retain(|(d_id, _)| *d_id != packet.header.device_id);
            // todo: check ack status
        }
        Ok(())
    }

    #[allow(unused, reason = "might be used later")]
    async fn recv_ack(&mut self, device_id: u32, packet_id: PacketId) -> OpenRgbResult<()> {
        if self.protocol_version < 6 {
            return Ok(()); // only applies to protocol version 6 and up
        }

        let ack = self
            .read_packet::<Acknowledge>(device_id, PacketId::Acknowledge)
            .await?;

        if !ack.status_code.is_ok() {
            return Err(OpenRgbError::ProtocolError(format!(
                "Acknowledge returned error: {:?}",
                ack
            )));
        }

        if ack.packet_id != packet_id {
            return Err(OpenRgbError::ProtocolError(format!(
                "Received acknowledge contains unexpected packet id. Expected {}, got {}",
                packet_id, ack.packet_id
            )));
        }
        Ok(())
    }

    /// Returns received packet without validating the device or packet id.
    pub async fn recv_packet(&mut self) -> OpenRgbResult<RecvPacket> {
        // This reads directly from the stream as opposed to other deserialisation
        RecvPacket::read(&mut self.stream).await
    }

    async fn read_packet<T: DeserFromBuf>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
    ) -> OpenRgbResult<T> {
        loop {
            // Keep receiving packets until we get one that we don't ignore. This is likely the response we're looking for
            let packet =
                tokio::time::timeout(Duration::from_secs(1), async { self.recv_packet().await })
                    .await??;

            if packet.header.packet_id.is_server_only()
                && packet.header.packet_id != packet_id.expected_response()
            {
                tracing::trace!(
                    "Received {} instead of expected {}, ignoring...",
                    packet.header.packet_id,
                    packet_id.expected_response()
                );
                continue;
            }

            let mut recv = ReceivedMessage::new(&packet.data, self.protocol_version());
            // tracing::debug!("Read packet data: {}", recv);
            match packet.header.packet_id {
                p if p != packet_id.expected_response() => {
                    return Err(OpenRgbError::ProtocolError(format!(
                        "Unexpected packet ID: expected {}, got {}",
                        packet_id.expected_response(),
                        packet.header.packet_id
                    )));
                }
                _ => {
                    if packet.header.device_id != device_id {
                        return Err(OpenRgbError::ProtocolError(format!(
                            "Unexpected device ID: expected {}, got {}",
                            device_id, packet.header.device_id
                        )));
                    }

                    return T::deserialize(&mut recv);
                }
            }
        }
    }

    pub(crate) async fn write_packet<T: SerToBuf>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data: T,
    ) -> OpenRgbResult<()> {
        let mut buf = WriteMessage::new(self.protocol_version());
        data.serialize(&mut buf)?;
        let packet_size = buf.len() as u32;
        let header = OpenRgbMessageHeader {
            packet_id,
            device_id,
            packet_size,
        };
        header.write(&mut self.stream).await?;

        tracing::trace!("Writing packet: {}", buf);
        self.stream.write_all(buf.bytes()).await?;
        Ok(())
    }
}

impl AsyncRead for ProtocolStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<std::io::Result<()>> {
        let pin = Pin::new(&mut self.stream);
        AsyncRead::poll_read(pin, cx, buf)
    }
}

impl AsyncWrite for ProtocolStream {
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<Result<usize, std::io::Error>> {
        let pin = Pin::new(&mut self.get_mut().stream);
        AsyncWrite::poll_write(pin, cx, buf)
    }

    fn poll_flush(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let pin = Pin::new(&mut self.get_mut().stream);
        AsyncWrite::poll_flush(pin, cx)
    }

    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), std::io::Error>> {
        let pin = Pin::new(&mut self.get_mut().stream);
        AsyncWrite::poll_shutdown(pin, cx)
    }
}
