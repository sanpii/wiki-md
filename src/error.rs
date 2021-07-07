#[derive(Debug)]
pub enum Error {
    NotFound,
    Template(tera::Error),
}

impl From<tera::Error> for Error
{
    fn from(err: tera::Error) -> Self
    {
        Error::Template(err)
    }
}

impl std::fmt::Display for Error
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result
    {
        let s = match self {
            Error::NotFound => "Not found",
            Error::Template(_) => "Template error",
        };

        write!(f, "{}", s)
    }
}

impl From<&Error> for actix_web::http::StatusCode
{
    fn from(error: &Error) -> Self
    {
        use actix_web::http::StatusCode;

        match error {
            Error::NotFound => StatusCode::NOT_FOUND,
            Error::Template(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl actix_web::error::ResponseError for Error
{
    fn error_response(&self) -> actix_web::HttpResponse
    {
        let status: actix_web::http::StatusCode = self.into();

        let file = format!("errors/{}.html", u16::from(status));
        let template = match tera::Tera::new("templates/**/*") {
            Ok(template) => template,
            Err(err) => panic!("Parsing error(s): {}", err),
        };
        let body = match template.render(&file, &tera::Context::new()) {
            Ok(body) => body,
            Err(err) => {
                eprintln!("{:?}", err);

                "Internal server error".to_string()
            }
        };

        actix_web::HttpResponse::build(status)
            .header(actix_web::http::header::CONTENT_TYPE, "text/html")
            .body(body)
    }
}
