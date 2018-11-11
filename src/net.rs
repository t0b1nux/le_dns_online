use curl::easy::{Easy2, Handler, List, WriteError};
pub use crate::config::*;
use std::convert;

impl convert::From<curl::Error> for Error {
    fn from(e: curl::Error) -> Error {
        Error::CurlError(e)
    }
}

impl convert::From<serde_json::Error> for Error {
    fn from(e: serde_json::Error) -> Error {
        Error::SerdeError(e)
    }
}

pub struct Collector(String);

impl Handler for Collector {
    fn write(&mut self, data: &[u8]) -> Result<usize, WriteError> {
        self.0.push_str(std::str::from_utf8(data).unwrap());
        Ok(data.len())
    }
}

pub fn make_query(api_endpoint: &str, auth_token: &str) -> Result<Easy2<Collector>, curl::Error> {
    let mut easy = Easy2::new(Collector(String::with_capacity(4096)));

    let mut url: String = API_URL.into();
    url.push_str(api_endpoint);
    easy.url(&url)?;

    let mut http_headers = List::new();
    let mut auth: String = "Authorization: Bearer ".into();
    auth.push_str(&auth_token);
    http_headers.append(&auth)?;
    easy.http_headers(http_headers)?;
    Ok(easy)
}

// Generate and execute an HTTP query to 'api_endpoint'.
// This function allow you tu provide a function to configure the query (e.g. setting the type of query
// or adding data) and another function to parse the api response
pub fn execute_query<T, F, F2, I: Into<Error>, I2: Into<Error>>(auth_token: &str, api_endpoint: &str, configure: F, parse: F2) -> Result<T, Error>
where F: Fn(Easy2<Collector>) -> Result<Easy2<Collector>, I> + Sized, F2: Fn(&str) -> Result<T, I2> + Sized {
    let req = make_query(api_endpoint, auth_token)?;

    let mut req = match configure(req) {
        Ok(x) => x,
        Err(e) => return Err(e.into())
    };

    req.perform()?;
    let res_code = req.response_code()?;
    if res_code < 200 || res_code >= 400 {
        return Err(Error::ApiError((req.effective_url()?.unwrap_or("").into(), req.response_code()?, req.get_ref().0.clone())));
    }

    match parse(&req.get_ref().0) {
        Ok(x) => Ok(x),
        Err(e) => Err(e.into())
    }
}

pub fn parse_json<T>(data: &str) -> Result<T, serde_json::Error> where for <'de> T: serde::Deserialize<'de> {
    Ok(serde_json::from_str(data)?)
}

pub fn get_data(mut req: Easy2<Collector>) -> Result<Easy2<Collector>, curl::Error> {
    req.get(true)?;
    Ok(req)
}

pub fn patch_data(mut req: Easy2<Collector>) -> Result<Easy2<Collector>, curl::Error> {
    req.custom_request("PATCH")?;
    Ok(req)
}

pub fn delete_data(mut req: Easy2<Collector>) -> Result<Easy2<Collector>, curl::Error> {
    req.custom_request("DELETE")?;
    Ok(req)
}

// PostData(name, value)
pub struct PostData<'a>(pub &'a str, pub &'a str);

pub fn post_data<'a>(data: &'a[PostData<'a>]) -> impl Fn(Easy2<Collector>) -> Result<Easy2<Collector>, curl::Error> + Sized +'a {
    move |mut req: Easy2<Collector>| {
        req.post(true)?;
        let mut post_fields = String::with_capacity(data.len()*25);
        for e in data {
            let entry = format!("{}={}&", req.url_encode(e.0.as_bytes()), req.url_encode(e.1.as_bytes()));
            post_fields.push_str(&entry);
        }
        // delete the last '&' if any
        post_fields.pop();

        req.post_field_size(post_fields.len() as u64)?;
        req.post_fields_copy(post_fields.as_bytes())?;
        Ok(req)
    }
}
