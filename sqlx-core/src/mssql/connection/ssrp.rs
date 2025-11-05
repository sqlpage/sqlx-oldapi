use crate::error::Error;
use sqlx_rt::spawn_blocking;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr, ToSocketAddrs, UdpSocket};
use std::str;
use std::time::Duration;

pub(super) async fn resolve_instance_port(host: &str, instance: &str) -> Result<u16, Error> {
    let host = host.to_owned();
    let instance = instance.to_owned();
    spawn_blocking(move || resolve_instance_port_blocking(&host, &instance)).await
}

fn resolve_instance_port_blocking(host: &str, instance: &str) -> Result<u16, Error> {
    if instance.as_bytes().len() > 32 {
        return Err(Error::protocol("SSRP instance name exceeds 32 bytes"));
    }
    let mut request = Vec::with_capacity(instance.len() + 2);
    request.push(0x04);
    request.extend_from_slice(instance.as_bytes());
    request.push(0);
    let mut last_io = None;
    let addrs = (host, 1434).to_socket_addrs()?;
    for addr in addrs {
        match query_addr(addr, &request) {
            Ok(port) => return Ok(port),
            Err(err) => match err {
                Error::Io(_) => last_io = Some(err),
                _ => return Err(err),
            },
        }
    }
    match last_io {
        Some(err) => Err(err),
        None => Err(Error::protocol("SSRP lookup returned no response")),
    }
}

fn query_addr(addr: SocketAddr, request: &[u8]) -> Result<u16, Error> {
    let bind_addr = match addr.ip() {
        IpAddr::V4(_) => SocketAddr::new(IpAddr::V4(Ipv4Addr::UNSPECIFIED), 0),
        IpAddr::V6(_) => SocketAddr::new(IpAddr::V6(Ipv6Addr::UNSPECIFIED), 0),
    };
    let socket = UdpSocket::bind(bind_addr)?;
    socket.set_write_timeout(Some(Duration::from_secs(1)))?;
    socket.set_read_timeout(Some(Duration::from_secs(1)))?;
    socket.send_to(request, addr)?;
    let mut buf = [0u8; 1024];
    let (len, _) = socket.recv_from(&mut buf)?;
    parse_response(&buf[..len])
}

fn parse_response(buf: &[u8]) -> Result<u16, Error> {
    if buf.len() < 3 {
        return Err(Error::protocol("SSRP response too short"));
    }
    if buf[0] != 0x05 {
        return Err(Error::protocol("SSRP response has unexpected type"));
    }
    let length = u16::from_le_bytes([buf[1], buf[2]]) as usize;
    if length == 0 {
        return Err(Error::protocol("SSRP response data empty"));
    }
    if length > buf.len() - 3 {
        return Err(Error::protocol("SSRP response truncated"));
    }
    parse_tcp_port(&buf[3..3 + length])
}

fn parse_tcp_port(data: &[u8]) -> Result<u16, Error> {
    let payload_end = data.iter().position(|&b| b == 0).unwrap_or(data.len());
    let payload = &data[..payload_end];
    let text = str::from_utf8(payload).map_err(|_| Error::protocol("SSRP response not UTF-8"))?;
    for entry in text.split(";;") {
        if entry.is_empty() {
            continue;
        }
        let mut tokens = entry.split(';');
        while let Some(key) = tokens.next() {
            if key.is_empty() {
                continue;
            }
            if key.eq_ignore_ascii_case("tcp") {
                let value = tokens
                    .next()
                    .ok_or_else(|| Error::protocol("SSRP TCP entry missing port"))?;
                if value.is_empty() {
                    return Err(Error::protocol("SSRP TCP entry missing port"));
                }
                let port = value
                    .parse::<u16>()
                    .map_err(|_| Error::protocol("SSRP TCP port invalid"))?;
                return Ok(port);
            } else {
                let _ = tokens.next();
            }
        }
    }
    Err(Error::protocol("SSRP response missing TCP entry"))
}
