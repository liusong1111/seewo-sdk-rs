use serde::{Deserialize, Serialize};

use crate::{
    client::{KvPairs, SeewoResponse},
    SeewoClient, SeewoError, SeewoHttpMethod, SeewoRequest,
};

pub enum SeewoTypedRequestBody<T: serde::Serialize + Sized> {
    Json(T),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeewoTypedResponse<T> {
    pub request_id: Option<String>,
    pub message: Option<String>,
    pub body: T,
}

impl<T> TryFrom<SeewoResponse> for SeewoTypedResponse<T>
where
    T: serde::de::DeserializeOwned,
{
    type Error = SeewoError;

    fn try_from(value: SeewoResponse) -> Result<Self, Self::Error> {
        let SeewoResponse {
            request_id,
            message,
            body,
        } = value;
        let body: T =
            serde_json::from_value(body).map_err(|err| SeewoError::JsonError { source: err })?;
        let ret = Self {
            request_id,
            message,
            body,
        };
        Ok(ret)
    }
}

impl<T: serde::Serialize + Sized> From<SeewoTypedRequestBody<T>> for Option<Vec<u8>> {
    fn from(r: SeewoTypedRequestBody<T>) -> Self {
        match r {
            SeewoTypedRequestBody::Json(body) => serde_json::to_vec(&body).ok(),
        }
    }
}

#[async_trait::async_trait]
pub trait SeewoTypedRequest<'a>: Sized {
    type Response: serde::de::DeserializeOwned;
    const METHOD: SeewoHttpMethod;
    const URI: &'static str;

    fn into_request(self) -> SeewoRequest {
        let method = Self::METHOD;
        let uri = Self::URI.to_string();
        let vars = self.vars();
        let queries = self.queries();
        let headers = self.headers();
        let body = self.body();
        SeewoRequest {
            method,
            uri,
            vars,
            headers,
            queries,
            body,
        }
    }
    fn vars(&self) -> KvPairs {
        KvPairs::new()
    }
    fn queries(&self) -> KvPairs {
        KvPairs::new()
    }
    fn headers(&self) -> KvPairs {
        KvPairs::new()
    }
    fn body(self) -> Option<Vec<u8>> {
        None
    }

    async fn invoke(
        self,
        client: &SeewoClient,
    ) -> Result<SeewoTypedResponse<Self::Response>, SeewoError> {
        let request = self.into_request();
        let response = client.invoke(request).await?;
        response.try_into()
    }
}
