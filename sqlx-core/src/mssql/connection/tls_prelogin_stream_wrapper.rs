// Original implementation from tiberius: https://github.com/prisma/tiberius/blob/main/src/client/tls.rs

use crate::mssql::protocol::packet::{PacketHeader, PacketType};

use super::stream::write_packets;

use crate::io::Decode;
use bytes::Bytes;
use sqlx_rt::{AsyncRead, AsyncWrite};

#[cfg(feature = "_rt-tokio")]
use sqlx_rt::ReadBuf;

use std::io;
use std::pin::Pin;
use std::task::{self, ready, Poll};

/// This wrapper handles TDS (Tabular Data Stream) packet encapsulation during the TLS handshake phase
/// of a connection to a Microsoft SQL Server.
///
/// In the PRELOGIN phase of the TDS protocol, all communication must be wrapped in TDS packets,
/// even during TLS negotiation. This presents a challenge when using standard TLS libraries,
/// which expect to work with raw TCP streams.
///
/// This wrapper solves the problem by:
/// 1. During handshake:
///    - For writes: It buffers outgoing data and wraps it in TDS packets before sending.
///      Each packet starts with an 8-byte header containing type (0x12 for PRELOGIN),
///      status flags, length, and other metadata.
///    - For reads: It strips the TDS packet headers from incoming data before passing
///      it to the TLS library.
/// 2. After handshake:
///    - It becomes transparent, directly passing through all reads and writes to the
///      underlying stream without modification.
///
/// This allows us to use standard TLS libraries while still conforming to the TDS protocol
/// requirements for the PRELOGIN phase.
const HEADER_BYTES: usize = 8;

pub(crate) struct TlsPreloginWrapper<S> {
    stream: S,
    pending_handshake: bool,

    header_buf: [u8; HEADER_BYTES],
    header_pos: usize,
    read_remaining: usize,

    wr_buf: Vec<u8>,
    header_written: bool,
}

impl<S> TlsPreloginWrapper<S> {
    pub fn new(stream: S) -> Self {
        TlsPreloginWrapper {
            stream,
            pending_handshake: false,

            header_buf: [0u8; HEADER_BYTES],
            header_pos: 0,
            read_remaining: 0,
            wr_buf: Vec::new(),
            header_written: false,
        }
    }

    pub fn start_handshake(&mut self) {
        log::trace!("Handshake starting");
        self.pending_handshake = true;
    }

    pub fn handshake_complete(&mut self) {
        log::trace!("Handshake complete");
        self.pending_handshake = false;
    }
}

#[cfg(feature = "_rt-async-std")]
type PollReadOut = usize;

#[cfg(feature = "_rt-tokio")]
type PollReadOut = ();

impl<S: AsyncRead + AsyncWrite + Unpin + Send> AsyncRead for TlsPreloginWrapper<S> {
    #[cfg(feature = "_rt-tokio")]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<PollReadOut>> {
        if !self.pending_handshake {
            return Pin::new(&mut self.stream).poll_read(cx, buf);
        }

        let inner = self.get_mut();

        if !inner.header_buf[inner.header_pos..].is_empty() {
            while !inner.header_buf[inner.header_pos..].is_empty() {
                let mut header_buf = ReadBuf::new(&mut inner.header_buf[inner.header_pos..]);
                ready!(Pin::new(&mut inner.stream).poll_read(cx, &mut header_buf))?;

                let read = header_buf.filled().len();
                if read == 0 {
                    return Poll::Ready(Ok(PollReadOut::default()));
                }

                inner.header_pos += read;
            }

            let header: PacketHeader = Decode::decode(Bytes::copy_from_slice(&inner.header_buf))
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

            inner.read_remaining = usize::from(header.length) - HEADER_BYTES;

            log::trace!(
                "Discarding header ({:?}), reading packet of {} bytes",
                header,
                inner.read_remaining,
            );
        }

        let max_read = std::cmp::min(inner.read_remaining, buf.remaining());
        let mut limited_buf = buf.take(max_read);

        let res = ready!(Pin::new(&mut inner.stream).poll_read(cx, &mut limited_buf))?;

        let read = limited_buf.filled().len();
        buf.advance(read);
        inner.read_remaining -= read;

        if inner.read_remaining == 0 {
            inner.header_pos = 0;
        }

        Poll::Ready(Ok(res))
    }

    #[cfg(feature = "_rt-async-std")]
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut [u8],
    ) -> Poll<io::Result<usize>> {
        if !self.pending_handshake {
            return Pin::new(&mut self.stream).poll_read(cx, buf);
        }

        let inner = self.get_mut();

        if !inner.header_buf[inner.header_pos..].is_empty() {
            while !inner.header_buf[inner.header_pos..].is_empty() {
                let header_buf = &mut inner.header_buf[inner.header_pos..];
                let read = ready!(Pin::new(&mut inner.stream).poll_read(cx, header_buf))?;

                if read == 0 {
                    return Poll::Ready(Ok(PollReadOut::default()));
                }

                inner.header_pos += read;
            }

            let header: PacketHeader = Decode::decode(Bytes::copy_from_slice(&inner.header_buf))
                .map_err(|err| io::Error::new(io::ErrorKind::Other, err))?;

            inner.read_remaining = usize::from(header.length) - HEADER_BYTES;

            log::trace!(
                "Discarding header ({:?}), reading packet of {} bytes",
                header,
                inner.read_remaining,
            );
        }

        let max_read = std::cmp::min(inner.read_remaining, buf.len());
        let limited_buf = &mut buf[..max_read];

        let read = ready!(Pin::new(&mut inner.stream).poll_read(cx, limited_buf))?;

        inner.read_remaining -= read;

        if inner.read_remaining == 0 {
            inner.header_pos = 0;
        }

        Poll::Ready(Ok(read))
    }
}

impl<S: AsyncRead + AsyncWrite + Unpin + Send> AsyncWrite for TlsPreloginWrapper<S> {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        // Normal operation does not need any extra treatment, we handle
        // packets in the codec.
        if !self.pending_handshake {
            return Pin::new(&mut self.stream).poll_write(cx, buf);
        }

        // Buffering data.
        self.wr_buf.extend_from_slice(buf);

        Poll::Ready(Ok(buf.len()))
    }

    fn poll_flush(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        let inner = self.get_mut();

        // If on handshake mode, wraps the data to a TDS packet before sending.
        if inner.pending_handshake {
            if !inner.header_written {
                let buf = std::mem::take(&mut inner.wr_buf);
                write_packets(
                    &mut inner.wr_buf,
                    4096,
                    PacketType::PreLogin,
                    buf.as_slice(),
                );
                inner.header_written = true;
            }

            while !inner.wr_buf.is_empty() {
                log::trace!("Writing {} bytes of TLS handshake", inner.wr_buf.len());

                let written = ready!(Pin::new(&mut inner.stream).poll_write(cx, &inner.wr_buf))?;

                inner.wr_buf.drain(..written);
            }

            inner.header_written = false;
        }

        Pin::new(&mut inner.stream).poll_flush(cx)
    }

    #[cfg(feature = "_rt-tokio")]
    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_shutdown(cx)
    }

    #[cfg(feature = "_rt-async-std")]
    fn poll_close(mut self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<io::Result<()>> {
        Pin::new(&mut self.stream).poll_close(cx)
    }
}

use std::ops::{Deref, DerefMut};

impl<S> Deref for TlsPreloginWrapper<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.stream
    }
}

impl<S> DerefMut for TlsPreloginWrapper<S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.stream
    }
}
