use std::pin::Pin;

use crate::protocol::PacketId;
use crate::{DeserFromBuf, OpenRgbError, OpenRgbResult, ReceivedMessage, SerToBuf, WriteMessage};
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt},
    net::{TcpStream, ToSocketAddrs},
};

/// Utility struct to write packets.
/// Some packets need to be prepended by their length.
/// This struct serializes the contents and prepends the length to the buffer.
pub(crate) struct OpenRgbPacket<T: SerToBuf> {
    pub contents: T,
}

impl<T: SerToBuf> OpenRgbPacket<T> {
    pub fn new(contents: T) -> OpenRgbPacket<T> {
        Self { contents }
    }
}

impl<T: SerToBuf> SerToBuf for OpenRgbPacket<T> {
    fn serialize(&self, buf: &mut WriteMessage) -> OpenRgbResult<()> {
        let mut inner_buf = WriteMessage::new(buf.protocol_version());
        self.contents.serialize(&mut inner_buf)?;
        let len = inner_buf.len() + size_of::<u32>(); // + u32 to account for the length field itself
        buf.write_u32(len as u32);
        buf.write_slice(inner_buf.bytes());
        Ok(())
    }
}

pub(crate) struct OpenRgbMessageHeader {
    packet_id: PacketId,
    device_id: u32,
    packet_size: u32,
}

impl OpenRgbMessageHeader {
    pub(crate) const MAGIC: [u8; 4] = *b"ORGB";

    async fn read(stream: &mut TcpStream) -> OpenRgbResult<Self> {
        // header is always 16 bytes long
        let mut buf = [0u8; 16];
        stream.read_exact(&mut buf).await?;
        let mut recv = ReceivedMessage::new(&buf, 0); // header is constant across protocol versions
        tracing::trace!("Read header: {}", recv);
        let magic = recv.read_value::<[u8; 4]>()?;
        if magic != Self::MAGIC {
            return Err(OpenRgbError::ProtocolError(format!(
                "expected OpenRGB magic value, got {magic:?}"
            )));
        }

        let device_id = recv.read_u32()?;
        let packet_id = recv.read_value::<PacketId>()?;
        let packet_size = recv.read_u32()?;
        Ok(Self {
            device_id,
            packet_id,
            packet_size,
        })
    }

    async fn write(&self, stream: &mut TcpStream) -> OpenRgbResult<()> {
        let mut buf = WriteMessage::with_capacity(0, 16);
        buf.write_slice(&Self::MAGIC);
        buf.write_u32(self.device_id);
        buf.write_value(&self.packet_id)?;
        buf.write_u32(self.packet_size);
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

    pub async fn request<I: SerToBuf, O: DeserFromBuf>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data: &I,
    ) -> OpenRgbResult<O> {
        self.write_packet(device_id, packet_id, data).await?;
        self.read_packet(device_id, packet_id).await
    }

    pub async fn read_packet<T: DeserFromBuf>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
    ) -> OpenRgbResult<T> {
        // the header tells us exactly how long the packet is, so we might as well read it all at once
        let header = self.read_header(device_id, packet_id).await?;
        let mut buf = vec![0u8; header.packet_size as usize];
        self.stream.read_exact(&mut buf).await?;
        let mut recv = ReceivedMessage::new(&buf, self.protocol_version());
        tracing::trace!("Read packet: {}", recv);
        T::deserialize(&mut recv)
    }

    pub async fn write_packet<T: SerToBuf>(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
        data: &T,
    ) -> OpenRgbResult<()> {
        // let mut buf = Vec::with_capacity(8);
        let mut buf = WriteMessage::new(self.protocol_version());
        data.serialize(&mut buf)?;
        let packet_size = buf.len() as u32;
        let header = OpenRgbMessageHeader {
            packet_id,
            device_id,
            packet_size,
        };
        header.write(&mut self.stream).await?;

        tracing::debug!("Writing packet: {}", buf);
        self.stream.write_all(buf.bytes()).await?;
        Ok(())
    }

    async fn read_header(
        &mut self,
        device_id: u32,
        packet_id: PacketId,
    ) -> OpenRgbResult<OpenRgbMessageHeader> {
        let header = OpenRgbMessageHeader::read(&mut self.stream).await?;
        if header.packet_id != packet_id {
            return Err(OpenRgbError::ProtocolError(format!(
                "Unexpected packet ID: expected {:?}, got {:?}",
                packet_id, header.packet_id
            )));
        }
        if header.device_id != device_id {
            return Err(OpenRgbError::ProtocolError(format!(
                "Unexpected device ID: expected {}, got {}",
                device_id, header.device_id
            )));
        }
        Ok(header)
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
