use std::ops::{Deref, DerefMut};

use bytes::{Bytes, BytesMut};
use sqlx_rt::{AsyncWriteExt, TcpStream};

use crate::error::Error;
use crate::ext::ustr::UStr;
use crate::io::{BufStream, Encode};
use crate::mssql::connection::tls_prelogin_stream_wrapper::TlsPreloginWrapper;
use crate::mssql::protocol::col_meta_data::ColMetaData;
use crate::mssql::protocol::done::{Done, Status as DoneStatus};
use crate::mssql::protocol::env_change::EnvChange;
use crate::mssql::protocol::error::Error as ProtocolError;
use crate::mssql::protocol::info::Info;
use crate::mssql::protocol::login_ack::LoginAck;
use crate::mssql::protocol::message::{Message, MessageType};
use crate::mssql::protocol::order::Order;
use crate::mssql::protocol::packet::{PacketHeader, PacketType, Status, PACKET_HEADER_SIZE};
use crate::mssql::protocol::return_status::ReturnStatus;
use crate::mssql::protocol::return_value::ReturnValue;
use crate::mssql::protocol::row::Row;
use crate::mssql::{MssqlColumn, MssqlConnectOptions, MssqlDatabaseError};
use crate::net::{MaybeTlsStream, TlsConfig};
use crate::HashMap;
use std::sync::Arc;

pub(crate) struct MssqlStream {
    inner: BufStream<MaybeTlsStream<TlsPreloginWrapper<TcpStream>>>,

    // how many Done (or Error) we are currently waiting for
    pub(crate) pending_done_count: usize,

    // current transaction descriptor
    // set from ENVCHANGE on `BEGIN` and reset to `0` on a ROLLBACK
    pub(crate) transaction_descriptor: u64,
    pub(crate) transaction_depth: usize,

    // current TabularResult from the server that we are iterating over
    response: Option<(PacketHeader, Bytes)>,

    // most recent column data from ColMetaData
    // we need to store this as its needed when decoding <Row>
    pub(crate) columns: Arc<Vec<MssqlColumn>>,
    pub(crate) column_names: Arc<HashMap<UStr, usize>>,

    // Maximum size of packets to send to the server
    pub(crate) max_packet_size: usize,

    options: MssqlConnectOptions,
}

impl MssqlStream {
    pub(super) async fn connect(options: &MssqlConnectOptions) -> Result<Self, Error> {
        let tcp_stream = TcpStream::connect((&*options.host, options.port)).await?;
        let wrapped_stream = TlsPreloginWrapper::new(tcp_stream);
        let inner = BufStream::new(MaybeTlsStream::Raw(wrapped_stream));

        Ok(Self {
            inner,
            columns: Default::default(),
            column_names: Default::default(),
            response: None,
            pending_done_count: 0,
            transaction_descriptor: 0,
            transaction_depth: 0,
            max_packet_size: options
                .requested_packet_size
                .try_into()
                .unwrap_or(usize::MAX),
            options: options.clone(),
        })
    }

    // writes the packet out to the write buffer, but does not flush
    // WARNING: if the packet is large, and we are over an encrypted connection, this will fail, since we would
    // need to flush each packet individually to the encryption layer
    pub(crate) fn write_packet<'en, T: Encode<'en>>(&mut self, ty: PacketType, payload: T) {
        write_packets(&mut self.inner.wbuf, self.max_packet_size, ty, payload)
    }

    // writes the packet out to the write buffer, splitting it if neccessary, and flushing TDS packets one at a time
    pub(crate) async fn write_packet_and_flush<'en, T: Encode<'en>>(
        &mut self,
        ty: PacketType,
        payload: T,
    ) -> Result<(), Error> {
        if !self.inner.wbuf.is_empty() {
            self.flush().await?;
        }
        self.write_packet(ty, payload);
        self.flush().await?;
        Ok(())
    }

    // writes the packet out to the write buffer, splitting it if neccessary, and flushing TDS packets one at a time
    pub(crate) async fn flush(&mut self) -> Result<(), Error> {
        // flush self.max_packet_size bytes at a time
        if self.inner.wbuf.len() > self.max_packet_size {
            for chunk in self.inner.wbuf.chunks(self.max_packet_size) {
                self.inner.stream.write_all(chunk).await?;
                self.inner.stream.flush().await?;
            }
            self.inner.wbuf.clear();
        } else {
            self.inner.flush().await?;
        }
        Ok(())
    }

