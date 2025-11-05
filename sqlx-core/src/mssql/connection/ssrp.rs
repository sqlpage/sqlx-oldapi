use crate::error::Error;
use sqlx_rt::{timeout, UdpSocket};
use std::collections::HashMap;
use std::time::Duration;

const SSRP_PORT: u16 = 1434;
const CLNT_UCAST_INST: u8 = 0x04;
const SVR_RESP: u8 = 0x05;
const SSRP_TIMEOUT: Duration = Duration::from_secs(1);

pub(crate) async fn resolve_instance_port(server: &str, instance: &str) -> Result<u16, Error> {
    log::debug!(
        "resolving SQL Server instance port for '{}' on server '{}'",
        instance,
        server
    );

    let mut request = Vec::with_capacity(1 + instance.len() + 1);
    request.push(CLNT_UCAST_INST);
    request.extend_from_slice(instance.as_bytes());
    request.push(0);

    let socket = UdpSocket::bind("0.0.0.0:0").await.map_err(|e| {
        log::debug!("failed to bind UDP socket for SSRP: {}", e);
        err_protocol!("failed to bind UDP socket for SSRP: {}", e)
    })?;

    log::debug!(
        "sending SSRP CLNT_UCAST_INST request to {}:{} for instance '{}'",
        server,
        SSRP_PORT,
        instance
    );

    socket
        .send_to(&request, (server, SSRP_PORT))
        .await
        .map_err(|e| {
            log::debug!("failed to send SSRP request to {}:{}: {}", server, SSRP_PORT, e);
            err_protocol!("failed to send SSRP request to {}:{}: {}", server, SSRP_PORT, e)
        })?;

    let mut buffer = [0u8; 1024];
    let bytes_read = timeout(SSRP_TIMEOUT, socket.recv(&mut buffer))
        .await
        .map_err(|_| {
            log::debug!(
                "SSRP request to {} for instance {} timed out after {:?}",
                server,
                instance,
                SSRP_TIMEOUT
            );
            err_protocol!(
                "SSRP request to {} for instance {} timed out after {:?}",
                server,
                instance,
                SSRP_TIMEOUT
            )
        })?
        .map_err(|e| {
            log::debug!(
                "failed to receive SSRP response from {} for instance {}: {}",
                server,
                instance,
                e
            );
            err_protocol!(
                "failed to receive SSRP response from {} for instance {}: {}",
                server,
                instance,
                e
            )
        })?;

    log::debug!(
        "received SSRP response from {} ({} bytes)",
        server,
        bytes_read
    );

    if bytes_read < 3 {
        log::debug!("SSRP response too short: {} bytes", bytes_read);
        return Err(err_protocol!(
            "SSRP response too short: {} bytes",
            bytes_read
        ));
    }

    if buffer[0] != SVR_RESP {
        log::debug!(
            "invalid SSRP response type: expected 0x05, got 0x{:02x}",
            buffer[0]
        );
        return Err(err_protocol!(
            "invalid SSRP response type: expected 0x05, got 0x{:02x}",
            buffer[0]
        ));
    }

    let response_size = u16::from_le_bytes([buffer[1], buffer[2]]) as usize;
    if response_size + 3 > bytes_read {
        log::debug!(
            "SSRP response size mismatch: expected {} bytes, got {}",
            response_size + 3,
            bytes_read
        );
        return Err(err_protocol!(
            "SSRP response size mismatch: expected {} bytes, got {}",
            response_size + 3,
            bytes_read
        ));
    }

    let response_data = String::from_utf8(buffer[3..(3 + response_size)].to_vec())
        .map_err(|e| {
            log::debug!("SSRP response is not valid UTF-8: {}", e);
            err_protocol!("SSRP response is not valid UTF-8: {}", e)
        })?;

    log::debug!("SSRP response data: {}", response_data);

    parse_ssrp_response(&response_data, instance)
}

