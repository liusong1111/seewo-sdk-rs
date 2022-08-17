# Seewo rust SDK

Seewo(希沃) rust SDK **(unofficial)**

[Seewo Open Platform Official Site](http://open.seewo.com/)

Note: This crate is **very experimental and incomplete**, only 3 interfaces are implemented, and this crate is not pushed to crates.io yet.

If you use rust to call seewo open api, you can fork this repository and implements your own interfaces.

## Usage

```rust
use seewo_sdk::{SeewoClient, SeewoTypedRequest, OtherDeviceV1StreamingStartRequest};

// initiate SeewoClient by reading environment variables: `SEEWO_APP_ID`, `SEEWO_APP_SECRET`.
// or use `SeewoClient::new(app_id, app_secret)` implicitly.
let client = SeewoClient::from_env();

// construct request
let start_request = OtherDeviceV1StreamingStartRequest {
  device_sn: device_sn.clone(),
  stream_url: None,
  biz_id: Some(biz_id.clone()),
};
// send http request to seewo open api server, and get the response.
// returns: Result<SeewoTypedResponse<OtherDeviceV1StreamingStartResponse>, SeewoError>
let start_response = start_request.invoke(&client).await.unwrap();
println!("response.headers['x-sw-req-id'] = {}", start_response.request_id);
// OtherDeviceV1StreamingStartResponse, like {"code": .., "message": ..}
println!("response.body={:?}", start_response.body);
```


