use std::time::Duration;

use maplit::hashmap;
use serde::{Deserialize, Serialize};
use serde_with::{serde_as, skip_serializing_none, DurationSeconds, NoneAsEmptyString};

use crate::typed::{SeewoTypedRequest, SeewoTypedRequestBody};
use crate::SeewoHttpMethod;

//----
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherDeviceV1StreamingStartRequest {
    pub device_sn: String,
    pub stream_url: Option<String>,
    pub biz_id: Option<String>,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherDeviceV1StreamingStartResponse {
    pub code: i64,
    pub message: Option<String>,
}

impl SeewoTypedRequest<'_> for OtherDeviceV1StreamingStartRequest {
    const METHOD: SeewoHttpMethod = SeewoHttpMethod::POST;
    const URI: &'static str = "/live/other-device/v1/streaming/start";
    type Response = OtherDeviceV1StreamingStartResponse;

    fn body(self) -> Option<Vec<u8>> {
        SeewoTypedRequestBody::Json(self).into()
    }
}

//-------------------
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherDeviceV1StreamingStopRequest {
    pub device_sn: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct OtherDeviceV1StreamingStopResponse {
    pub code: i64,
    pub message: Option<String>,
}

impl SeewoTypedRequest<'_> for OtherDeviceV1StreamingStopRequest {
    type Response = OtherDeviceV1StreamingStopResponse;
    const METHOD: SeewoHttpMethod = SeewoHttpMethod::POST;
    const URI: &'static str = "/live/other-device/v1/streaming/stop";

    fn body(self) -> Option<Vec<u8>> {
        SeewoTypedRequestBody::Json(self).into()
    }
}

//-------------------
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingVideosRequest {
    pub biz_id: String,
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingVideosResponse {
    pub code: i64,
    pub message: Option<String>,
    pub data: Vec<StreamingVideo>,
}

#[skip_serializing_none]
#[serde_as]
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StreamingVideo {
    #[serde_as(as = "NoneAsEmptyString")]
    pub name: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub biz_id: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub ftp_path: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub url: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub index: Option<String>,
    #[serde(rename = "durationInSec")]
    #[serde_as(as = "DurationSeconds<u64>")]
    pub duration: Duration,
    pub size_in_kb: u64,
    #[serde_as(as = "NoneAsEmptyString")]
    pub video_group_id: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub video_group_name: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub room_name: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub teacher_name: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub subject_name: Option<String>,
    #[serde_as(as = "NoneAsEmptyString")]
    pub stage_name: Option<String>,
    pub record_timestamp: u64,
}

impl SeewoTypedRequest<'_> for StreamingVideosRequest {
    type Response = StreamingVideosResponse;
    const METHOD: SeewoHttpMethod = SeewoHttpMethod::GET;
    const URI: &'static str = "/live/resource/v1/videos";
    fn queries(&self) -> crate::client::KvPairs {
        hashmap! {
            "bizId".to_owned() => self.biz_id.clone(),
        }
    }
}
