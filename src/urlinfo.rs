use reqwest::header::CONTENT_TYPE;
use select::document::Document;
use select::predicate::Name;

pub fn urlinfo(url: &str) -> String {
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::http("http://127.0.0.1:9080").unwrap())
        .build();
    let resp = match client.unwrap().get(url).send() {
        Ok(resp) => resp,
        Err(e) => return format!("reqwest::get(): {}", e),
    };

    if !resp.status().is_success() {
        return format!("fail: {}", resp.status().to_string());
    }

    let headers = resp.headers().clone();

    match headers.get(CONTENT_TYPE).and_then(|t| t.to_str().ok()) {
        Some(i)
            if i.contains("text/html")
                || i.contains("application/xhtml+xml") =>
        {
            match Document::from_read(resp)
                .unwrap()
                .find(Name("title"))
                .nth(0)
            {
                Some(title) => match title.children().next() {
                    Some(child) => format!("`{}`", child.text()),
                    None => "[title tag is empty]".to_string(),
                },
                None => "[title tag is missing]".to_string(),
            }
        }
        // just content type
        Some(i) => i.to_string(),
        None => "[content-type is missing]".to_string(),
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
