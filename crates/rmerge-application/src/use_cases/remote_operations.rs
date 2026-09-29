use crate::ports::out::{GitCliPort, NetworkPort};
use crate::ports::r#in::RemoteOperationsUseCase;
use rmerge_domain::errors::DomainError;
use std::path::Path;
use std::sync::Arc;

pub struct RemoteOperationsService {
    network_port: Arc<dyn NetworkPort>,
    git_cli_port: Arc<dyn GitCliPort>,
}

impl RemoteOperationsService {
    pub fn new(network_port: Arc<dyn NetworkPort>, git_cli_port: Arc<dyn GitCliPort>) -> Self {
        Self {
            network_port,
            git_cli_port,
        }
    }

    pub fn extract_remote_host_and_port(raw_url: &str) -> Option<(String, u16)> {
        let url = raw_url.trim();
        if url.is_empty()
            || url.starts_with('/')
            || url.starts_with('\\')
            || url.starts_with("file://")
        {
            return None;
        }

        let bytes = url.as_bytes();
        if bytes.len() >= 3
            && bytes[0].is_ascii_alphabetic()
            && bytes[1] == b':'
            && (bytes[2] == b'\\' || bytes[2] == b'/')
        {
            return None;
        }

        if let Some(rest) = url.strip_prefix("ssh://") {
            let authority = rest.split('/').next().unwrap_or(rest);
            let host_port = authority.split('@').next_back().unwrap_or(authority);
            return Self::parse_host_port(host_port, 22);
        }

        if let Some(rest) = url.strip_prefix("https://") {
            let authority = rest.split('/').next().unwrap_or(rest);
            let host_port = authority.split('@').next_back().unwrap_or(authority);
            return Self::parse_host_port(host_port, 443);
        }

        if let Some(rest) = url.strip_prefix("http://") {
            let authority = rest.split('/').next().unwrap_or(rest);
            let host_port = authority.split('@').next_back().unwrap_or(authority);
            return Self::parse_host_port(host_port, 80);
        }

        if let Some(rest) = url.strip_prefix("git://") {
            let authority = rest.split('/').next().unwrap_or(rest);
            let host_port = authority.split('@').next_back().unwrap_or(authority);
            return Self::parse_host_port(host_port, 9418);
        }

        if !url.contains("://") && url.contains(':') {
            let before_colon = url.split(':').next().unwrap_or("");
            let host = before_colon.split('@').next_back().unwrap_or(before_colon);
            if !host.is_empty() && !host.contains('/') && !host.contains('\\') {
                return Some((host.to_string(), 22));
            }
        }

        None
    }

    fn parse_host_port(host_port: &str, default_port: u16) -> Option<(String, u16)> {
        let hp = host_port.trim();
        if hp.is_empty() {
            return None;
        }
        if hp.starts_with('[') {
            if let Some(end_bracket) = hp.find(']') {
                let host = &hp[1..end_bracket];
                let rest = &hp[end_bracket + 1..];
                if let Some(port_str) = rest.strip_prefix(':') {
                    if let Ok(port) = port_str.parse::<u16>() {
                        return Some((host.to_string(), port));
                    }
                }
                return Some((host.to_string(), default_port));
            }
        }

        if let Some((host, port_str)) = hp.rsplit_once(':') {
            if let Ok(port) = port_str.parse::<u16>() {
                return Some((host.to_string(), port));
            }
        }

        Some((hp.to_string(), default_port))
    }
}

#[async_trait::async_trait]
impl RemoteOperationsUseCase for RemoteOperationsService {
    async fn execute_remote_command(
        &self,
        repo_path: &Path,
        remote_url: Option<String>,
        args: &[&str],
    ) -> Result<String, DomainError> {
        let is_remote_op = args
            .iter()
            .any(|&a| a == "pull" || a == "push" || a == "fetch");

        if is_remote_op {
            if let Some(url) = remote_url {
                if let Some((host, port)) = Self::extract_remote_host_and_port(&url) {
                    self.network_port.check_connection(&host, port).await?;
                }
            }
        }

        self.git_cli_port.execute(repo_path, args).await
    }
}
