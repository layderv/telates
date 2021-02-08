pub struct Client {
    pub client: reqwest::blocking::Client,
}

impl Client {
    pub fn new(timeout: std::time::Duration) -> Client {
        Client {
            client: reqwest::blocking::ClientBuilder::default()
                .timeout(timeout)
                .build()
                .unwrap()
        }
    }

    pub fn client(&mut self) -> reqwest::blocking::Client {
        self.client.to_owned()
    }
}