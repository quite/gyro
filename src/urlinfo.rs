use htmlescape::decode_html;
use regex::Regex;
use reqwest::header::CONTENT_TYPE;
use std::io::Read;
use std::time::Duration;

const MAXBYTES: u64 = 30 * 1024;
const TIMEOUTSECS: u64 = 8;

fn get_title(contents: &str) -> Result<String, String> {
    let re = Regex::new("<(?i:title).*?>((.|\n)*?)</(?i:title)>").unwrap();
    let title = match re.captures(contents) {
        Some(cap) => cap.get(1).unwrap().as_str(),
        None => return Err("[no title tag]".to_string()),
    };
    match decode_html(title) {
        Ok(decoded) => Ok(decoded.trim().to_string()),
        Err(_) => Err("html entity error in title tag".to_string()),
    }
}

fn formaterr(e: reqwest::Error) -> String {
    if e.is_redirect() {
        return "[redirect loop]".to_string();
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
            return format!("[certificate error: {}]", err);
        }
    }

    return format!("[{}]", e);
}

pub fn urlinfo(url: &str) -> String {
    let client = reqwest::Client::builder()
        .timeout(Duration::from_secs(TIMEOUTSECS))
        .proxy(reqwest::Proxy::all("http://127.0.0.1:8118").unwrap())
        .build()
        .unwrap();
    let resp = match client.get(url).send() {
        Ok(resp) => resp,
        Err(e) => return formaterr(e),
    };

    if !resp.status().is_success() {
        return format!("[http error: {}]", resp.status().to_string());
    }

    let headers = resp.headers().clone();

    match headers.get(CONTENT_TYPE).and_then(|t| t.to_str().ok()) {
        Some(i) if i.contains("text/html") || i.contains("application/xhtml+xml") => {
            let mut buf = Vec::new();
            if resp.take(MAXBYTES).read_to_end(&mut buf).is_err() {
                return "[read failed]".to_string();
            }
            let contents = String::from_utf8_lossy(&buf);
            match get_title(&contents) {
                Ok(title) => format!("`{}`", title),
                Err(msg) => format!("[{}]", msg),
            }
        }
        // just content type
        Some(i) => i.to_string(),
        None => "[no content-type]".to_string(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_urlinfo() {
        assert_eq!(
            Some(String::from("Welcome to nginx!")),
            urlinfo("http://localhost")
        );
    }
}
