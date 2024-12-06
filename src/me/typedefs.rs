use serde::Serialize;

#[derive(Serialize)]
pub struct InnerJson<'a> {
    uuid: &'a str,
}

#[derive(Serialize)]
pub struct OuterJson<'a> {
    json: InnerJson<'a>,
}

#[derive(Serialize)]
pub struct RootJson<'a> {
    #[serde(rename = "0")]
    inner: OuterJson<'a>,
}

impl<'a> RootJson<'a> {
    pub fn to_string(uuid: &'a str) -> eyre::Result<String, serde_json::Error> {
        let query = Self {
            inner: OuterJson {
                json: InnerJson { uuid },
            },
        };

        serde_json::to_string(&query)
    }
}
