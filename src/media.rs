#[derive(serde_derive::Serialize)]
pub struct Media {
    path: std::path::PathBuf,
    pub id: String,
    pub url: String,
    pub info: Info,
    pub title: String,
    pub thumbnail: String,
}

impl Media {
    pub fn new(path: &std::path::Path, url: &str, title: &str) -> Self {
        Self {
            path: path.to_path_buf(),
            id: url.to_string(),
            thumbnail: format!("/thumbnail{url}"),
            url: url.to_string(),
            info: Info::new(path),
            title: title.to_string(),
        }
    }
}

#[derive(serde_derive::Serialize)]
pub struct Info {
    is_dir: bool,
    is_image: bool,
    is_sound: bool,
    is_video: bool,
    is_media: bool,
}

impl Info {
    pub fn new(path: &std::path::Path) -> Self {
        Self {
            is_dir: path.is_dir(),
            is_image: Self::is_image(path),
            is_sound: Self::is_sound(path),
            is_video: Self::is_video(path),
            is_media: Self::is_media(path),
        }
    }

    fn is_media(path: &std::path::Path) -> bool {
        Self::is_image(path) || Self::is_sound(path) || Self::is_video(path)
    }

    fn is_image(path: &std::path::Path) -> bool {
        ["jpg", "jpeg", "png", "gif"].contains(&Self::extension(path))
    }

    fn is_video(path: &std::path::Path) -> bool {
        ["mpeg", "ogv", "mp4", "mov"].contains(&Self::extension(path))
    }

    fn is_sound(path: &std::path::Path) -> bool {
        ["ogg", "mp3"].contains(&Self::extension(path))
    }

    fn extension(path: &std::path::Path) -> &str {
        path.extension()
            .unwrap_or_default()
            .to_str()
            .unwrap_or_default()
    }
}
