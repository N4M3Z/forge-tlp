const TLP_RED_MARKER: &str = "#tlp/red";
const TLP_BOUNDARY_TAGS: &[&str] = &["#tlp/amber", "#tlp/green", "#tlp/clear"];

use regex::Regex;
use std::sync::OnceLock;

/// Secret detection patterns curated from [gitleaks](https://github.com/gitleaks/gitleaks).
/// Each pattern targets a specific service's token format. Combined into a single
/// alternation and compiled once via `OnceLock`.
#[rustfmt::skip]
const SECRET_PATTERNS: &str = concat!(
    // AI/ML platforms
    r"sk-ant-api\d{2}-[a-zA-Z0-9_-]{20,}",              // Anthropic
    "|", r"sk-proj-[a-zA-Z0-9]{20,}",                    // OpenAI project key
    "|", r"sk-or-[a-zA-Z0-9_-]{20,}",                    // OpenRouter
    // Cloud providers
    "|", r"AKIA[0-9A-Z]{16}",                             // AWS access key ID
    "|", r"AIza[0-9A-Za-z_-]{35}",                        // GCP API key
    // Code hosting — GitHub
    "|", r"ghp_[0-9a-zA-Z]{36}",                          // GitHub PAT
    "|", r"gho_[0-9a-zA-Z]{36}",                          // GitHub OAuth
    "|", r"ghs_[0-9a-zA-Z]{36,}",                         // GitHub server-to-server
    "|", r"ghu_[0-9a-zA-Z]{36}",                          // GitHub user-to-server
    "|", r"github_pat_[0-9a-zA-Z_]{82}",                  // GitHub fine-grained PAT
    // Code hosting — GitLab
    "|", r"glpat-[0-9a-zA-Z_-]{20,}",                     // GitLab PAT
    "|", r"glptt-[0-9a-f]{40}",                            // GitLab pipeline trigger
    "|", r"GR1348941[0-9a-zA-Z_-]{20,}",                  // GitLab runner registration
    // Communication — Slack
    "|", r"xoxb-[0-9]+-[0-9A-Za-z-]+",                    // Slack bot token
    "|", r"xoxp-[0-9]+-[0-9A-Za-z-]+",                    // Slack user token
    "|", r"xoxa-[0-9]+-[0-9A-Za-z-]+",                    // Slack app token
    "|", r"xoxe-[0-9]+-[0-9A-Za-z-]+",                    // Slack config token
    // Payment — Stripe
    "|", r"(?:sk|rk)_(?:live|test|prod)_[0-9a-zA-Z]{24,}", // Stripe secret/restricted key
    // Package registries
    "|", r"npm_[0-9a-zA-Z]{36}",                           // npm access token
    "|", r"pypi-[0-9a-zA-Z_-]{16,}",                       // PyPI API token
    // SaaS tools
    "|", r"SG\.[0-9a-zA-Z_-]{22}\.[0-9a-zA-Z_-]{43}",     // SendGrid API key
    "|", r"SK[0-9a-fA-F]{32}",                             // Twilio API key
    "|", r"PMAK-[0-9a-fA-F]{24}-[0-9a-fA-F]{34}",         // Postman API key
    "|", r"lin_api_[a-zA-Z0-9]{40}",                       // Linear API key
    "|", r"dp\.pt\.[a-zA-Z0-9]{43}",                       // Doppler CLI token
    "|", r"dapi[0-9a-f]{32}",                               // Databricks access token
    // Infrastructure — DigitalOcean
    "|", r"dop_v1_[a-f0-9]{64}",                           // DigitalOcean PAT
    "|", r"doo_v1_[a-f0-9]{64}",                           // DigitalOcean OAuth
    "|", r"dor_v1_[a-f0-9]{64}",                           // DigitalOcean refresh
    // Infrastructure — Hashicorp Vault
    "|", r"hvs\.[a-zA-Z0-9_-]{24,}",                       // Vault service token
    "|", r"hvb\.[a-zA-Z0-9_-]{100,}",                      // Vault batch token
    // Infrastructure — other
    "|", r"pul-[a-f0-9]{40}",                               // Pulumi access token
    // E-commerce — Shopify
    "|", r"shpss_[0-9a-fA-F]{32}",                         // Shopify shared secret
    "|", r"shpat_[0-9a-fA-F]{32}",                         // Shopify access token
    "|", r"shpca_[0-9a-fA-F]{32}",                         // Shopify custom app
    "|", r"shppa_[0-9a-fA-F]{32}",                         // Shopify private app
    // Databases
    "|", r"mongodb(?:\+srv)?://[^:@\s]{3,}:[^@\s]{3,}@[^\s]+", // MongoDB with creds
    // Monitoring — Grafana
    "|", r"glc_[A-Za-z0-9+/]{32,}={0,2}",                  // Grafana Cloud API key
    "|", r"glsa_[A-Za-z0-9]{32}_[A-Fa-f0-9]{8}",           // Grafana service account
    // Platform
    "|", r"pscale_tkn_[a-zA-Z0-9_.-]{43}",                 // PlanetScale token
    "|", r"pscale_oauth_[a-zA-Z0-9_.-]{43}",               // PlanetScale OAuth
    // CMS
    "|", r"CFPAT-[a-zA-Z0-9_-]{43}",                       // Contentful PAT
    // Encryption
    "|", r"AGE-SECRET-KEY-1[qpzry9x8gf2tvdw0s3jn54khce6mua7l]{58}", // age secret key
    "|", r"-----BEGIN[A-Z ]*PRIVATE KEY-----",              // PEM private key header
);

