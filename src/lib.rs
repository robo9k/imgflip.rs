use reqwest::header::{HeaderValue, CONTENT_TYPE};
use serde::{Deserialize, Serialize};
use url::Url;

/// Blank meme template that can be captioned with text boxes
#[derive(Debug, Deserialize)]
pub struct MemeTemplate {
    id: String,
    name: String,
    url: Url,
    width: u32,
    height: u32,
    box_count: u32,
}

impl MemeTemplate {
    /// Returns the identifier of this meme template.
    ///
    /// This equals the required `template_id` input parameter for the `/caption_image` API endpoint.
    pub fn id(&self) -> &str {
        &self.id
    }

    /// Returns the name of this meme template.
    ///
    /// This is a human readable english string such as "Grumpy Cat".
    pub fn name(&self) -> &str {
        &self.name
    }

    /// Returns the URL of the blank image for this meme template.
    ///
    /// This is an image with the dimensions given in `width` and `height`,
    /// without any caption boxes.
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the width in pixels of the blank image for this meme template.
    pub fn width(&self) -> u32 {
        self.width
    }

    /// Returns the height in pixels of the blank image for this meme template.
    pub fn height(&self) -> u32 {
        self.height
    }

    /// Returns the number of caption boxes that this meme templates uses.
    ///
    /// Some memes have more than just a top and bottom text by default.
    pub fn box_count(&self) -> u32 {
        self.box_count
    }
}

#[derive(Debug, Deserialize)]
#[serde(untagged)]
enum Response<T> {
    SuccessResponse {
        success: bool,
        data: T,
    },
    FailureResponse {
        success: bool,
        error_message: String,
    },
}

impl<T> Response<T> {
    fn convert(self) -> Result<T> {
        match self {
            Response::SuccessResponse { data, .. } => Ok(data),
            Response::FailureResponse { error_message, .. } => Err(Error::ApiError(error_message)),
        }
    }
}

/// Font for [`CaptionBoxesRequest`](crate::CaptionBoxesRequest)
///
/// API defaults to `Impact`
#[derive(Debug, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CaptionFont {
    Impact,
    Arial,
}

/// Text and other fields for image caption
#[derive(Debug, Serialize)]
pub struct CaptionBox {
    text: String,
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    color: Option<String>,
    outline_color: Option<String>,
}

/// Builder for [`CaptionBox`](crate::CaptionBox)
pub struct CaptionBoxBuilder {
    text: String,
    x: Option<u32>,
    y: Option<u32>,
    width: Option<u32>,
    height: Option<u32>,
    color: Option<String>,
    outline_color: Option<String>,
}

impl CaptionBoxBuilder {
    pub fn new<S: Into<String>>(text: S) -> Self {
        CaptionBoxBuilder {
            text: text.into(),
            x: None,
            y: None,
            width: None,
            height: None,
            color: None,
            outline_color: None,
        }
    }

    pub fn dimension(mut self, x: u32, y: u32, width: u32, height: u32) -> Self {
        self.x = Some(x);
        self.y = Some(y);
        self.width = Some(width);
        self.height = Some(height);
        self
    }

    pub fn color<S: Into<String>>(mut self, color: S) -> Self {
        self.color = Some(color.into());
        self
    }

    pub fn outline_color<S: Into<String>>(mut self, outline_color: S) -> Self {
        self.outline_color = Some(outline_color.into());
        self
    }

    pub fn build(self) -> CaptionBox {
        CaptionBox {
            text: self.text,
            x: self.x,
            y: self.y,
            width: self.width,
            height: self.height,
            color: self.color,
            outline_color: self.outline_color,
        }
    }
}

/// Request data to caption a meme template
#[derive(Debug, Serialize)]
pub struct CaptionBoxesRequest {
    template_id: String,
    font: Option<CaptionFont>,
    max_font_size: Option<u32>,
    boxes: Vec<CaptionBox>,
}

/// Builder for [`CaptionBoxesRequest`](crate::CaptionBoxesRequest)
pub struct CaptionBoxesRequestBuilder {
    template_id: String,
    font: Option<CaptionFont>,
    max_font_size: Option<u32>,
    boxes: Vec<CaptionBox>,
}

impl CaptionBoxesRequestBuilder {
    pub fn new<S: Into<String>>(template_id: S) -> Self {
        CaptionBoxesRequestBuilder {
            template_id: template_id.into(),
            font: None,
            max_font_size: None,
            boxes: Vec::with_capacity(2),
        }
    }

    pub fn font(mut self, font: CaptionFont) -> Self {
        self.font = Some(font);
        self
    }

