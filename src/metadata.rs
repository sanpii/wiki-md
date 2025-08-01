use std::collections::BTreeMap;

#[derive(Default)]
pub struct Metadata(BTreeMap<String, String>);

impl Metadata {
    pub fn parse(input: &str) -> Self {
        if !input.starts_with("---") {
            return Self::default();
        }

        let metadata = input
            .split("---")
            .nth(1)
            .unwrap_or_default()
            .split('\n')
            .filter_map(|x| x.split_once(':'))
            .map(|(k, v)| {
                (
                    k.trim_matches(' ').to_string(),
                    v.trim_matches(' ').to_string(),
                )
            })
            .collect();

        Self(metadata)
    }

    pub fn to_html(&self) -> String {
        if self.0.is_empty() {
            return String::new();
        }

        let span = self
            .0
            .iter()
            .map(|(k, v)| {
                let class = if v == "yes" {
                    k.clone()
                } else {
                    format!("{v}-{k}")
                };

                format!("<span class=\"{class}\"></span>")
            })
            .collect::<Vec<_>>()
            .join(" ");

        format!("<div id=\"metadata\">{span}</div>")
    }
}
