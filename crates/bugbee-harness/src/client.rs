use std::time::Duration;

use tonic::transport::{Endpoint, Uri};

use crate::proto::{
    harness_client::HarnessClient as ProtoHarnessClient,
    DiffRequest, DiffResponse, ReplayRequest, ReplayResponse, VerifyRequest, VerifyResponse,
};

const DEFAULT_TIMEOUT: Duration = Duration::from_secs(30);
const CONNECT_TIMEOUT: Duration = Duration::from_secs(5);

/// Wrapper around the gRPC harness client with typed error handling.
pub struct HarnessClient {
    inner: Option<ProtoHarnessClient<tonic::transport::channel::Channel>>,
    socket_path: String,
}

impl HarnessClient {
    pub fn new(socket_path: impl Into<String>) -> Self {
        Self {
            inner: None,
            socket_path: socket_path.into(),
        }
    }

    async fn ensure_connected(&mut self) -> Result<(), HarnessError> {
        if self.inner.is_some() {
            return Ok(());
        }

        let uri: Uri = format!("unix://{}", self.socket_path)
            .parse()
            .map_err(|e| HarnessError::Connect(format!("invalid socket path: {e}")))?;

        let channel = Endpoint::from(uri)
            .connect_timeout(CONNECT_TIMEOUT)
            .timeout(DEFAULT_TIMEOUT)
            .connect()
            .await
            .map_err(|e| {
                // tonic errors don't have code() in all versions
                let msg = e.to_string();
                if msg.contains("No such file") || msg.contains("connection refused") || msg.contains("unavailable") {
                    HarnessError::ProcessDown(format!(
                        "harness not running at {}: {msg}",
                        self.socket_path
                    ))
                } else {
                    HarnessError::Connect(msg)
                }
            })?;

        self.inner = Some(ProtoHarnessClient::new(channel));
        Ok(())
    }

    pub async fn verify(
        &mut self,
        request: VerifyRequest,
    ) -> Result<VerifyResponse, HarnessError> {
        self.ensure_connected().await?;

        let client = self
            .inner
            .as_mut()
            .ok_or_else(|| HarnessError::ProcessDown("not connected".into()))?;

        let result = tokio::time::timeout(DEFAULT_TIMEOUT, client.verify(request))
            .await
            .map_err(|_| HarnessError::Timeout("verify request timed out".into()))?
            .map_err(|e| HarnessError::Rpc(e.to_string()))?;

        Ok(result.into_inner())
    }

    pub async fn replay(
        &mut self,
        request: ReplayRequest,
    ) -> Result<ReplayResponse, HarnessError> {
        self.ensure_connected().await?;

        let client = self
            .inner
            .as_mut()
            .ok_or_else(|| HarnessError::ProcessDown("not connected".into()))?;

        let result = tokio::time::timeout(DEFAULT_TIMEOUT, client.replay(request))
            .await
            .map_err(|_| HarnessError::Timeout("replay request timed out".into()))?
            .map_err(|e| HarnessError::Rpc(e.to_string()))?;

        Ok(result.into_inner())
    }

    pub async fn diff(
        &mut self,
        request: DiffRequest,
    ) -> Result<DiffResponse, HarnessError> {
        self.ensure_connected().await?;

        let client = self
            .inner
            .as_mut()
            .ok_or_else(|| HarnessError::ProcessDown("not connected".into()))?;

        let result = tokio::time::timeout(DEFAULT_TIMEOUT, client.diff(request))
            .await
            .map_err(|_| HarnessError::Timeout("diff request timed out".into()))?
            .map_err(|e| HarnessError::Rpc(e.to_string()))?;

        Ok(result.into_inner())
    }
}

#[derive(Debug, thiserror::Error)]
pub enum HarnessError {
    #[error("harness process not running: {0}")]
    ProcessDown(String),

    #[error("connection failed: {0}")]
    Connect(String),

    #[error("RPC error: {0}")]
    Rpc(String),

    #[error("request timed out")]
    Timeout(String),

    #[error("internal error: {0}")]
    Internal(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_error_display() {
        let err = HarnessError::ProcessDown("socket not found".into());
        assert!(err.to_string().contains("not running"));

        let err = HarnessError::Timeout("timed out".into());
        assert!(err.to_string().contains("timed out"));

        let err = HarnessError::Rpc("broken pipe".into());
        assert!(err.to_string().contains("broken pipe"));
    }

    #[tokio::test]
    async fn test_connect_to_nonexistent() {
        let mut client = HarnessClient::new("/tmp/nonexistent-harness-test.sock");
        let result = client
            .verify(VerifyRequest {
                finding_id: "test".into(),
                target_url: "http://localhost".into(),
                cwe_id: "CWE-89".into(),
                baseline: None,
                exploit: None,
                payload: String::new(),
                metadata: vec![],
            })
            .await;
        // Must return an error (not panic/hang) — the exact variant depends
        // on OS and tonic version timing
        assert!(result.is_err(), "connection to nonexistent socket must fail");
    }
}
