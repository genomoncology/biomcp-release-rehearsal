use super::body;

pub(super) fn live_body(request: &str) -> Result<String, String> {
    let path = request.split_whitespace().nth(1).ok_or("request path")?;
    let page = path
        .split(['?', '&'])
        .find_map(|pair| pair.strip_prefix("page="))
        .ok_or("request page")?
        .parse()
        .map_err(|_| "request page number")?;
    Ok(body(&format!("https://www.ebi.ac.uk{path}"), page, 6).to_string())
}
