use std::{net::SocketAddr, path::PathBuf};
pub fn data_dir() -> Result<PathBuf, String> {
    let dir = std::env::var_os("SIPIBRAIN_DATA_DIR")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("LOCALAPPDATA")
                .map(|root| PathBuf::from(root).join("SIPECOM/SIPI-brain/pilot"))
        })
        .ok_or("Set SIPIBRAIN_DATA_DIR to a private local directory.")?;
    if dir.to_string_lossy().starts_with("\\\\") {
        return Err("Data directory must be local.".into());
    }
    Ok(dir)
}
pub fn validate_bind(bind: SocketAddr) -> Result<(), String> {
    if !bind.ip().is_loopback() {
        return Err("Only loopback bind addresses are permitted.".into());
    }
    Ok(())
}
pub fn origin(bind: SocketAddr) -> Result<String, String> {
    let origin = std::env::var("SIPIBRAIN_ORIGIN").unwrap_or_else(|_| format!("http://{bind}"));
    let uri: axum::http::Uri = origin.parse().map_err(|_| "Invalid application origin.")?;
    if !matches!(uri.scheme_str(), Some("http" | "https"))
        || !matches!(
            uri.host(),
            Some("127.0.0.1" | "[::1]" | "::1" | "localhost")
        )
        || uri.path_and_query().is_some_and(|p| p.as_str() != "/")
        || origin.ends_with('/')
        || uri.authority().is_some_and(|a| a.as_str().contains('@'))
    {
        return Err(
            "Application origin must be an exact local http(s) origin, without a trailing slash."
                .into(),
        );
    }
    Ok(origin)
}
