mod error;
mod filters;
mod media;

use error::Error;
use media::Media;

struct State {
    template: tera::Tera,
    root: std::path::PathBuf,
    title: String,
}

fn main()
{
    #[cfg(debug_assertions)]
    dotenv::dotenv()
        .ok();

    let bind = format!("{}:{}", env("LISTEN_IP"), env("LISTEN_PORT"));

    actix_web::server::new(|| {
        let root = env("APP_WIKI_ROOT");
        let title = env("APP_TITLE");
        let mut template = tera::compile_templates!("templates/**/*");
        template.register_filter("markdown", filters::markdown);

        let state = State {
            root: std::path::PathBuf::from(root),
            template,
            title,
        };

        let static_files = actix_web::fs::StaticFiles::new("static/")
            .expect("failed constructing static files handler");
        let errors = actix_web::middleware::ErrorHandlers::new()
                .handler(actix_web::http::StatusCode::NOT_FOUND, |req, res| error(404, req, res))
                .handler(actix_web::http::StatusCode::INTERNAL_SERVER_ERROR, |req, res| error(500, req, res));

        actix_web::App::with_state(state)
            .middleware(errors)
            .handler("/static", static_files)
            .route("/thumbnail/{slug:.*}", actix_web::http::Method::GET, thumbnail)
            .route("/{slug:.*}", actix_web::http::Method::GET, index)
    })
    .bind(&bind)
    .unwrap_or_else(|_| panic!("Can not bind to {}", bind))
    .run();
}

fn env(name: &str) -> String
{
    std::env::var(name)
        .unwrap_or_else(|_| panic!("Missing {} env variable", name))
}

fn thumbnail(request: actix_web::HttpRequest<State>) -> actix_web::HttpResponse
{
    let path = get_path(&request);

    let image = match image::open(&path) {
        Ok(image) => image,
        Err(_) => {
            use actix_web::Responder;

            return actix_web::fs::NamedFile::open("static/img/missing.png")
                .unwrap()
                .respond_to(&request)
                .unwrap();
        },
    };

    let thumbnail = image.thumbnail(200, 200);
    let mut body: Vec<u8> = Vec::new();
    let (format, content_type) = guess_format(&path);

    thumbnail.write_to(&mut body, format)
        .unwrap();

    actix_web::HttpResponse::Ok()
        .content_type(content_type)
        .body(body)
}

fn guess_format(path: &std::path::Path) -> (image::ImageOutputFormat, &'static str)
{
    let ext = path.extension()
        .and_then(|s| s.to_str())
        .map_or("".to_string(), |s| s.to_ascii_lowercase());

    match ext.as_str() {
        "png" => (image::ImageOutputFormat::PNG, "image/png"),
        "jpeg" | "jpg" => (image::ImageOutputFormat::JPEG(80), "image/jpeg"),
        "gif" => (image::ImageOutputFormat::GIF, "image/gif"),
        "bmp" => (image::ImageOutputFormat::BMP, "image/bmp"),
        "ico" => (image::ImageOutputFormat::ICO, "image/x-icon"),
        _ => (image::ImageOutputFormat::Unsupported(ext), "image/octet-stream"),
    }
}

fn index(request: actix_web::HttpRequest<State>) -> actix_web::HttpResponse
{
    use std::io::Read;

    let slug: String = request.match_info().query("slug")
        .unwrap();

    let path = get_path(&request);

    if !path.exists() {
        return Error::NotFound.into();
    }

    let mut media = false;
    let mut is_index = false;

    let mut context = tera::Context::new();
    let mut contents = String::new();

    if path.is_dir() {
        is_index = true;

        let index = path.join("index.md");

        if index.exists() {
            if let Ok(mut file) = std::fs::File::open(index) {
                let mut summary = String::new();

                file.read_to_string(&mut summary)
                    .ok();

                #[allow(clippy::trivial_regex)]
                let regex = regex::Regex::new(r"(?m)^")
                    .unwrap();

                contents = markdown(&regex.replace_all(&summary, "> "));
            }
        }

        media = path.join(".media").exists();

        if media {
            let media = match generate_media(&request.state().template, &slug, &path) {
                Ok(contents) => contents,
                Err(err) => return err.into(),
            };
            contents.push_str(&media);
            context.insert("contents", &contents);
        }
        else {
            let index = generate_index(&slug, &path);
            contents.push_str(&markdown(&index));
            context.insert("contents", &contents);
        }
    }
    else if is_markdown(&path) {
        let mut contents = String::new();

        let mut file = match std::fs::File::open(path) {
            Ok(file) => file,
            Err(_) => return Error::NotFound.into(),
        };

        match file.read_to_string(&mut contents) {
            Ok(_) => (),
            Err(_) => return Error::NotFound.into(),
        };

        context.insert("contents", &markdown(&contents));
    }
    else {
        use actix_web::Responder;

        return actix_web::fs::NamedFile::open(path)
            .unwrap()
            .respond_to(&request)
            .unwrap();
    }

    context.insert("is_index", &(!media && is_index));
    context.insert("nav", &generate_breadcrumb(&slug));
    context.insert("title", &generate_title(&request.state().title, &slug));

    let body = match request.state().template.render("index.html", &context) {
        Ok(body) => body,
        Err(err) => return Error::from(err).into(),
    };

    actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body)
}