    // receive the next packet from the database
    // blocks until a packet is available
    pub(super) async fn recv_packet(&mut self) -> Result<(PacketHeader, Bytes), Error> {
        let mut header: PacketHeader = self.inner.read(8).await?;

        // NOTE: From what I can tell, the response type from the server should ~always~
        //       be TabularResult. Here we expect that and die otherwise.
        if !matches!(header.r#type, PacketType::TabularResult) {
            return Err(err_protocol!(
                "received unexpected packet: {:?}",
                header.r#type
            ));
        }

        let mut payload = BytesMut::new();

        loop {
            self.inner
                .read_raw_into(&mut payload, (header.length - 8) as usize)
                .await?;

            if header.status.contains(Status::END_OF_MESSAGE) {
                break;
            }

            header = self.inner.read(8).await?;
        }

        Ok((header, payload.freeze()))
    }

    // receive the next ~message~
    // TDS communicates in streams of packets that are themselves streams of messages
    pub(super) async fn recv_message(&mut self) -> Result<Message, Error> {
        loop {
            while self.response.as_ref().map_or(false, |r| !r.1.is_empty()) {
                let buf = if let Some((_, buf)) = self.response.as_mut() {
                    buf
                } else {
                    // this shouldn't be reachable but just nope out
                    // and head to refill our buffer
                    break;
                };

                let ty = MessageType::get(buf)?;

                let message = match ty {
                    MessageType::EnvChange => {
                        match EnvChange::get(buf)? {
                            EnvChange::BeginTransaction(desc) => {
                                self.transaction_descriptor = desc;
                            }

                            EnvChange::CommitTransaction(_) | EnvChange::RollbackTransaction(_) => {
                                self.transaction_descriptor = 0;
                            }

                            EnvChange::PacketSize(size) => {
                                self.max_packet_size = size.clamp(512, 32767).try_into().unwrap();
                            }

                            _ => {}
                        }

                        continue;
                    }

                    MessageType::Info => {
                        let _ = Info::get(buf)?;
                        continue;
                    }

                    MessageType::Row => Message::Row(Row::get(buf, false, &self.columns)?),
                    MessageType::NbcRow => Message::Row(Row::get(buf, true, &self.columns)?),
                    MessageType::LoginAck => Message::LoginAck(LoginAck::get(buf)?),
                    MessageType::ReturnStatus => Message::ReturnStatus(ReturnStatus::get(buf)?),
                    MessageType::ReturnValue => Message::ReturnValue(ReturnValue::get(buf)?),
                    MessageType::Done => Message::Done(Done::get(buf)?),
                    MessageType::DoneInProc => Message::DoneInProc(Done::get(buf)?),
                    MessageType::DoneProc => Message::DoneProc(Done::get(buf)?),
                    MessageType::Order => Message::Order(Order::get(buf)?),

                    MessageType::Error => {
                        let error = ProtocolError::get(buf)?;
                        return self.handle_error(error);
                    }

                    MessageType::ColMetaData => {
                        // NOTE: there isn't anything to return as the data gets
                        //       consumed by the stream for use in subsequent Row decoding
                        ColMetaData::get(
                            buf,
                            Arc::make_mut(&mut self.columns),
                            Arc::make_mut(&mut self.column_names),
                        )?;
                        continue;
                    }
                };

                return Ok(message);
            }

            // no packet from the server to iterate (or its empty); fill our buffer
            self.response = Some(self.recv_packet().await?);
        }
    }

    pub(crate) fn handle_done(&mut self, _done: &Done) {
        self.pending_done_count -= 1;
    }

    pub(crate) fn handle_error<T>(&mut self, error: ProtocolError) -> Result<T, Error> {
        // NOTE: [error] is sent IN ADDITION TO [done]
        Err(MssqlDatabaseError(error).into())
    }

    pub(crate) async fn wait_until_ready(&mut self) -> Result<(), Error> {
        if !self.wbuf.is_empty() {
            self.flush().await?;
        }

        while self.pending_done_count > 0 {
            let message = self.recv_message().await?;

            if let Message::DoneProc(done) | Message::Done(done) = message {
                if !done.status.contains(DoneStatus::DONE_MORE) {
                    // finished RPC procedure *OR* SQL batch
                    self.handle_done(&done);
                }
            }
        }

        Ok(())
    }

