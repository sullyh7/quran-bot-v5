mod api_endpoints {
    pub const POST_TW: &str = "https://api.twitter.com/2/tweets";
}

pub mod quran_csv {

    use std::{error::Error};

    use serde::{de::DeserializeOwned, Deserialize};

    pub struct CsvReader<T> {
        pub t: T,
    }
    impl<T: DeserializeOwned> CsvReader<T> {
        pub fn get(csv_file: String) -> Result<Vec<T>, Box<dyn Error>> {
            let mut vec = vec![];
            let mut rdr = csv::Reader::from_path(csv_file)?;
            for result in rdr.deserialize() {
                let item: T = result?;
                vec.push(item);
            }   
            Ok(vec)
        }   
    }
    // surah_no,surah_name_en,surah_name_ar,surah_name_roman,ayah_no_surah,ayah_no_quran,ayah_ar,ayah_en,ruko_no,juz_no,manzil_no,hizb_quarter,total_ayah_surah,total_ayah_quran,place_of_revelation,sajah_ayah,sajdah_no,no_of_word_ayah,list_of_words

    #[derive(Deserialize)]
    #[derive(Debug)]
    pub struct Verse {
        pub surah_no: String,
        pub surah_name_en: String,
        pub surah_name_ar: String,
        pub surah_name_roman: String,
        pub ayah_no_surah: String,
        pub ayah_no_quran: String,
        pub ayah_ar: String,
        pub ayah_en: String,
        pub ruko_no: String,
        pub juz_no: String,
        pub manzil_no: String,
        pub hizb_quarter: String,
        pub total_ayah_surah: String,
        pub total_ayah_quran: String,
        pub place_of_revelation: String,
        pub sajah_ayah: String,
        pub sajdah_no: String,
        pub no_of_word_ayah: String,
        pub list_of_words: String,
    }

    impl Verse {
        pub fn tafsir_link(&self) -> String {
            format!("https://quran.com/{}:{}/tafsirs/en-tafisr-ibn-kathir", self.surah_no, self.ayah_no_surah)
        }
    }

}

pub mod tw_api {
    use colored::Colorize;
    use dotenv::dotenv;
    use oauth::{Request};
    use oauth_client::authorization_header;
    use reqwest::{
        blocking::Client,
        header::{AUTHORIZATION, CONTENT_TYPE, self},
        StatusCode, Method,
    };
    use serde::{Serialize, Deserialize};
    use std::{
        env,
        error::Error,
        fmt::Display,
        fmt::Debug, collections::HashMap, rc::Rc,
    };
    

    use crate::{api_endpoints, quran_csv::Verse};

    pub struct Credentials {
        pub api_key: String,
        pub api_key_secret: String,
        pub access_token: String,
        pub access_token_secret: String,
        pub client_id: String,
        pub client_secret: String,
    }

    impl Credentials {
        pub fn from_env() -> Result<Self, Box<dyn Error>> {
            match dotenv() {
                Ok(_) => {
                    let api_key = env::var("APIKey")?;
                    let api_key_secret = env::var("APIKeySecret")?;
                    let access_token = env::var("AccessToken")?;
                    let access_token_secret = env::var("AccessTokenSecret")?;
                    let client_id = env::var("ClientId")?;
                    let client_secret = env::var("ClientSecret")?;
                    Ok(Self {
                        api_key,
                        api_key_secret,
                        access_token,
                        access_token_secret,
                        client_id,
                        client_secret,
                    })
                }
                Err(err) => Err(Box::new(err)),
            }
        }
    }

    pub struct TwClient {
        creds: Credentials,
        client: reqwest::blocking::Client,
    }

    impl TwClient {
        pub fn new(creds: Credentials) -> Self {
            Self {
                creds,
                client: reqwest::blocking::Client::new(),
            }
        }

        pub fn send<T: Tweetable + Request + serde::ser::Serialize + Debug>(
            &self,
            tweet: T,
        ) -> Result<(), Box<dyn Error>> {
            // let tweet_body = tweet.fmt_tweet();
            // let token = Token::from_parts(
            //     &self.creds.api_key,
            //     &self.creds.api_key_secret,
            //     &self.creds.access_token,
            //     &self.creds.access_token_secret,
            // );
            // let auth_header = oauth::post(ApiEndpoints::POST_TW, &tweet, &token, HMAC_SHA1);

            let res = self.send_req(tweet.fmt_tweet())?;
            let reply = self.send_req(tweet.fmt_tweet_tr(res.data.id));
            match reply {
                Ok(_) => Ok(()),
                Err(err) => Err(Box::new(TweetError(format!("{}", err)))),
            }
        }

