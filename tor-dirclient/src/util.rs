//! Helper functions for the directory client code

/// Encode an HTTP request in a quick and dirty HTTP 1.0 format.
pub(crate) fn encode_request(req: http::Request<()>) -> String {
    let mut s = format!("{} {} HTTP/1.0\r\n", req.method(), req.uri());

    for (key, val) in req.headers().iter() {
        // XXXX Unwrap is a bit naughty, but should be harmless
        s.push_str(&format!("{}: {}\r\n", key, val.to_str().unwrap()));
    }
    s.push_str("\r\n");
    s
}