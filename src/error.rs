use derive_more::derive::From;

pub type Result<T> = core::result::Result<T, Error<'static>>;
#[derive(Debug, From)]
pub enum Error<'a> {
    Static(String),
    Any(anyhow::Error),
    Tracing(tracing::subscriber::SetGlobalDefaultError),
    Io(std::io::Error),
    Parsing(scraper::error::SelectorErrorKind<'a>)

}

impl core::fmt::Display for Error<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return write!(f, "{self:?}");
    }
}

impl std::error::Error for Error<'_> {}
