#![doc = include_str!("../README.md")]

mod client;
mod recording;
mod typed;

pub use client::{
    SeewoClient, SeewoError, SeewoHttpMethod, SeewoRequest, SeewoSignType, SeewoStage,
};

pub use recording::*;
pub use typed::SeewoTypedRequest;

#[cfg(test)]
mod tests {
    use super::*;
    // use maplit::hashmap;
    // use serde_json::json;
    use tracing::debug;

    #[tokio::test]
    async fn it_works() {
        std::env::set_var("RUST_LOG", "debug");
        tracing_subscriber::fmt::init();
        let biz_id = "abc".to_owned();
        let device_sn = "48SV31V010103823800166".to_owned();
        let client = SeewoClient::from_env();
        let start_request = OtherDeviceV1StreamingStartRequest {
            device_sn: device_sn.clone(),
            stream_url: None,
            biz_id: Some(biz_id.clone()),
        };
        let start_response = start_request.invoke(&client).await.unwrap();
        debug!(?start_response);

        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let stop_request = OtherDeviceV1StreamingStopRequest {
            device_sn: device_sn.clone(),
        };
        let stop_response = stop_request.invoke(&client).await.unwrap();
        debug!(?stop_response);

        tokio::time::sleep(std::time::Duration::from_secs(8)).await;

        let videos_request = StreamingVideosRequest {
            biz_id: biz_id.clone(),
        };
        let videos_response = videos_request.invoke(&client).await.unwrap();
        debug!(?videos_response);
    }
}