    pub fn max_font_size(mut self, max_font_size: u32) -> Self {
        self.max_font_size = Some(max_font_size);
        self
    }

    pub fn caption_box(mut self, caption_box: CaptionBox) -> Self {
        self.boxes.push(caption_box);
        self
    }

    pub fn build(self) -> CaptionBoxesRequest {
        CaptionBoxesRequest {
            template_id: self.template_id,
            font: self.font,
            max_font_size: self.max_font_size,
            boxes: self.boxes,
        }
    }
}

/// A captioned meme template
#[derive(Debug, Deserialize)]
pub struct CaptionImageResponse {
    url: Url,
    page_url: Url,
}

impl CaptionImageResponse {
    /// Returns the URL of the generated captioned image
    pub fn url(&self) -> &Url {
        &self.url
    }

    /// Returns the URL of the generated captioned image page
    pub fn page_url(&self) -> &Url {
        &self.page_url
    }
}

/// [`Error`](std:error:Error) implementation for all crate errors
#[derive(thiserror::Error, Debug)]
pub enum Error {
    /// Error internal to `reqwest`
    #[error("HTTP request/response error")]
    Reqwest(#[from] reqwest::Error),

    /// Error internal to `serde_qs`
    #[error("form querystring de/serialization error")]
    SerdeQs(#[from] serde_qs::Error),

    /// API error message from `api.imgflip.com`
    #[error("API error: {0}")]
    ApiError(String),
}

/// [`Result`](std::result::Result) alias with crate's [`Error`](crate::Error)
pub type Result<T> = std::result::Result<T, crate::Error>;

/// Client for `api.imgflip.com` that obtains blank meme templates
///
/// You should resuse `Client` instances, since they do internal connection pooling.
/// # Example
/// ```no_run
/// # #[tokio::main]
/// # async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
/// let client = imgflip::Client::new();
/// let memes = client.memes().await?;
/// println!("much memes. very easy. wow.\n{:?}", memes);
/// # Ok(())
/// # }
/// ```
pub struct Client {
    client: reqwest::Client,
}

impl Client {
    /// Creates a new instance with default values
    pub fn new() -> Self {
        Client {
            client: reqwest::Client::new(),
        }
    }

    async fn client_memes(client: &reqwest::Client) -> Result<Vec<MemeTemplate>> {
        #[derive(Debug, Deserialize)]
        struct MemeTemplatesData {
            memes: Vec<MemeTemplate>,
        }

        client
            .get("https://api.imgflip.com/get_memes")
            .send()
            .await?
            .error_for_status()?
            .json::<Response<MemeTemplatesData>>()
            .await?
            .convert()
            .map(|r| r.memes)
    }

    /// Calls the `/get_memes` endpoint to return a list of popular meme templates
    pub async fn memes(&self) -> Result<Vec<MemeTemplate>> {
        Self::client_memes(&self.client).await
    }
}

/// Client for `api.imgflip.com` that can caption meme templates
///
/// Unlike [`Client`](imgflip::Client) this requires an account on [imgflip.com](https://imgflip.com/signup).
pub struct AccountClient {
    username: String,
    password: String,
    client: reqwest::Client,
}

impl AccountClient {
    /// Creates a new instance for the given account
    pub fn new<S: Into<String>>(username: S, password: S) -> Self {
        AccountClient {
            client: reqwest::Client::new(),
            username: username.into(),
            password: password.into(),
        }
    }

    /// Calls the `/caption_image` endpoint to add caption boxes to a meme template
    pub async fn caption_image(
        &self,
        image_caption: CaptionBoxesRequest,
    ) -> Result<CaptionImageResponse> {
        #[derive(Debug, Serialize)]
        struct RequestAuthWrapper<T> {
            #[serde(flatten)]
            request: T,
            username: String,
            password: String,
        }

        self.client
            .post("https://api.imgflip.com/caption_image")
            .header(
                CONTENT_TYPE,
                HeaderValue::from_static("application/x-www-form-urlencoded"),
            )
            .body(serde_qs::to_string(&RequestAuthWrapper {
                request: image_caption,
                username: self.username.clone(),
                password: self.password.clone(),
            })?)
            .send()
            .await?
            .error_for_status()?
            .json::<Response<CaptionImageResponse>>()
            .await?
            .convert()
    }

    /// Calls the `/get_memes` endpoint to return a list of popular meme templates
    pub async fn memes(&self) -> Result<Vec<MemeTemplate>> {
        Client::client_memes(&self.client).await
    }
}