fn parse_ssrp_response(data: &str, instance_name: &str) -> Result<u16, Error> {
    let instances: Vec<&str> = data.split(";;").collect();
    log::debug!(
        "parsing SSRP response, found {} instance entries",
        instances.len()
    );

    for instance_data in instances {
        if instance_data.is_empty() {
            continue;
        }

        let tokens: Vec<&str> = instance_data.split(';').collect();
        let mut properties: HashMap<&str, &str> = HashMap::new();

        let mut i = 0;
        while i + 1 < tokens.len() {
            let key = tokens[i];
            let value = tokens[i + 1];
            properties.insert(key, value);
            i += 2;
        }

        if let Some(name) = properties.get("InstanceName") {
            log::debug!("found instance '{}' in SSRP response", name);
            
            if name.eq_ignore_ascii_case(instance_name) {
                log::debug!(
                    "instance '{}' matches requested instance '{}'",
                    name,
                    instance_name
                );
                
                if let Some(tcp_port_str) = properties.get("tcp") {
                    let port = tcp_port_str.parse::<u16>().map_err(|e| {
                        log::debug!(
                            "invalid TCP port '{}' in SSRP response: {}",
                            tcp_port_str,
                            e
                        );
                        err_protocol!(
                            "invalid TCP port '{}' in SSRP response: {}",
                            tcp_port_str,
                            e
                        )
                    })?;
                    
                    log::debug!(
                        "resolved instance '{}' to port {}",
                        instance_name,
                        port
                    );
                    
                    return Ok(port);
                } else {
                    log::debug!(
                        "instance '{}' found but no TCP port available in properties: {:?}",
                        instance_name,
                        properties.keys().collect::<Vec<_>>()
                    );
                    return Err(err_protocol!(
                        "instance '{}' found but no TCP port available",
                        instance_name
                    ));
                }
            }
        }
    }

    log::debug!(
        "instance '{}' not found in SSRP response",
        instance_name
    );
    Err(err_protocol!(
        "instance '{}' not found in SSRP response",
        instance_name
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ssrp_response_single_instance() {
        let data = "ServerName;MYSERVER;InstanceName;SQLEXPRESS;IsClustered;No;Version;15.0.2000.5;tcp;1433;;";
        let port = parse_ssrp_response(data, "SQLEXPRESS").unwrap();
        assert_eq!(port, 1433);
    }

    #[test]
    fn test_parse_ssrp_response_multiple_instances() {
        let data = "ServerName;SRV1;InstanceName;INST1;IsClustered;No;Version;15.0.2000.5;tcp;1433;;ServerName;SRV1;InstanceName;INST2;IsClustered;No;Version;16.0.1000.6;tcp;1434;np;\\\\SRV1\\pipe\\MSSQL$INST2\\sql\\query;;";
        let port = parse_ssrp_response(data, "INST2").unwrap();
        assert_eq!(port, 1434);
    }

    #[test]
    fn test_parse_ssrp_response_case_insensitive() {
        let data = "ServerName;MYSERVER;InstanceName;SQLExpress;IsClustered;No;Version;15.0.2000.5;tcp;1433;;";
        let port = parse_ssrp_response(data, "sqlexpress").unwrap();
        assert_eq!(port, 1433);
    }

    #[test]
    fn test_parse_ssrp_response_instance_not_found() {
        let data = "ServerName;MYSERVER;InstanceName;SQLEXPRESS;IsClustered;No;Version;15.0.2000.5;tcp;1433;;";
        let result = parse_ssrp_response(data, "NOTFOUND");
        assert!(result.is_err());
    }

    #[test]
    fn test_parse_ssrp_response_no_tcp_port() {
        let data = "ServerName;MYSERVER;InstanceName;SQLEXPRESS;IsClustered;No;Version;15.0.2000.5;;";
        let result = parse_ssrp_response(data, "SQLEXPRESS");
        assert!(result.is_err());
    }
}
