use super::*;

/// Serve one resume scenario after the first transfer was cut.
pub(super) fn answer_resume(
    server: &support::Server,
    request: support::Request<()>,
    scenario: &str,
    cut: usize,
) {
    match scenario {
        "ignored" => request.respond(200, "ETag: \"two\"\r\n", TSV),
        "412" => {
            request.respond(412, "", "");
            let next = server.next();
            assert!(
                !next.text.to_ascii_lowercase().contains("range:"),
                "a refused resume must restart without a range"
            );
            next.respond(200, "ETag: \"two\"\r\n", TSV);
        }
        _ => {
            let etag = if scenario == "changed" { "two" } else { "one" };
            let offset = if scenario == "offset" { cut + 1 } else { cut };
            let total = if scenario == "total" {
                TSV.len() + 1
            } else {
                TSV.len()
            };
            let range = if scenario == "missing" {
                String::new()
            } else {
                format!(
                    "Content-Range: bytes {offset}-{}/{total}\r\n",
                    TSV.len() - 1
                )
            };
            request.respond(206, &format!("ETag: \"{etag}\"\r\n{range}"), &TSV[cut..]);
        }
    }
}
