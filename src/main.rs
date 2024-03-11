#![warn(warnings)]

mod error;
mod filters;
mod media;

use error::*;
use media::Media;
use std::fmt::Write as _;

static TEMPLATE_DIR: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/templates");

struct Data {
    template: tera_hot::Template,
    root: std::path::PathBuf,
    title: String,
}

#[actix_web::main]
async fn main() -> Result {
    #[cfg(debug_assertions)]
    envir::dotenv();

    env_logger::init();

    let bind = format!("{}:{}", envir::get("LISTEN_IP")?, envir::get("LISTEN_PORT")?);

    let root = envir::get("APP_WIKI_ROOT")?;
    let title = envir::get("APP_TITLE")?;

    actix_web::HttpServer::new(move || {
        let mut template = tera_hot::Template::new(TEMPLATE_DIR);
        template.register_filter("markdown", crate::filters::markdown);

        let data = Data {
            root: std::path::PathBuf::from(&root),
            template,
            title: title.clone(),
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
    .await?;

    Ok(())
}

async fn thumbnail(request: actix_web::HttpRequest) -> actix_web::HttpResponse {
    let path = get_path(&request);

    let Ok(image) = image::open(&path) else {
        return missing(&request);
    };

    let thumbnail = image.thumbnail(200, 200);
    let mut body: Vec<u8> = Vec::new();
    let Ok(format) = image::ImageFormat::from_path(&path) else {
        return missing(&request);
    };
    let content_type = format.to_mime_type();

    thumbnail
        .write_to(&mut std::io::Cursor::new(&mut body), format)
        .unwrap();

    actix_web::HttpResponse::Ok()
        .content_type(content_type)
        .body(body)
}

fn missing(request: &actix_web::HttpRequest) -> actix_web::HttpResponse {
    use actix_web::Responder;

    return actix_files::NamedFile::open("static/img/missing.png")
        .unwrap()
        .respond_to(&request);
}

async fn index(request: actix_web::HttpRequest) -> Result<actix_web::HttpResponse> {
    use std::io::Read;

    let data: &Data = request.app_data().unwrap();
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

                file.read_to_string(&mut summary).ok();

                let regex = regex::Regex::new(r"(?m)^").unwrap();

                contents = markdown(&regex.replace_all(&summary, "> "));
            }
        }

        media = path.join(".media").exists();

        if media {
            let media = generate_media(&data.template, slug, &path)?;
            contents.push_str(&media);
        } else {
            let index = generate_index(slug, &path);
            contents.push_str(&markdown(&index));
        }
        context.insert("contents", &contents);
    } else if is_markdown(&path) {
        let mut contents = String::new();

        let mut file = std::fs::File::open(path).map_err(|_| Error::NotFound)?;

        file.read_to_string(&mut contents)
            .map_err(|_| Error::NotFound)?;

        context.insert("toc", &table_of_content(&contents));
        context.insert("contents", &markdown(&contents));
    } else {
        use actix_web::Responder;

        let response = actix_files::NamedFile::open(path)
            .unwrap()
            .respond_to(&request);

        return Ok(response);
    }

    context.insert("is_index", &(!media && is_index));
    context.insert("nav", &generate_breadcrumb(slug));
    context.insert("title", &generate_title(&data.title, slug));

    let body = match data.template.render("index.html", &context) {
        Ok(body) => body,
        Err(err) => return Err(Error::from(err)),
    };

    let response = actix_web::HttpResponse::Ok()
        .content_type("text/html")
        .body(body);

    Ok(response)
}

fn is_markdown(path: &std::path::Path) -> bool {
    path.extension() == Some(std::ffi::OsStr::new("md"))
}

fn get_path(request: &actix_web::HttpRequest) -> std::path::PathBuf {
    let slug = request.match_info().query("slug");
    let data: &Data = request.app_data().unwrap();

    let mut path = std::path::PathBuf::new();
    path.push(&data.root);
    path.push(slug);

    path
}

fn markdown(input: &str) -> String {
    let mut output = String::new();

    let mut options = pulldown_cmark::Options::empty();
    options.insert(pulldown_cmark::Options::ENABLE_TABLES);
    options.insert(pulldown_cmark::Options::ENABLE_FOOTNOTES);

    let parser = pulldown_cmark::Parser::new_ext(input, options);
    pulldown_cmark::html::push_html(&mut output, parser);

    let regex = regex::Regex::new(r#"<h1>(?P<text>.*?)</h1>"#).unwrap();

    regex
        .replace_all(&output, |caps: &regex::Captures<'_>| {
            let id = text_to_id(&caps["text"]);

            format!("<h1 id=\"{id}\">{}</h1>", &caps["text"])
        })
        .to_string()
}

fn text_to_id(text: &str) -> String {
    text.to_lowercase().replace(' ', "-")
}

fn table_of_content(input: &str) -> String {
    let mut toc = String::new();
    let mut current_level = None;

    let parser = pulldown_cmark::Parser::new(input);

    for event in parser {
        use pulldown_cmark::{Event::*, Tag::*, TagEnd};

        match event {
            Start(Heading { level, .. }) if level == pulldown_cmark::HeadingLevel::H1 => {
                toc.push_str("<ul>\n");
                current_level = Some(level);
            }
            Text(text) if current_level.is_some() => {
                writeln!(
                    toc,
                    "<li><a href=\"#{}\">{text}</a></li>",
                    text_to_id(&text)
                )
                .ok();
            }
            End(TagEnd::Heading(level)) => {
                if Some(level) <= current_level {
                    toc.push_str("</ul>\n");
                    current_level = None;
                }
            }
            _ => (),
        }
    }

    toc
}

fn generate_title(title: &str, slug: &str) -> String {
    slug.split('/')
        .rev()
        .chain(vec![title])
        .collect::<Vec<&str>>()
        .join(" | ")
}

fn generate_breadcrumb(slug: &str) -> String {
    let mut breadcrumb = String::from("[~](/)");
    let mut url = String::new();

    for part in slug.split('/') {
        write!(url, "/{part}").ok();
        write!(breadcrumb, "/[{part}]({})", url_encode(&url)).ok();
    }

    breadcrumb
}

fn generate_media(
    template: &tera_hot::Template,
    root: &str,
    path: &std::path::Path,
) -> Result<String> {
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
        let url = link(root, path, &entry);
        let title = title(&entry);

        files.push(Media::new(entry.path(), &url, &title));
    }

    let mut context = tera::Context::new();
    context.insert("files", &files);

    match template.render("panel.html", &context) {
        Ok(body) => Ok(body),
        Err(err) => Err(Error::from(err)),
    }
}

fn generate_index(root: &str, path: &std::path::Path) -> String {
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
            let link = link(root, path, &entry);
            let title = title(&entry);

            writeln!(summary, "{indent}* [{title}]({link})").ok();
        }
    }

    summary
}

fn link(root: &str, path: &std::path::Path, entry: &walkdir::DirEntry) -> String {
    let mut root = root.to_string();
    let entry = entry.path().strip_prefix(path).unwrap().display();

    if !root.starts_with('/') {
        root.insert(0, '/');
    }

    if !root.ends_with('/') {
        root.push('/');
    }

    let link = format!("{root}{entry}");

    url_encode(&link)
}

fn url_encode(url: &str) -> String {
    url.replace(' ', "%20")
}

fn title(entry: &walkdir::DirEntry) -> String {
    entry.file_name().to_str().unwrap().replace(".md", "")
}

fn is_hidden(entry: &walkdir::DirEntry) -> bool {
    entry
        .file_name()
        .to_str()
        .map_or(false, |s| s.starts_with('.') || s == "index.md")
}
