// HTTP client for pushing outbox rows to the central server.
//
// Sends the RECORD uuid (entity_uuid) so the server upserts by record identity
// — retries of the same change never duplicate. Carries tenant_id, device_id,
// and a device bearer token so the server can authenticate the device and scope
// the write to the tenant.

use reqwest::Client;
use std::time::Duration;

/// A single outbox row rendered for transport.
#[derive(Debug, serde::Serialize)]
pub struct SyncRequest {
    pub idempotency_key: String,
    pub entity_type: String,
    pub entity_uuid: String,
    pub operation: String,
    pub payload: String, // JSON string of the record's full state
    pub tenant_id: String,
    pub device_id: String,
}

/// Result of attempting to push one row.
pub enum SyncOutcome {
    /// Server confirmed receipt (2xx). Safe to mark done.
    Confirmed,
    /// Anything else (network error, timeout, non-2xx). Retry later.
    Failed(String),
}

pub struct SyncClient {
    http: Client,
    base_url: String,
    device_token: String,
}

impl SyncClient {
    pub fn new(base_url: &str, device_token: &str) -> Self {
        // A failed builder shouldn't crash the app; fall back to a default client.
        let http = Client::builder()
            .timeout(Duration::from_secs(15))
            .build()
            .unwrap_or_default();
        Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            device_token: device_token.to_string(),
        }
    }

    /// POST one change to /sync/upsert. The server is responsible for
    /// authenticating the device, verifying the tenant, and upserting by
    /// entity_uuid. We treat any 2xx as confirmation.
    pub async fn send(&self, req: &SyncRequest) -> SyncOutcome {
        let url = format!("{}/sync/upsert", self.base_url);

        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.device_token)
            .header("Idempotency-Key", &req.idempotency_key)
            .json(req)
            .send()
            .await;

        match resp {
            Ok(r) if r.status().is_success() => SyncOutcome::Confirmed,
            Ok(r) => {
                let code = r.status();
                let body = r.text().await.unwrap_or_default();
                SyncOutcome::Failed(format!("server returned {code}: {body}"))
            }
            Err(e) => SyncOutcome::Failed(format!("request failed: {e}")),
        }
    }

    /// Ask the server which of the given record UUIDs it already has, for a
    /// tenant+device. Used by nightly reconciliation to find gaps.
    pub async fn reconcile_missing(
        &self,
        tenant_id: &str,
        device_id: &str,
        entity_type: &str,
        uuids: &[String],
    ) -> Result<Vec<String>, String> {
        let url = format!("{}/sync/reconcile", self.base_url);
        let body = serde_json::json!({
            "tenant_id": tenant_id,
            "device_id": device_id,
            "entity_type": entity_type,
            "uuids": uuids,
        });
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.device_token)
            .json(&body)
            .send()
            .await
            .map_err(|e| format!("reconcile request failed: {e}"))?;

        if !resp.status().is_success() {
            return Err(format!("reconcile returned {}", resp.status()));
        }
        let parsed: ReconcileResponse = resp
            .json()
            .await
            .map_err(|e| format!("reconcile decode failed: {e}"))?;
        Ok(parsed.missing)
    }

    /// Upload a point-in-time snapshot file (VACUUM INTO output) as a coarse backup.
    pub async fn upload_snapshot(
        &self,
        tenant_id: &str,
        device_id: &str,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<(), String> {
        let url = format!("{}/sync/snapshot", self.base_url);
        let resp = self
            .http
            .post(&url)
            .bearer_auth(&self.device_token)
            .header("X-Tenant-Id", tenant_id)
            .header("X-Device-Id", device_id)
            .header("X-Snapshot-Name", filename)
            .header("Content-Type", "application/octet-stream")
            .body(bytes)
            .send()
            .await
            .map_err(|e| format!("snapshot upload failed: {e}"))?;
        if resp.status().is_success() {
            Ok(())
        } else {
            Err(format!("snapshot upload returned {}", resp.status()))
        }
    }
}

#[derive(serde::Deserialize)]
struct ReconcileResponse {
    /// UUIDs the client has that the server is missing.
    missing: Vec<String>,
}
