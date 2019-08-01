use htmlescape::decode_html;
use regex::Regex;
use reqwest::header::CONTENT_TYPE;
use std::collections::HashMap;
use std::io::Read;
use std::time::Duration;

const MAX_TITLE_TAKE: u64 = 15 * 1024;

fn extract_html_title(contents: &str) -> Result<String, String> {
    let re = Regex::new("<(?i:title).*?>((.|\n)*?)</(?i:title)>").unwrap();
    let title = match re.captures(contents) {
        Some(cap) => cap.get(1).unwrap().as_str(),
        None => return Err(format!("no title tag ({} KiB read)", MAX_TITLE_TAKE / 1024)),
    };
    match decode_html(title) {
        Ok(decoded) => Ok(decoded.trim().replace("\n", " ")),
        Err(_) => Err("html entity error in title tag".to_string()),
    }
}

fn get_title(resp: reqwest::Response) -> Result<String, String> {
    let headers = resp.headers().clone();

    match headers.get(CONTENT_TYPE).and_then(|t| t.to_str().ok()) {
        Some(i) if i.contains("text/html") || i.contains("application/xhtml+xml") => {
            let mut buf = Vec::new();
            if resp.take(MAX_TITLE_TAKE).read_to_end(&mut buf).is_err() {
                return Err("read failed".to_string());
            }
            let contents = String::from_utf8_lossy(&buf);
            extract_html_title(&contents)
        }
        // just content type
        Some(i) => Err(i.to_string()),
        None => Err("no content-type".to_string()),
    }
}

fn get_wp_extract(resp: reqwest::Response) -> Result<String, String> {
    let mut buf = Vec::new();
    if resp.take(10 * 1024).read_to_end(&mut buf).is_err() {
        return Err("read failed".to_string());
    }
    let contents = String::from_utf8_lossy(&buf);

    let v: serde_json::Value = match serde_json::from_str(&contents) {
        Ok(v) => v,
        Err(e) => return Err(format!("wikipedia api err: {}", e.to_string())),
    };

    match &v["extract"] {
        serde_json::Value::String(extract) => Ok(extract.replace("\n", ", ")),
        _ => Err("wikipedia json: no extract".to_string()),
    }
}

fn formaterr(e: reqwest::Error) -> String {
    if e.is_redirect() {
        return "redirect loop".to_string();
    }

    // Boy, it's ugly
    const ERRS: &[&str] = &[
        "unable to get local issuer",
        "certificate has expired",
        "self signed certificate",
        "Hostname mismatch",
    ];
    for err in ERRS.iter() {
        if e.to_string().contains(err) {
            return format!("certificate error: {}", err);
        }
    }

    return format!("{}", e);
}

fn fetch(options: &HashMap<String, String>, url: &str) -> Result<reqwest::Response, String> {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(
            options.get("timeout").unwrap().parse().unwrap(),
        ))
        .proxy(reqwest::Proxy::all(options.get("proxy").unwrap()).unwrap())
        .build()
        .unwrap();

    let resp = match client.get(url).send() {
        Ok(resp) => resp,
        Err(e) => return Err(formaterr(e)),
    };

    if !resp.status().is_success() {
        return Err(format!("http error: {}", resp.status()));
    }

    Ok(resp)
}

pub fn urlinfo(options: &HashMap<String, String>, url: &str) -> Result<String, String> {
    let re = Regex::new(r"^https?://([-a-z]+\.(?:m\.)?wikipedia\.org)/wiki/(.*)").unwrap();
    if let Some(cap) = re.captures(url) {
        let url = format!(
            "https://{}/api/rest_v1/page/summary/{}",
            cap.get(1).unwrap().as_str(),
            cap.get(2).unwrap().as_str(),
        );
        return get_wp_extract(fetch(options, &url)?);
    };
    get_title(fetch(options, url)?)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_html_title() {
        assert_eq!(
            Ok("bar".to_string()),
            extract_html_title("foo<title>bar</title>baz")
        );
        assert_eq!(
            Ok("bar baz".to_string()),
            extract_html_title("foo<title>bar\nbaz</title>blubb")
        );
    }
}
