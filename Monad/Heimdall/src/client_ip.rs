//! Shared client IP extraction for gateway logging and rate limiting.

use std::net::{IpAddr, SocketAddr};

use axum::extract::{ConnectInfo, Request};

/// Return the most trustworthy client IP available for a request.
///
/// We only trust `X-Forwarded-For` when the direct peer is clearly a trusted
/// proxy on a private or loopback network.
pub fn extract_client_ip(req: &Request) -> String {
    let peer_addr = req
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .map(|c| c.0);

    let Some(peer_addr) = peer_addr else {
        return "unknown".to_owned();
    };

    if is_trusted_proxy(peer_addr.ip()) {
        if let Some(forwarded_for) = req.headers().get("x-forwarded-for") {
            if let Ok(value) = forwarded_for.to_str() {
                if let Some(first) = value.split(',').next() {
                    let candidate = first.trim();
                    if !candidate.is_empty() {
                        return candidate.to_owned();
                    }
                }
            }
        }
    }

    peer_addr.ip().to_string()
}

fn is_trusted_proxy(ip: IpAddr) -> bool {
    match ip {
        IpAddr::V4(v4) => v4.is_loopback() || v4.is_private() || v4.is_link_local(),
        IpAddr::V6(v6) => v6.is_loopback() || v6.is_unique_local() || v6.is_unicast_link_local(),
    }
}