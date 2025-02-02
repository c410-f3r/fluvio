//!
//! # Create Spu Groups Request
//!
//! Converts Spu Gruups API request into KV request and sends to KV store for processing.
//!

use std::time::Duration;

use tracing::{info, trace, instrument};
use anyhow::{anyhow, Result};

use fluvio_stream_dispatcher::actions::WSAction;
use fluvio_protocol::link::ErrorCode;
use fluvio_sc_schema::Status;
use fluvio_sc_schema::objects::{CreateRequest};
use fluvio_sc_schema::spg::SpuGroupSpec;
use fluvio_controlplane_metadata::extended::SpecExt;
use fluvio_auth::{AuthContext, TypeAction};

use crate::core::Context;
use crate::services::auth::AuthServiceContext;

const DEFAULT_SPG_CREATE_TIMEOUT: u32 = 120 * 1000; // 2 minutes

/// Handler for spu groups request
#[instrument(skip(req, auth_ctx))]
pub async fn handle_create_spu_group_request<AC: AuthContext>(
    req: CreateRequest<SpuGroupSpec>,
    auth_ctx: &AuthServiceContext<AC>,
) -> Result<Status> {
    let (create, spg) = req.parts();
    let name = create.name;

    info!( spg = %name,
         replica = %spg.replicas,
         "creating spg");

    if let Ok(authorized) = auth_ctx
        .auth
        .allow_type_action(SpuGroupSpec::OBJECT_TYPE, TypeAction::Create)
        .await
    {
        if !authorized {
            trace!("authorization failed");
            return Ok(Status::new(
                name.clone(),
                ErrorCode::PermissionDenied,
                Some(String::from("permission denied")),
            ));
        }
    } else {
        return Err(anyhow!("authorization io error"));
    }

    let status = process_custom_spu_request(&auth_ctx.global_ctx, name, create.timeout, spg).await;
    trace!("create spu-group response {:#?}", status);

    Ok(status)
}

/// Process custom spu, converts spu spec to K8 and sends to KV store
#[instrument(skip(ctx, spg_spec))]
async fn process_custom_spu_request(
    ctx: &Context,
    name: String,
    timeout: Option<u32>,
    spg_spec: SpuGroupSpec,
) -> Status {
    if let Err(err) = ctx
        .spgs()
        .wait_action_with_timeout(
            &name,
            WSAction::UpdateSpec((name.clone(), spg_spec)),
            Duration::from_millis(timeout.unwrap_or(DEFAULT_SPG_CREATE_TIMEOUT) as u64),
        )
        .await
    {
        let error = Some(err.to_string());
        Status::new(name, ErrorCode::SpuError, error)
    } else {
        info!(%name, "spg created");
        Status::new_ok(name.clone())
    }
}
