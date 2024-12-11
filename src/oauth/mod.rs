pub mod google;

pub mod types {
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Serialize)]
    pub struct Links {
        pub google: String,
    }

    #[derive(Debug, Deserialize)]
    pub struct GoogleOauthCallback {
        pub code: String,
        pub state: String,
    }
}