/// Compiled regex for secret detection.
fn secret_regex() -> &'static Regex {
    static RE: OnceLock<Regex> = OnceLock::new();
    RE.get_or_init(|| Regex::new(SECRET_PATTERNS).expect("secret patterns must compile"))
}

/// All TLP markers (red + boundary tags), used for inline detection.
const ALL_TLP_MARKERS: &[&str] = &["#tlp/red", "#tlp/amber", "#tlp/green", "#tlp/clear"];

/// Check if a trimmed line is a TLP boundary tag (not #tlp/red, which starts sections).
fn is_tlp_boundary(trimmed: &str) -> bool {
    TLP_BOUNDARY_TAGS.contains(&trimmed)
}

/// Strip content between #tlp/red and any other #tlp/* boundary marker.
/// Each RED section is replaced with a single [REDACTED] line.
///
/// Supports two modes:
/// - **Block mode**: `#tlp/red` alone on a line starts a multi-line redacted section,
///   ended by any `#tlp/*` boundary tag alone on a line.
/// - **Inline mode**: `#tlp/red` mid-line redacts from the marker to the next
///   `#tlp/*` boundary tag on the same line, or to end of line if none found.
pub fn redact_tlp_sections(content: &str) -> String {
    let mut result = Vec::new();
    let mut in_redacted = false;
    let mut redaction_emitted = false;

    for line in content.lines() {
        let trimmed = line.trim();

        // Block mode: whole-line #tlp/red starts multi-line redaction
        if trimmed == TLP_RED_MARKER {
            if !in_redacted {
                in_redacted = true;
                redaction_emitted = false;
            }
            continue;
        }

        // Block mode: whole-line boundary tag ends multi-line redaction
        if in_redacted && is_tlp_boundary(trimmed) {
            if !redaction_emitted {
                result.push("[REDACTED]".to_string());
            }
            in_redacted = false;
            continue;
        }

        if in_redacted {
            if !redaction_emitted {
                result.push("[REDACTED]".to_string());
                redaction_emitted = true;
            }
        } else {
            // Inline mode: check for #tlp/red mid-line
            result.push(redact_inline_markers(line));
        }
    }

    // Handle unterminated RED block
    if in_redacted && !redaction_emitted {
        result.push("[REDACTED]".to_string());
    }

    let mut output = result.join("\n");
    if content.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// Process a single line for inline `#tlp/red` markers.
/// Redacts from each `#tlp/red` to the next `#tlp/*` boundary tag, or to end of line.
fn redact_inline_markers(line: &str) -> String {
    if !line.contains(TLP_RED_MARKER) {
        return line.to_string();
    }

    let mut result = String::new();
    let mut remaining = line;

    while let Some(red_pos) = remaining.find(TLP_RED_MARKER) {
        // Keep everything before the marker
        result.push_str(&remaining[..red_pos]);

        // Skip past the #tlp/red marker
        let after_marker = &remaining[red_pos + TLP_RED_MARKER.len()..];

        // Find the closest boundary tag on this line
        let mut closest: Option<(usize, usize)> = None; // (position, tag_len)
        for &tag in ALL_TLP_MARKERS {
            if tag == TLP_RED_MARKER {
                continue; // Don't match another #tlp/red as a boundary
            }
            if let Some(pos) = after_marker.find(tag) {
                match closest {
                    None => closest = Some((pos, tag.len())),
                    Some((prev, _)) if pos < prev => closest = Some((pos, tag.len())),
                    _ => {}
                }
            }
        }

        result.push_str("[REDACTED]");

        match closest {
            Some((pos, tag_len)) => {
                remaining = &after_marker[pos + tag_len..];
            }
            None => {
                // No boundary — redact to end of line
                remaining = "";
            }
        }
    }

    result.push_str(remaining);
    result
}

/// Scan content for known secret patterns and redact them.
/// Returns `(redacted_content, secrets_found)`.
pub fn redact_secrets(content: &str) -> (String, bool) {
    let re = secret_regex();
    let mut found = false;
    let mut lines = Vec::new();

    for line in content.lines() {
        if re.is_match(line) {
            found = true;
            lines.push(re.replace_all(line, "[SECRET REDACTED]").into_owned());
        } else {
            lines.push(line.to_string());
        }
    }

    let mut output = lines.join("\n");
    if content.ends_with('\n') {
        output.push('\n');
    }
    (output, found)
}

#[cfg(test)]
mod tests;
