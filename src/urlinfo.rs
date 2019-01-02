use regex::Regex;
use reqwest::header::CONTENT_TYPE;
use select::document::Document;
use select::predicate::Name;
use std::error::Error as StdError;

fn formaterr(e: reqwest::Error) -> String {
    if e.is_redirect() {
        return "[redirect loop]".to_string();
    }

    if let Some(err) = e.get_ref().and_then(|e| e.downcast_ref::<hyper::Error>()) {
        if err.is_connect() {
            let re = r"1416F086.*unable to get local issuer cert";
            if Regex::new(re)
                .unwrap()
                .is_match(&err.cause().unwrap().to_string())
            {
                return format!("[openssl:{}]", re);
            }
            // TODO: Wow, that was ugly! Should get hold of the
            // openssl::error::Error to run code() or so on. A failed attempt
            // follows, err is already a std::error::Error ?!
            //
            //     use openssl::error::Error as OpensslError;
            //     if let Some(sslerr) = err.cause2().unwrap().downcast_ref::<OpensslError>() {
            //         return format!("code:{}", sslerr.code());
            //     }
        }
    }
    return format!("reqwest::get(): {}", e);
}

pub fn urlinfo(url: &str) -> String {
    let client = reqwest::Client::builder()
        .proxy(reqwest::Proxy::http("http://127.0.0.1:9080").unwrap())
        .build();
    let resp = match client.unwrap().get(url).send() {
        Ok(resp) => resp,
        Err(e) => return formaterr(e),
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
                    Some(child) => format!("`{}`", child.text().trim()),
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
