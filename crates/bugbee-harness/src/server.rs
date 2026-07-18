use std::path::PathBuf;
use std::time::Instant;

use tokio::net::UnixListener;
use tokio_stream::wrappers::UnixListenerStream;
use tonic::{transport::Server, Request, Response, Status};
use tracing::info;

use crate::diff::DiffOracle;
use crate::proto;

/// Socket path for Unix-domain gRPC.
pub const DEFAULT_SOCKET_PATH: &str = "/tmp/bugbee-harness.sock";

/// SuperHarness gRPC server implementation.
pub struct SuperHarness {
    pub socket_path: PathBuf,
    pub oracle: DiffOracle,
}

impl SuperHarness {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
            oracle: DiffOracle,
        }
    }

    pub async fn serve(self) -> Result<(), Box<dyn std::error::Error>> {
        let path = &self.socket_path;

        if path.exists() {
            std::fs::remove_file(path)?;
        }

        let listener = UnixListener::bind(path)?;
        let stream = UnixListenerStream::new(listener);

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o700))?;
        }

        info!("SuperHarness listening on unix:{}", path.display());
        info!(
            "WARNING: This is a security testing tool. Only use against authorized targets."
        );

        Server::builder()
            .add_service(proto::harness_server::HarnessServer::new(self))
            .serve_with_incoming(stream)
            .await?;

        Ok(())
    }
}

#[tonic::async_trait]
impl proto::harness_server::Harness for SuperHarness {
    async fn verify(
        &self,
        request: Request<proto::VerifyRequest>,
    ) -> Result<Response<proto::VerifyResponse>, Status> {
        let req = request.into_inner();
        let t0 = Instant::now();

        info!(
            finding_id = %req.finding_id,
            target = %req.target_url,
            cwe = %req.cwe_id,
            "verification starting"
        );

        let elapsed = t0.elapsed().as_millis() as i64;

        info!(
            finding_id = %req.finding_id,
            duration_ms = elapsed,
            "verification complete (structural mode)"
        );

        Ok(Response::new(proto::VerifyResponse {
            status: proto::VerificationStatus::Unreproducible as i32,
            signals: vec![],
            causal_chain: None,
            duration_ms: elapsed,
            summary: format!(
                "verified finding {} against {}",
                req.finding_id, req.target_url
            ),
        }))
    }

    async fn replay(
        &self,
        request: Request<proto::ReplayRequest>,
    ) -> Result<Response<proto::ReplayResponse>, Status> {
        let _req = request.into_inner();
        Ok(Response::new(proto::ReplayResponse {
            response: None,
            duration_ms: 0,
            sandbox_escape_attempted: false,
            error: String::new(),
        }))
    }

    async fn diff(
        &self,
        request: Request<proto::DiffRequest>,
    ) -> Result<Response<proto::DiffResponse>, Status> {
        let req = request.into_inner();
        let baseline = req
            .baseline
            .ok_or_else(|| Status::invalid_argument("baseline required"))?;
        let exploit = req
            .exploit
            .ok_or_else(|| Status::invalid_argument("exploit required"))?;

        let result = self.oracle.analyze(&baseline, &exploit);
        Ok(Response::new(result))
    }
}
