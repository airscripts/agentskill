#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Section {
    pub level: usize,
    pub heading: String,
    pub body: String,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Document {
    pub preamble: String,
    pub sections: Vec<Section>,
}

pub fn normalize_section_name(name: &str) -> String {
    let name = name.trim();

    let name = name
        .split_once('.')
        .filter(|(prefix, _)| {
            !prefix.is_empty() && prefix.chars().all(|char| char.is_ascii_digit())
        })
        .map_or(name, |(_, value)| value.trim());
    name.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

pub fn parse(text: &str) -> Document {
    let mut preamble = String::new();

    let mut sections = Vec::new();
    let mut current: Option<Section> = None;

    for line in text.lines() {
        let Some((level, heading)) = parse_heading(line) else {
            if let Some(section) = current.as_mut() {
                section.body.push_str(line);
                section.body.push('\n');
            } else {
                preamble.push_str(line);
                preamble.push('\n');
            }
            continue;
        };

        if current.is_none()
            && level == 1
            && matches!(
                normalize_section_name(&heading).as_str(),
                "agents" | "agents.md"
            )
        {
            preamble.push_str(line);
            preamble.push('\n');
            continue;
        }

        {
            if let Some(section) = current.take() {
                sections.push(section);
            }
            current = Some(Section {
                level,
                heading,
                body: String::new(),
            });
        }
    }

    if let Some(section) = current {
        sections.push(section);
    }
    Document { preamble, sections }
}

fn parse_heading(line: &str) -> Option<(usize, String)> {
    let trimmed = line.trim_start_matches([' ', '\t']);
    let indentation = line.len() - trimmed.len();

    if indentation > 3 || !trimmed.starts_with('#') {
        return None;
    }

    let level = trimmed.chars().take_while(|char| *char == '#').count();
    if !(1..=6).contains(&level) {
        return None;
    }

    let rest = &trimmed[level..];
    if !rest.is_empty() && !rest.starts_with([' ', '\t']) {
        return None;
    }

    Some((level, rest.trim().to_string()))
}
