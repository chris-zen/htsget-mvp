use std::collections::HashMap;
use std::sync::Arc;
use lambda_http::IntoResponse;
use htsget_http_core::{Endpoint, get_response_for_get_request};
use htsget_search::htsget::{HtsGet};
use crate::handlers::handle_response;

/// GET request reads endpoint
pub async fn reads<H: HtsGet + Send + Sync + 'static>(
  id_path: String,
  searcher: Arc<H>,
  mut query: HashMap<String, String>,
  endpoint: Endpoint
) -> impl IntoResponse {
  query.insert("id".to_string(), id_path);
  handle_response(
    get_response_for_get_request(
      searcher,
      query,
      endpoint,
    ).await,
  )
}
