pub(crate) const CANONICAL_SIGNATURE: &str = "---\n\n> Generated and maintained by [Agentskill](https://github.com/airscripts/agentskill).\n> Do not touch this file. It is automatically managed by Agentskill.";

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub(crate) enum SignatureIssue {
    Missing,
    Duplicate,
    Malformed,
    NonTerminal,
    Unexpected,
}

impl SignatureIssue {
    pub(crate) const fn kind(self) -> &'static str {
        match self {
            Self::Missing => "missing_signature",
            Self::Duplicate => "duplicate_signature",
            Self::Malformed => "malformed_signature",
            Self::NonTerminal => "non_terminal_signature",
            Self::Unexpected => "configuration_contradiction",
        }
    }

    pub(crate) const fn message(self) -> &'static str {
        match self {
            Self::Missing => "managed signature is missing",
            Self::Duplicate => "managed signature appears more than once",
            Self::Malformed => "Agentskill signature is malformed",
            Self::NonTerminal => "managed signature must be final content",
            Self::Unexpected => "managed signature is present while signatures are disabled",
        }
    }
}

pub(crate) fn issues(content: &str, enabled: bool) -> Vec<SignatureIssue> {
    let normalized = content.replace("\r\n", "\n");
    let count = normalized.matches(CANONICAL_SIGNATURE).count();
    let marker_present = normalized.contains("Generated and maintained by [Agentskill]")
        || normalized.contains("automatically managed by Agentskill");
    let mut issues = Vec::new();

    if count > 1 {
        issues.push(SignatureIssue::Duplicate);
    }

    if count == 0 && marker_present {
        issues.push(SignatureIssue::Malformed);
    }

    if count == 1 && !normalized.trim_end().ends_with(CANONICAL_SIGNATURE) {
        issues.push(SignatureIssue::NonTerminal);
    }

    if count == 0 && enabled && !marker_present {
        issues.push(SignatureIssue::Missing);
    }

    if count > 0 && !enabled {
        issues.push(SignatureIssue::Unexpected);
    }

    issues
}

pub fn reconcile_signature(content: &str, enabled: bool) -> String {
    let newline = if content.contains("\r\n") {
        "\r\n"
    } else {
        "\n"
    };

    let windows_signature = CANONICAL_SIGNATURE.replace('\n', "\r\n");
    let without_signature = content
        .replace(CANONICAL_SIGNATURE, "")
        .replace(&windows_signature, "");
    let without_signature = without_signature.trim_end_matches(['\n', '\r']);

    if enabled {
        let signature = CANONICAL_SIGNATURE.replace('\n', newline);
        format!("{without_signature}{newline}{newline}{signature}{newline}")
    } else {
        format!("{without_signature}{newline}")
    }
}

#[cfg(test)]
mod tests {
    use super::{CANONICAL_SIGNATURE, SignatureIssue, issues, reconcile_signature};

    #[test]
    fn reconciles_enabled_and_disabled_documents() {
        let content = "# AGENTS.md\n";
        let enabled = reconcile_signature(content, true);
        assert!(enabled.trim_end().ends_with(CANONICAL_SIGNATURE));
        assert!(enabled.ends_with('\n'));
        assert_eq!(reconcile_signature(&enabled, false), content);
    }

    #[test]
    fn detects_signature_shapes() {
        let content = format!("# AGENTS.md\n\n{CANONICAL_SIGNATURE}\n");
        assert!(issues(&content, true).is_empty());
        assert!(issues("# AGENTS.md\n", true).contains(&SignatureIssue::Missing));
        assert!(
            issues(&format!("{content}{CANONICAL_SIGNATURE}\n"), true)
                .contains(&SignatureIssue::Duplicate)
        );
        assert!(
            issues(
                "# AGENTS.md\n\n> Generated and maintained by [Agentskill]",
                true
            )
            .contains(&SignatureIssue::Malformed)
        );
        assert!(
            issues(&format!("{content}\nMore text\n"), true).contains(&SignatureIssue::NonTerminal)
        );
        assert!(issues(&content, false).contains(&SignatureIssue::Unexpected));
    }

    #[test]
    fn accepts_and_preserves_windows_line_endings() {
        let content = format!("# AGENTS.md\n\n{CANONICAL_SIGNATURE}\n");
        let windows_content = content.replace('\n', "\r\n");

        assert!(issues(&windows_content, true).is_empty());
        assert_eq!(reconcile_signature(&windows_content, true), windows_content);
        assert_eq!(
            reconcile_signature(&windows_content, false),
            "# AGENTS.md\r\n"
        );
    }
}