        fn send_req<T: Request + serde::ser::Serialize + Debug>(&self, tweet: T) -> Result<TweetResponse, Box<dyn Error>>{
            // let auth_header = format!("OAuth oauth_consumer_key=\"{}\",oauth_token=\"{}\",oauth_signature_method=\"HMAC-SHA1\",oauth_timestamp=\"1683217809\",oauth_nonce=\"vvpVQCyJaKz\",oauth_version=\"1.0\",oauth_signature=\"UPxsH0DH0FdtBEZYqBBg27S3IT8%3D\"",
            // self.creds.api_key, self.creds.access_token);
            let creds = oauth1_header::Credentials::new(
                &self.creds.api_key,
                &self.creds.api_key_secret,
                &self.creds.access_token,
                &self.creds.access_token_secret,
            );
            let header_value = creds.auth(&Method::POST, api_endpoints::POST_TW, &HashMap::new());

            let req = self
                .client
                .post(api_endpoints::POST_TW)
                .header(AUTHORIZATION, header_value)
                .header(CONTENT_TYPE, "application/json")
                .json(&tweet)
                .build()?;

        let resp = Client::execute(&self.client, req)?;

        let status = resp.status();

        match status {
            StatusCode::CREATED => {
                let r = resp.json::<TweetResponse>()?;
                return Ok(r);
            },
            StatusCode::UNAUTHORIZED => {
                match resp.json::<GeneralErrorResponse>() {
                    Ok(err_resp) => Err(Box::new(TweetError(format!("Auth error: {}, {}", status.to_string(), err_resp.detail).color("red").to_string()))),
                    Err(e) => Err(Box::new(TweetError(format!("Auth error: {}", status.to_string()).color("red").to_string()))),
                }
            },
            _ => {
                match resp.json::<OtherErrorResponse>() {
                    Ok(a) => return Err(Box::new(TweetError(format!("Other Error: {}: {}", status.to_string(), a.detail)))),
                    Err(_) => Err(Box::new(TweetError(format!("Other error: {}", status.to_string()).color("red").to_string()))) 
                }
            },
        }

        }
    }

    pub trait Tweetable {
        fn fmt_tweet(&self) -> Tweet;
        fn fmt_tweet_tr(&self, reply_id: String) -> Reply;
    }

    #[derive(Serialize, Debug, Deserialize, Request)]
    pub struct Tweet {
        pub text: String,
    }
    #[derive(Serialize, Debug, Deserialize, Request)]
    pub struct Reply {
        pub text: String,
        pub reply: ReplyId,
    }

    #[derive(Serialize, Debug, Deserialize, Request)]
    pub struct ReplyId {
        in_reply_to_tweet_id: String,
    }
    
    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct TweetResponse {
        pub data: TweetResponseData,
    }

    #[derive(Default, Debug, Clone, PartialEq, Serialize, Deserialize)]
    pub struct TweetResponseData {
        id: String,
        text: String,
    }

    #[derive(oauth::Request, Serialize, Debug)]

    pub struct QuranTweet<'a> {
        pub verse_en: &'a str,
        pub verse_ar: &'a str,
        pub verse_num: String,
        pub tafsir: String,
    }

    impl<'a> QuranTweet<'a> {
        pub fn new(verse_en: &'a str, verse_ar: &'a str, verse_num: String, tafsir: String) -> Self {
            Self {
                verse_en,
                verse_ar,
                verse_num,
                tafsir,
            }
        }

        pub fn from(v: &'a Verse) -> Self{
            let verse_num = format!("{}-{}", &v.surah_no, &v.ayah_no_surah);
            let link = &v.tafsir_link();
            QuranTweet::new(
                &v.ayah_en,
                &v.ayah_ar,
                verse_num,
                link.to_owned(),
            )
        }
    }

    impl Tweetable for QuranTweet<'_>{
        fn fmt_tweet(&self) -> Tweet {
            let text = format!(
                "{}",
                self.verse_ar
            );
            // if text.chars().count() > 200 {
            //     text = format!(
            //         "{}\n{}\nTafsir: {}",
            //         self.verse_en, self.verse_num, self.tafsir
            //     ); 
            // }
            Tweet {
                text
            }
        }
        fn fmt_tweet_tr(&self, reply_id: String) -> Reply {
            let text = format!(
                "{}\n{}\nTafsir: {}",
                self.verse_en, self.verse_num, self.tafsir
            );
            Reply {
                text,
                reply: ReplyId { in_reply_to_tweet_id: reply_id}
            }
        }
    }
    #[derive(Debug)]
    pub struct TweetError(String);


    #[derive(Debug, Deserialize, Serialize)]
    struct OtherErrorResponse {
        pub errors: Vec<ErrorResponseElement>,
        pub title: String,
        pub detail: String,
        #[serde(alias = "type")]
        pub type_err: String,
    }

    #[derive(Debug, Deserialize, Serialize)] 
    struct GeneralErrorResponse {
        pub title: String,
        #[serde(alias = "type")]
        pub typ: String,
        pub status: i32,
        pub detail: String,
    }

    #[derive(Debug, Deserialize, Serialize)]
    struct ErrorResponseElement {
        message: String,
    }

    impl Display for TweetError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Display for ReplyId {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.in_reply_to_tweet_id)
        }
    }
    impl Display for Reply {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "{}", self.text)
        }
    }

    impl Error for TweetError {}
}
