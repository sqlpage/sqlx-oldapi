use crate::error::Error;
use std::time::Duration;

const SSRP_PORT: u16 = 1434;
const CLNT_UCAST_INST: u8 = 0x04;
const SVR_RESP: u8 = 0x05;
const RESPONSE_TIMEOUT: Duration = Duration::from_secs(1);
const MAX_INSTANCE_NAME_LEN: usize = 32;
const MAX_RESPONSE_SIZE: usize = 1024;

#[derive(Debug)]
struct SsrpError(String);

impl std::fmt::Display for SsrpError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "SSRP error: {}", self.0)
    }
}

impl std::error::Error for SsrpError {}

struct InstanceRequest {
    instance_name: String,
}

impl InstanceRequest {
    fn new(instance_name: String) -> Result<Self, Error> {
        if instance_name.len() > MAX_INSTANCE_NAME_LEN {
            return Err(Error::config(SsrpError(format!(
                "instance name exceeds maximum length of {} bytes",
                MAX_INSTANCE_NAME_LEN
            ))));
        }
        Ok(Self { instance_name })
    }

    fn to_bytes(&self) -> Vec<u8> {
        let mut request = vec![CLNT_UCAST_INST];
        request.extend_from_slice(self.instance_name.as_bytes());
        request.push(0);
        request
    }
}

struct InstanceResponse {
    tcp_port: Option<u16>,
}

impl InstanceResponse {
    fn parse(response_data: &[u8]) -> Result<Self, Error> {
        let (response_str, _, _) = encoding_rs::WINDOWS_1252.decode(response_data);
        let response_str = response_str.as_ref();

        let tcp_port = Self::extract_tcp_port(response_str)?;
        Ok(Self { tcp_port })
    }

    fn extract_tcp_port(response_str: &str) -> Result<Option<u16>, Error> {
        let mut search_start = 0;
        while let Some(tcp_pos) = find_case_insensitive(response_str, search_start, "tcp;") {
            let port_start = tcp_pos + 4;
            if port_start >= response_str.len() {
                break;
            }

            let port_end = response_str[port_start..]
                .find(';')
                .map(|i| port_start + i)
                .unwrap_or(response_str.len());

            if port_end > port_start {
                let port_str = &response_str[port_start..port_end];
                if let Ok(port) = port_str.parse::<u16>() {
                    return Ok(Some(port));
                }
            }

            search_start = tcp_pos + 1;
        }

        Ok(None)
    }
}

pub async fn resolve_instance_port(server: &str, instance: &str) -> Result<u16, Error> {
    let request = InstanceRequest::new(instance.to_string())?;
    let request_bytes = request.to_bytes();
    let response_data = send_request(server, &request_bytes).await?;
    let response = InstanceResponse::parse(&response_data)?;
    response.tcp_port.ok_or_else(|| {
        Error::config(SsrpError(
            "SSRP response does not contain TCP port information".to_string(),
        ))
    })
}

async fn send_request(server: &str, request: &[u8]) -> Result<Vec<u8>, Error> {
    let mut buf = vec![0u8; MAX_RESPONSE_SIZE];
    let n = send_udp_request(server, request, &mut buf).await?;
    validate_and_extract_response(&buf[..n])
}

fn validate_and_extract_response(buf: &[u8]) -> Result<Vec<u8>, Error> {
    if buf.len() < 3 {
        return Err(Error::config(SsrpError(
            "SSRP response too short".to_string(),
        )));
    }

    if buf[0] != SVR_RESP {
        return Err(Error::config(SsrpError(format!(
            "invalid SSRP response header: expected {}, got {}",
            SVR_RESP, buf[0]
        ))));
    }

    let resp_size = u16::from_le_bytes([buf[1], buf[2]]) as usize;
    if resp_size == 0 || resp_size > MAX_RESPONSE_SIZE {
        return Err(Error::config(SsrpError(format!(
            "invalid SSRP response size: {}",
            resp_size
        ))));
    }

    if buf.len() < 3 + resp_size {
        return Err(Error::config(SsrpError(
            "SSRP response truncated".to_string(),
        )));
    }

    Ok(buf[3..3 + resp_size].to_vec())
}

async fn send_udp_request(server: &str, request: &[u8], buf: &mut [u8]) -> Result<usize, Error> {
    #[cfg(feature = "_rt-tokio")]
    {
        use sqlx_rt::timeout;
        use sqlx_rt::tokio::net::UdpSocket;

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket
            .send_to(request, (server, SSRP_PORT))
            .await
            .map_err(|e| Error::config(SsrpError(format!("failed to send SSRP request: {}", e))))?;

        let (n, _) = timeout(RESPONSE_TIMEOUT, socket.recv_from(buf))
            .await
            .map_err(|_| Error::config(SsrpError("SSRP request timed out".to_string())))?
            .map_err(|e| {
                Error::config(SsrpError(format!("failed to receive SSRP response: {}", e)))
            })?;

        Ok(n)
    }

    #[cfg(feature = "_rt-async-std")]
    {
        use sqlx_rt::async_std::net::UdpSocket;
        use sqlx_rt::timeout;

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket
            .send_to(request, (server, SSRP_PORT))
            .await
            .map_err(|e| Error::config(SsrpError(format!("failed to send SSRP request: {}", e))))?;

        let (n, _) = timeout(RESPONSE_TIMEOUT, socket.recv_from(buf))
            .await
            .map_err(|_| Error::config(SsrpError("SSRP request timed out".to_string())))?
            .map_err(|e| {
                Error::config(SsrpError(format!("failed to receive SSRP response: {}", e)))
            })?;

        Ok(n)
    }
}

fn find_case_insensitive(haystack: &str, start: usize, needle: &str) -> Option<usize> {
    if needle.is_empty() {
        return Some(start);
    }

    let haystack_bytes = haystack.as_bytes();
    let needle_bytes = needle.as_bytes();
    let needle_len = needle_bytes.len();

    (start..haystack.len().saturating_sub(needle_len - 1)).find(|&i| {
        haystack_bytes[i..i + needle_len]
            .iter()
            .zip(needle_bytes.iter())
            .all(|(h, n)| h.eq_ignore_ascii_case(n))
    })
}
