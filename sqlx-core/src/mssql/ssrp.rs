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

pub async fn resolve_instance_port(server: &str, instance: &str) -> Result<u16, Error> {
    if instance.len() > MAX_INSTANCE_NAME_LEN {
        return Err(Error::config(SsrpError(format!(
            "instance name exceeds maximum length of {} bytes",
            MAX_INSTANCE_NAME_LEN
        ))));
    }

    let request = build_request(instance)?;
    let response = send_request(server, &request).await?;
    parse_tcp_port(&response)
}

fn build_request(instance: &str) -> Result<Vec<u8>, Error> {
    let mut request = vec![CLNT_UCAST_INST];
    request.extend_from_slice(instance.as_bytes());
    request.push(0);
    Ok(request)
}

async fn send_request(server: &str, request: &[u8]) -> Result<Vec<u8>, Error> {
    #[cfg(feature = "_rt-tokio")]
    {
        use sqlx_rt::tokio::net::UdpSocket;
        use sqlx_rt::timeout;

        let socket = UdpSocket::bind("0.0.0.0:0").await?;
        socket
            .send_to(request, (server, SSRP_PORT))
            .await
            .map_err(|e| Error::config(SsrpError(format!("failed to send SSRP request: {}", e))))?;

        let mut buf = vec![0u8; MAX_RESPONSE_SIZE];
        let (n, _) = timeout(RESPONSE_TIMEOUT, socket.recv_from(&mut buf))
            .await
            .map_err(|_| Error::config(SsrpError("SSRP request timed out".to_string())))?
            .map_err(|e| Error::config(SsrpError(format!("failed to receive SSRP response: {}", e))))?;

        if n < 3 {
            return Err(Error::config(SsrpError("SSRP response too short".to_string())));
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

        if n < 3 + resp_size {
            return Err(Error::config(SsrpError("SSRP response truncated".to_string())));
        }

        Ok(buf[3..3 + resp_size].to_vec())
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

        let mut buf = vec![0u8; MAX_RESPONSE_SIZE];
        let (n, _) = timeout(RESPONSE_TIMEOUT, socket.recv_from(&mut buf))
            .await
            .map_err(|_| Error::config(SsrpError("SSRP request timed out".to_string())))?
            .map_err(|e| Error::config(SsrpError(format!("failed to receive SSRP response: {}", e))))?;

        if n < 3 {
            return Err(Error::config(SsrpError("SSRP response too short".to_string())));
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

        if n < 3 + resp_size {
            return Err(Error::config(SsrpError("SSRP response truncated".to_string())));
        }

        Ok(buf[3..3 + resp_size].to_vec())
    }
}

fn parse_tcp_port(response_data: &[u8]) -> Result<u16, Error> {
    let response_str = encoding_rs::WINDOWS_1252
        .decode(response_data)
        .0
        .to_string();

    let entries: Vec<&str> = response_str.split(";;").collect();

    for entry in entries {
        if entry.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = entry.split(';').collect();
        let mut i = 0;
        while i + 1 < tokens.len() {
            let key = tokens[i];
            let value = tokens[i + 1];

            if key.eq_ignore_ascii_case("tcp") {
                let port = value
                    .parse::<u16>()
                    .map_err(|_| Error::config(SsrpError(format!("invalid TCP port value: {}", value))))?;
                return Ok(port);
            }

            i += 2;
        }
    }

    Err(Error::config(SsrpError(
        "SSRP response does not contain TCP port information".to_string(),
    )))
}
