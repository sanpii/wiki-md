mod error;
mod filters;
mod media;

use error::Error;
use media::Media;

#[derive(Debug)]
struct Data {
    template: tera::Tera,
    root: std::path::PathBuf,
    title: String,
}

#[actix_rt::main]
async fn main() -> std::io::Result<()>
{
    #[cfg(debug_assertions)]
    dotenv::dotenv()
        .ok();

    let bind = format!("{}:{}", env("LISTEN_IP"), env("LISTEN_PORT"));

    actix_web::HttpServer::new(|| {
        let root = env("APP_WIKI_ROOT");
        let title = env("APP_TITLE");
        let mut template = match tera::Tera::new("templates/**/*") {
            Ok(template) => template,
            Err(err) => panic!("Parsing error(s): {}", err),
        };
        template.register_filter("markdown", filters::markdown);

        let data = Data {
            root: std::path::PathBuf::from(root),
            template,
            title,
        };

        let static_files = actix_files::Files::new("/static", "static/");

        actix_web::App::new()
            .app_data(data)
            .service(static_files)
            .route("/thumbnail/{slug:.*}", actix_web::web::get().to(thumbnail))
            .route("/{slug:.*}", actix_web::web::get().to(index))
    })
    .bind(&bind)?
    .run()
    .await
}

fn env(name: &str) -> String
{
    std::env::var(name)
        .unwrap_or_else(|_| panic!("Missing {} env variable", name))
}

async fn thumbnail(request: actix_web::HttpRequest) -> actix_web::HttpResponse
{
    let path = get_path(&request);

    let image = match image::open(&path) {
        Ok(image) => image,
        Err(_) => {
            use actix_web::Responder;

            return actix_files::NamedFile::open("static/img/missing.png")
                .unwrap()
                .respond_to(&request)
                .await
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

async fn index(request: actix_web::HttpRequest) -> Result<actix_web::HttpResponse, Error>
{
    use std::io::Read;

    let data: &Data = &request.app_data()
        .unwrap();
    let slug = request.match_info().query("slug");

    let path = get_path(&request);

    if !path.exists() {
        return Err(Error::NotFound);
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
            let media = match generate_media(&data.template, &slug, &path) {
                Ok(contents) => contents,
                Err(err) => return Err(err),
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
            Err(_) => return Err(Error::NotFound),
        };

        match file.read_to_string(&mut contents) {
            Ok(_) => (),
            Err(_) => return Err(Error::NotFound),
        };

        context.insert("contents", &markdown(&contents));
    }
    else {
        use actix_web::Responder;

        let response = actix_files::NamedFile::open(path)
            .unwrap()
            .respond_to(&request)
            .await
            .unwrap();

        return Ok(response);
    }

    context.insert("is_index", &(!media && is_index));
    context.insert("nav", &generate_breadcrumb(&slug));
    context.insert("title", &generate_title(&data.title, &slug));

    let body = match data.template.render("index.html", &context) {
        Ok(body) => body,
        Err(err) => return Err(Error::from(err)),
    };

    let response = actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body);

    Ok(response)
}

fn is_markdown(path: &std::path::Path) -> bool
{
    path.extension() == Some(std::ffi::OsStr::new("md"))
}

fn get_path(request: &actix_web::HttpRequest) -> std::path::PathBuf
{
    let slug = request.match_info().query("slug");
    let data: &Data = request.app_data()
        .unwrap();

    let mut path = std::path::PathBuf::new();
    path.push(&data.root);
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

    if !root.starts_with('/') {
        root.insert_str(0, "/");
    }

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
