#[napi(object)]
#[derive(Debug)]
pub struct AouResponse {
  pub status: Option<u32>,
  #[napi(ts_type = "Record<String,String>")]
  pub headers: Option<serde_json::Value>,
  pub body: Option<serde_json::Value>,
}
