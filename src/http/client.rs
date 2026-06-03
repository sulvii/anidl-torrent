use anyhow::Result;
use reqwest::{
    Client,
    header::{HeaderMap, HeaderValue},
};
use std::time::Duration;

pub struct HttpClient {
    pub inner: Client,
}

impl HttpClient {
    pub fn new() -> Result<Self> {
        let mut headers = HeaderMap::new();

        headers.insert("upgrade-insecure-requests", HeaderValue::from_static("1"));
        headers.insert("Referer", HeaderValue::from_static("https://anidb.net"));
        headers.insert(
            "Cookie",
            HeaderValue::from_static("adbuin=1768286160-sLzV;cf_clearance=2.bygVlV4mNzDoO_xrpgdbx_RV8kxMZeBNsQwf8R1ds-1780460673-1.2.1.1-nCRwl_cmL8g6557zIk4q13uyt8eAvEuF1o0GNkzImXBAVPo8aTfUR1TQMxjeM_91MV3hSZ94og.rV8dqCxRlqhKOuM861GuUhZOPxyGOmcuC5YMQUC4VlRhQ_TXJOZ3hbpfi0AWytHE.1kNXsUAL3SUmnrOS3zr89y8KXkogJzCs54MYmVdWMyerX9YIEUL.cNRJJNzmZ8Y5WA5f6Hjf0Yu0Z3iMBbyFlP6UDO6.BITyPVAtSjG1w.wkEnqToObc156YPYd0M2Eq.HFnnE3AbbN2gG53NHqrG2A8NW9pPRqsN4YkXBGUc84xj1NXTfT7ctNIkEil5Fj5H8iS_VFFtGuJQs4XG2dJs630dvsZd3HnkLb9USFcjp3n319FhiRr90oaEqXbcH5C6z88t2qAWDSS746YOHsEJWy3QbH5zo0"),
        );

        let inner = Client::builder()
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/148.0.0.0 Safari/537.36"
            )
            .default_headers(headers)
            .timeout(Duration::from_secs(15))
            .build()?;

        Ok(Self { inner })
    }
}
