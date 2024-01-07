use std::{error::Error, sync::Arc};

use crate::proxy::{Proxy, Request};

pub fn map_proxy_request(
    mut request: Request,
    proxies: Arc<Vec<Proxy>>,
) -> Result<Request, Box<dyn Error>> {
    let proxy = proxies
        .iter()
        .find(|x| request.path.starts_with(&x.local_path))
        .expect("should be a matching proxy for the request");

    request.path = request.path.replace(&proxy.local_path, &proxy.remote_path);

    request.port = proxy.remote_port;

    Ok(request)
}