    pub(crate) async fn setup_encryption(&mut self) -> Result<(), Error> {
        let tls_config = TlsConfig {
            accept_invalid_certs: self.options.trust_server_certificate,
            hostname: self.options.hostname_in_certificate.as_deref().unwrap_or(&self.options.host),
            accept_invalid_hostnames: self.options.hostname_in_certificate.is_none(),
            root_cert_path: self.options.ssl_root_cert.as_ref(),
            client_cert_path: None,
            client_key_path: None,
        };
        self.inner.deref_mut().start_handshake();
        self.inner.upgrade(tls_config).await?;
        self.inner.deref_mut().handshake_complete();
        Ok(())
    }
}

// writes the packet out to the write buffer
pub(crate) fn write_packets<'en, T: Encode<'en>>(
    buffer: &mut Vec<u8>,
    max_packet_size: usize,
    ty: PacketType,
    payload: T,
) {
    assert!(buffer.is_empty());

    let mut packet_header = [0u8; PACKET_HEADER_SIZE].to_vec();
    // leave room for setting the packet header later
    buffer.extend_from_slice(&packet_header);

    // write out the payload
    payload.encode(buffer);

    let len = buffer.len() - PACKET_HEADER_SIZE;

    let max_packet_contents_size = max_packet_size - PACKET_HEADER_SIZE;
    let mut packet_count = len / max_packet_contents_size;
    let last_packet_contents_size = len % max_packet_contents_size;
    if last_packet_contents_size > 0 {
        packet_count += 1;
    }

    // Add space for the missing packet headers
    buffer.resize(len + PACKET_HEADER_SIZE * packet_count, 0);
    // Iterate over packets starting from the end in order to never overwrite an existing packet
    for packet_index in (0..packet_count).rev() {
        let header_start = packet_index * max_packet_size;
        let target_contents_start = header_start + PACKET_HEADER_SIZE;
        let is_last = packet_index + 1 == packet_count;
        let packet_contents_size = if is_last && last_packet_contents_size > 0 {
            last_packet_contents_size
        } else {
            max_packet_contents_size
        };
        let packet_size = packet_contents_size + PACKET_HEADER_SIZE;
        let current_contents_start = PACKET_HEADER_SIZE + packet_index * max_packet_contents_size;
        let current_contents_end = current_contents_start + packet_contents_size;

        if current_contents_start != target_contents_start {
            assert!(current_contents_start < target_contents_start);
            buffer.copy_within(
                current_contents_start..current_contents_end,
                target_contents_start,
            );
        }

        packet_header.truncate(0);
        PacketHeader {
            r#type: ty,
            status: if is_last {
                Status::END_OF_MESSAGE
            } else {
                Status::NORMAL
            },
            length: u16::try_from(packet_size).expect("packet size impossibly large"),
            server_process_id: 0,
            packet_id: 1,
        }
        .encode(&mut packet_header);
        assert_eq!(packet_header.len(), PACKET_HEADER_SIZE);
        buffer[header_start..target_contents_start].copy_from_slice(&packet_header);
    }
}

#[test]
fn test_write_packets() {
    let mut buffer = Vec::<u8>::new();
    // small packet sizes are forbidden, but easy for testing
    write_packets(
        &mut buffer,
        PACKET_HEADER_SIZE + 4,
        PacketType::Rpc,
        &b"123456789"[..],
    );
    // Our 9-byte string was split into 3 packets, each with an 8-byte header
    let expected = b"\
        \x03\x00\x00\x0C\x00\x00\x01\x00\
        1234\
        \x03\x00\x00\x0C\x00\x00\x01\x00\
        5678\
        \x03\x01\x00\x09\x00\x00\x01\x00\
        9";
    assert_eq!(buffer, expected);

    // Test the case when there is no smaller packet in the end
    buffer.truncate(0);
    write_packets(
        &mut buffer,
        PACKET_HEADER_SIZE + 4,
        PacketType::Rpc,
        &b"12345678"[..],
    );
    // Our 9-byte string was split into 3 packets, each with an 8-byte header
    let expected = b"\
        \x03\x00\x00\x0C\x00\x00\x01\x00\
        1234\
        \x03\x01\x00\x0C\x00\x00\x01\x00\
        5678";
    assert_eq!(buffer, expected);
}

impl Deref for MssqlStream {
    type Target = BufStream<MaybeTlsStream<TlsPreloginWrapper<TcpStream>>>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl DerefMut for MssqlStream {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
