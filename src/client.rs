use serde::{Deserialize, Serialize};
use smart_default::SmartDefault;
use std::{collections::HashMap, str::FromStr, time::Duration};
use strfmt::strfmt;
use url::Url;

use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use reqwest::{
    header::{HeaderMap, HeaderName, HeaderValue},
    RequestBuilder,
};
use snafu::prelude::*;
use strum::{AsRefStr, Display, EnumString};
type HmacMd5 = Hmac<Md5>;

#[derive(Debug, Copy, Clone, AsRefStr, Display, EnumString, PartialEq, SmartDefault)]
pub enum SeewoSignType {
    #[default]
    #[strum(serialize = "hmac")]
    Hmac,
    #[strum(serialize = "md5")]
    Md5,
}
pub use reqwest::Method as SeewoHttpMethod;
use tracing::debug;
pub type KvPairs = HashMap<String, String>;

fn to_header_map(pairs: &KvPairs) -> Result<HeaderMap, SeewoError> {
    let mut ret = HeaderMap::new();
    for (name, value) in pairs {
        let name = HeaderName::from_str(name).context(InvalidHeaderNameSnafu {
            name: name.to_string(),
            value: value.to_string(),
        })?;
        let value = HeaderValue::from_str(value.as_str()).context(InvalidHeaderValueSnafu {
            name: name.to_string(),
            value: value.to_string(),
        })?;
        ret.append(name, value);
    }
    Ok(ret)
}

#[derive(Debug, Snafu)]
pub enum SeewoError {
    #[snafu(display("send request error"))]
    ClientError { source: reqwest::Error },
    #[snafu(display("build uri error"))]
    BuildUriError { source: strfmt::FmtError },
    #[snafu(display("build uri2 error"))]
    BuildUri2Error { source: url::ParseError },
    #[snafu(display("build request error"))]
    BuildRequestError { source: reqwest::Error },
    #[snafu(display("response error"))]
    ResponseError { source: reqwest::Error },
    #[snafu(display("json error"))]
    JsonError { source: serde_json::Error },
    #[snafu(display("invalid header name, name={name}, value={value}"))]
    InvalidHeaderNameError {
        name: String,
        value: String,
        source: reqwest::header::InvalidHeaderName,
    },
    #[snafu(display("invalid header value, name={name}, value={value}"))]
    InvalidHeaderValueError {
        name: String,
        value: String,
        source: reqwest::header::InvalidHeaderValue,
    },
    #[snafu(display("response status_code error"))]
    ResponseStatusCodeError { status_code: u16 },
}

#[derive(Debug, Default)]
pub struct SeewoClient {
    pub app_id: String,
    pub app_secret: String,
    pub sign_type: SeewoSignType,
    pub stage: SeewoStage,
}

impl SeewoClient {
    pub fn new(app_id: String, app_secret: String) -> Self {
        Self {
            app_id,
            app_secret,
            ..Default::default()
        }
    }

    pub fn get_host(&self) -> &'static str {
        // 沙箱环境 http(s)://openapi.test.seewo.com/live/resource/v1/videos
        // 正式环境 http(s)://openapi.seewo.com/live/resource/v1/videos
        match self.stage {
            SeewoStage::Development => "https://openapi.test.seewo.com",
            SeewoStage::Production => "https://openapi.seewo.com",
        }
    }

    pub fn from_env() -> Self {
        let app_id = std::env::var("SEEWO_APP_ID").expect("SEEWO_APP_ID?");
        let app_secret = std::env::var("SEEWO_APP_SECRET").expect("SEEWO_APP_SECRET?");
        let sign_type = std::env::var("SEEWO_SIGN_TYPE")
            .ok()
            .and_then(|it| SeewoSignType::from_str(&it).ok())
            .unwrap_or_default();
        let stage = std::env::var("SEEWO_STAGE")
            .ok()
            .and_then(|it| SeewoStage::from_str(&it).ok())
            .unwrap_or_default();
        Self {
            app_id,
            app_secret,
            sign_type,
            stage,
        }
    }

    pub async fn invoke(&self, request: SeewoRequest) -> Result<SeewoResponse, SeewoError> {
        let req = request.build_request(self)?;
        let ret = req.send().await.context(ClientSnafu)?;
        if !ret.status().is_success() {
            return Err(SeewoError::ResponseStatusCodeError {
                status_code: ret.status().as_u16(),
            });
        }
        let request_id = ret
            .headers()
            .get("x-sw-req-id")
            .and_then(|it| it.to_str().ok())
            .map(ToOwned::to_owned);
        let message = ret
            .headers()
            .get("x-sw-message")
            .and_then(|it| it.to_str().ok())
            .map(ToOwned::to_owned);
        let ret = ret.text().await.context(ResponseSnafu)?;
        let response_body: serde_json::Value = serde_json::from_str(&ret).context(JsonSnafu)?;
        let ret = SeewoResponse {
            request_id,
            message,
            body: response_body,
        };
        debug!(?ret);
        Ok(ret)
    }
}

#[derive(Debug, Copy, Clone, SmartDefault, AsRefStr, Display, EnumString, PartialEq)]
pub enum SeewoStage {
    Development,

