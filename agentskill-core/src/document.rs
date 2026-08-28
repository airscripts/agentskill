use std::collections::HashMap;

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

pub fn serialize(document: &Document) -> String {
    let mut output = document.preamble.clone();

    for section in &document.sections {
        if !output.is_empty() && !output.ends_with("\n\n") {
            output.push('\n');
        }
        output.push_str(&"#".repeat(section.level));
        output.push(' ');
        output.push_str(&section.heading);
        output.push('\n');

        if section.body.is_empty() {
            output.push('\n');
        } else {
            if !section.body.starts_with('\n') {
                output.push('\n');
            }
            output.push_str(&section.body);

            if !output.ends_with('\n') {
                output.push('\n');
            }
        }

        if !output.ends_with("\n\n") {
            output.push('\n');
        }
    }
    output
}

pub fn merge(
    existing: &str,
    generated: &Document,
    only: &[String],
    exclude: &[String],
    force: bool,
) -> String {
    if force {
        return serialize(generated);
    }

    let requested: Option<std::collections::HashSet<_>> = if only.is_empty() {
        None
    } else {
        Some(only.iter().map(|x| normalize_section_name(x)).collect())
    };

    let excluded: std::collections::HashSet<_> =
        exclude.iter().map(|x| normalize_section_name(x)).collect();

    let mut document = parse(existing);
    let generated_map: HashMap<_, _> = generated
        .sections
        .iter()
        .map(|section| (normalize_section_name(&section.heading), section))
        .collect();

    for section in &mut document.sections {
        let key = normalize_section_name(&section.heading);

        if requested
            .as_ref()
            .is_some_and(|items| !items.contains(&key))
            || excluded.contains(&key)
        {
            continue;
        }

        if let Some(new_section) = generated_map.get(&key) {
            *section = (*new_section).clone();
        }
    }

    for section in &generated.sections {
        let key = normalize_section_name(&section.heading);

        if requested
            .as_ref()
            .is_some_and(|items| !items.contains(&key))
            || excluded.contains(&key)
        {
            continue;
        }

        if !document
            .sections
            .iter()
            .any(|item| normalize_section_name(&item.heading) == key)
        {
            document.sections.push(section.clone());
        }
    }
    serialize(&document)
}