fn is_markdown(path: &std::path::Path) -> bool
{
    path.extension() == Some(std::ffi::OsStr::new("md"))
}

fn error(status: u32, request: &actix_web::HttpRequest<State>, resp: actix_web::HttpResponse)
    -> actix_web::Result<::actix_web::middleware::Response>
{
    let template = format!("errors/{}.html", status);
    let mut context = tera::Context::new();
    context.insert("title", &request.state().title);
    context.insert("nav", "");

    let body = match request.state().template.render(&template, &context) {
        Ok(body) => body,
        Err(_) => "Internal server error".to_string(),
    };

    let builder = resp.into_builder()
        .header(actix_web::http::header::CONTENT_TYPE, "text/html")
        .body(body);

    Ok(actix_web::middleware::Response::Done(builder))
}

fn get_path(request: &actix_web::HttpRequest<State>) -> std::path::PathBuf
{
    let slug: String = request.match_info().query("slug")
        .unwrap();

    let root = &request.state().root;

    let mut path = std::path::PathBuf::new();
    path.push(root);
    path.push(slug);

    path
}

fn markdown(input: &str) -> String
{
    let mut output = String::new();

    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);

    let parser = pulldown_cmark::Parser::new_ext(&input, options);
    pulldown_cmark::html::push_html(&mut output, parser);

    output
}

fn generate_title(title: &str, slug: &str) -> String
{
    slug.split('/')
        .rev()
        .chain(vec![title])
        .collect::<Vec<&str>>()
        .join(" | ")
}

fn generate_breadcrumb(slug: &str) -> String
{
    let mut breadcrumb = String::from("[~](/)");
    let mut url = String::new();

    for part in slug.split('/') {
        url.push_str(&format!("/{}", part));
        breadcrumb.push_str(&format!("/[{}]({})", part, url));
    }

    breadcrumb
}

fn generate_media(template: &tera::Tera, root: &str, path: &std::path::Path) -> Result<String, Error>
{
    let mut files = vec![];

    if path.to_str() == Some("") {
        return Ok(String::new());
    }

    let walker = walkdir::WalkDir::new(path)
        .min_depth(1)
        .max_depth(1)
        .into_iter();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();
        let url = link(root, &path, &entry);
        let title = title(&entry);

        files.push(Media::new(&entry.path(), &url, &title));
    }

    let mut context = tera::Context::new();
    context.insert("files", &files);

    match template.render("panel.html", &context) {
        Ok(body) => Ok(body),
        Err(err) => Err(Error::from(err)),
    }
}

fn generate_index(root: &str, path: &std::path::Path) -> String
{
    let mut summary = String::new();

    if path.to_str() == Some("") {
        return summary;
    }

    let walker = walkdir::WalkDir::new(path)
        .sort_by(|a, b| a.file_name().cmp(b.file_name()))
        .min_depth(1)
        .into_iter();

    for entry in walker.filter_entry(|e| !is_hidden(e)) {
        let entry = entry.unwrap();

        if entry.path().is_dir() || is_markdown(entry.path()) {
            let indent = " ".repeat((entry.depth() - 1) * 4);
            let link = link(root, &path, &entry);
            let title = title(&entry);

            summary.push_str(&format!("{}* [{}]({})\n", indent, title, link));
        }
    }

    summary
}

fn link(root: &str, path: &std::path::Path, entry: &walkdir::DirEntry) -> String
{
    let mut root = root.to_string();
    let entry = entry.path()
        .strip_prefix(path)
        .unwrap()
        .display();

    if !root.ends_with('/') {
        root.push_str("/");
    }

    let link = format!("{}{}", root, entry);

    url::form_urlencoded::Serializer::new(link)
        .finish()
}

fn title(entry: &walkdir::DirEntry) -> String
{
    entry.file_name()
        .to_str()
        .unwrap()
        .replace(".md", "")
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool
{
    entry.file_name()
         .to_str()
         .map(|s| s.starts_with('.') || s == "index.md")
         .unwrap_or(false)
}