    #[default]
    Production,
}

#[derive(Debug, Clone, SmartDefault)]
pub struct SeewoRequest {
    #[default(SeewoHttpMethod::GET)]
    pub method: SeewoHttpMethod,
    pub uri: String,
    // variables for uri template
    pub vars: KvPairs,
    pub queries: KvPairs,
    pub headers: KvPairs,
    pub body: Option<Vec<u8>>,
}

impl SeewoRequest {
    pub fn get_uri(&self) -> Result<String, SeewoError> {
        strfmt(&self.uri, &self.vars).context(BuildUriSnafu)
    }
    pub fn build_sw_headers(&self, config: &SeewoClient) -> Result<KvPairs, SeewoError> {
        let uri = self.get_uri()?;

        let now = std::time::SystemTime::now();
        let now = now.duration_since(std::time::UNIX_EPOCH).unwrap();
        let now_in_millis = format!("{}", now.as_millis());

        let mut sw_headers = KvPairs::new();
        sw_headers.insert("x-sw-app-id".to_owned(), config.app_id.clone());
        sw_headers.insert("x-sw-req-path".to_owned(), uri);
        sw_headers.insert("x-sw-version".to_owned(), "2".to_owned());
        sw_headers.insert("x-sw-timestamp".to_owned(), now_in_millis);

        let header_names = self.headers.keys().map(AsRef::as_ref).collect::<Vec<_>>();
        let sign_headers = header_names.join(",");
        if !sign_headers.is_empty() {
            sw_headers.insert("x-sw-sign-headers".to_owned(), sign_headers);
        }
        sw_headers.insert(
            "x-sw-sign-type".to_owned(),
            config.sign_type.as_ref().to_owned(),
        );

        if let Some(body) = &self.body {
            let mut hasher = Md5::new();
            hasher.update(body);
            let body_md5 = hasher.finalize();
            let body_md5hex = format!("{:X}", body_md5);
            sw_headers.insert("x-sw-content-md5".to_owned(), body_md5hex);
        }

        let sign = self.sign(&sw_headers, config);
        sw_headers.insert("x-sw-sign".to_owned(), sign);
        Ok(sw_headers)
    }

    fn sign(&self, sw_headers: &KvPairs, config: &SeewoClient) -> String {
        let mut params = Vec::<(String, String)>::new();
        for (k, v) in self.queries.clone() {
            params.push((k, v));
        }
        for (k, v) in self.headers.clone() {
            params.push((k, v));
        }
        for (k, v) in sw_headers.clone() {
            params.push((k, v));
        }
        params.sort_by(|a, b| a.0.cmp(&b.0));

        let secret = &config.app_secret;
        let sign_type = &config.sign_type;

        let mut query = String::new();
        if sign_type == &SeewoSignType::Md5 {
            query.push_str(secret);
        }
        for (key, value) in &params {
            if !value.is_empty() {
                query.push_str(key);
                query.push_str(value);
            }
        }
        if sign_type == &SeewoSignType::Md5 {
            query.push_str(secret);
        }
        let ret = match sign_type {
            SeewoSignType::Hmac => {
                let mut mac = HmacMd5::new_from_slice(secret.as_bytes()).expect("secret?");
                mac.update(query.as_bytes());
                let ret = mac.finalize();
                let ret = ret.into_bytes();
                format!("{:X}", ret)
            }
            SeewoSignType::Md5 => {
                let mut hasher = Md5::new();
                hasher.update(query.as_bytes());
                let ret = hasher.finalize();
                format!("{:X}", ret)
            }
        };
        ret
    }

    pub fn build_request(&self, config: &SeewoClient) -> Result<RequestBuilder, SeewoError> {
        let sw_headers = self.build_sw_headers(config)?;

        let mut headers = self.headers.clone();
        // let common_headers = KvPairs::new();
        // common_headers.insert(
        //     "Accept".to_owned(),
        //     "text/xml,text/javascript,text/html,application/json".to_owned(),
        // );
        // common_headers.insert(
        //     "User-Agent".to_owned(),
        //     "x-sw-client- 0.0.1 - rust".to_owned(),
        // );
        // headers.extend(common_headers);

        headers.extend(sw_headers);

        let header_map = to_header_map(&headers)?;

        // debug!(?request.method, ?request.url, ?header_map);
        let client = reqwest::Client::new();
        let uri = self.get_uri()?;
        let uri_with_host = format!("{}{}", config.get_host(), uri);
        let mut url = Url::parse(&uri_with_host).context(BuildUri2Snafu)?;
        for (k, v) in &self.queries {
            url.query_pairs_mut().append_pair(k, v);
        }
        // todo: timeout as optional config
        let method = self.method.clone();
        let mut client = client
            .request(method, url)
            .headers(header_map)
            .timeout(Duration::from_secs(15));
        if let Some(body) = &self.body {
            client = client
                .header(reqwest::header::CONTENT_TYPE, "application/json")
                .body(body.to_owned());
        }
        Ok(client)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct SeewoResponse {
    pub request_id: Option<String>,
    pub message: Option<String>,
    pub body: serde_json::Value,
}
