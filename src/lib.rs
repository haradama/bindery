use std::fs;
use std::path::{Path, PathBuf};

use ignore::WalkBuilder;
use ignore::overrides::OverrideBuilder;
use tokei::{Config, LanguageType};

pub mod scanner {
    use super::*;

    pub struct CodeScanner {
        paths: Vec<PathBuf>,
        excluded: Vec<String>,
        include_hidden: bool,
        strip_comments: bool,
        output_path: Option<PathBuf>,
    }

    impl CodeScanner {
        pub fn new(
            paths: Vec<PathBuf>,
            excluded: Vec<String>,
            include_hidden: bool,
            strip_comments: bool,
            output_path: Option<PathBuf>,
        ) -> Self {
            Self {
                paths,
                excluded,
                include_hidden,
                strip_comments,
                output_path,
            }
        }

        pub fn concatenate(&self) -> std::io::Result<String> {
            let mut items: Vec<(String, String, LanguageType)> = vec![];

            let output_canon = self
                .output_path
                .as_ref()
                .and_then(|p| p.canonicalize().ok());

            for base in &self.paths {
                let mut builder = WalkBuilder::new(base);
                builder.follow_links(false);

                if self.include_hidden {
                    builder.hidden(false);
                }

                if !self.excluded.is_empty() {
                    let mut ov = OverrideBuilder::new(base);
                    for pat in &self.excluded {
                        ov.add(&format!("!{pat}")).map_err(|e| {
                            std::io::Error::new(std::io::ErrorKind::InvalidInput, e)
                        })?;
                    }
                    let overrides = ov
                        .build()
                        .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidInput, e))?;
                    builder.overrides(overrides);
                }

                for result in builder.build() {
                    let entry = match result {
                        Ok(e) => e,
                        Err(_) => continue,
                    };

                    if !entry.file_type().map(|ft| ft.is_file()).unwrap_or(false) {
                        continue;
                    }

                    let path = entry.path();

                    if let Some(out) = &output_canon
                        && let Ok(p) = path.canonicalize()
                        && &p == out
                    {
                        continue;
                    }

                    if let Some(lang) = LanguageType::from_path(path, &Config::default())
                        && let Ok(mut content) = fs::read_to_string(path)
                    {
                        if self.strip_comments {
                            content = remove_comments(&content, lang);
                        }
                        let rel = relative_display(path);
                        items.push((rel, content, lang));
                    }
                }
            }

            items.sort_by(|a, b| a.0.cmp(&b.0));

            let mut out = String::new();
            for (i, (path_str, content, lang)) in items.into_iter().enumerate() {
                if i > 0 {
                    out.push('\n');
                }
                out.push_str(&path_str);
                out.push_str("\n\n");
                out.push_str("```");
                out.push_str(language_name(lang));
                out.push('\n');
                out.push_str(&content);
                if !content.ends_with('\n') {
                    out.push('\n');
                }
                out.push_str("```\n");
            }

            Ok(out)
        }
    }

    /// Remove comments using language-aware rules from `tokei`.
    pub fn remove_comments(content: &str, language_type: LanguageType) -> String {
        let mut out_lines: Vec<String> = Vec::new();

        let mut in_multiline_comment = false;
        let mut multiline_comment_end = "";

        for line in content.lines() {
            let was_in_multiline_comment = in_multiline_comment;

            let original_has_non_ws = line.chars().any(|c| !c.is_whitespace());

            let processed_line = if in_multiline_comment {
                process_line_in_multiline_comment(
                    line,
                    language_type,
                    &mut in_multiline_comment,
                    &mut multiline_comment_end,
                )
            } else {
                process_line_normal(
                    line,
                    language_type,
                    &mut in_multiline_comment,
                    &mut multiline_comment_end,
                )
            };

            let processed_is_ws_only = processed_line.trim().is_empty();

            if processed_is_ws_only {
                if was_in_multiline_comment {
                    continue;
                }
                if original_has_non_ws {
                    continue;
                }
            }

            out_lines.push(processed_line);
        }

        out_lines.join("\n")
    }

    fn process_line_normal(
        line: &str,
        language_type: LanguageType,
        in_multiline: &mut bool,
        multiline_end: &mut &str,
    ) -> String {
        let mut result = String::new();
        let chars = line.chars().collect::<Vec<char>>();
        let mut i = 0;
        let mut in_string = false;
        let mut string_delim = "";

        while i < chars.len() {
            let remaining: String = chars[i..].iter().collect();

            // String start?
            if !in_string {
                let mut found = false;
                for &(start, end) in language_type.quotes() {
                    if remaining.starts_with(start) {
                        in_string = true;
                        string_delim = end;
                        // Append start token literally
                        for _ in start.chars() {
                            if i < chars.len() {
                                result.push(chars[i]);
                                i += 1;
                            }
                        }
                        found = true;
                        break;
                    }
                }
                if found {
                    continue;
                }
            } else if remaining.starts_with(string_delim) {
                // String end
                for _ in string_delim.chars() {
                    if i < chars.len() {
                        result.push(chars[i]);
                        i += 1;
                    }
                }
                in_string = false;
                continue;
            }

            if in_string {
                // Handle escapes
                if chars[i] == '\\' && i + 1 < chars.len() {
                    result.push(chars[i]);
                    result.push(chars[i + 1]);
                    i += 2;
                } else {
                    result.push(chars[i]);
                    i += 1;
                }
                continue;
            }

            // Multiline comment start
            let mut consumed = false;
            for &(start, end) in language_type.multi_line_comments() {
                if remaining.starts_with(start) {
                    *in_multiline = true;
                    *multiline_end = end;
                    consumed = true;
                    i += start.len();

                    // If it also ends on same line, eat until end token
                    let rest = chars[i..].iter().collect::<String>();
                    if let Some(pos) = rest.find(end) {
                        i += pos + end.len();
                        *in_multiline = false;
                    } else {
                        return result;
                    }
                    break;
                }
            }
            if consumed {
                continue;
            }

            // Nested comments (if the language has them)
            for &(start, end) in language_type.nested_comments() {
                if remaining.starts_with(start) {
                    *in_multiline = true;
                    *multiline_end = end;
                    consumed = true;
                    i += start.len();
                    let rest = chars[i..].iter().collect::<String>();
                    if let Some(pos) = rest.find(end) {
                        i += pos + end.len();
                        *in_multiline = false;
                    } else {
                        return result;
                    }
                    break;
                }
            }
            if consumed {
                continue;
            }

            // Line comments
            for &start in language_type.line_comments() {
                if remaining.starts_with(start) {
                    return result;
                }
            }

            // Normal char
            result.push(chars[i]);
            i += 1;
        }

        result
    }

    fn process_line_in_multiline_comment(
        line: &str,
        language_type: LanguageType,
        in_multiline: &mut bool,
        multiline_end: &mut &str,
    ) -> String {
        if let Some(end_pos) = line.find(*multiline_end) {
            *in_multiline = false;
            let remaining = &line[end_pos + multiline_end.len()..];
            return process_line_normal(remaining, language_type, in_multiline, multiline_end);
        }
        String::new()
    }

    fn relative_display(path: &Path) -> String {
        if let Ok(cwd) = std::env::current_dir()
            && let Ok(rel) = path.strip_prefix(&cwd)
        {
            return to_forward_slash(rel);
        }
        to_forward_slash(path)
    }

    fn to_forward_slash(p: &Path) -> String {
        let s = p.to_string_lossy().to_string();
        s.replace('\\', "/")
    }

    pub fn language_name(lang: LanguageType) -> &'static str {
        match lang {
            LanguageType::Rust => "rust",
            LanguageType::TypeScript => "ts",
            LanguageType::JavaScript => "js",
            LanguageType::Python => "python",
            LanguageType::C => "c",
            LanguageType::Cpp => "cpp",
            LanguageType::Go => "go",
            LanguageType::Java => "java",
            LanguageType::Kotlin => "kotlin",
            LanguageType::Swift => "swift",
            LanguageType::Ruby => "ruby",
            LanguageType::Scala => "scala",
            LanguageType::Haskell => "haskell",
            _ => "text",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::scanner::*;
    use tokei::LanguageType;

    #[test]
    fn rust_comment_removal_preserves_strings() {
        let content = r#"
// This is a line comment
fn main() {
    let x = 5; // inline
    /* block
       comment */
    println!("Hello // not a comment");
    let s = "/* also not a comment */";
}
"#;
        let result = remove_comments(content, LanguageType::Rust);
        assert!(!result.contains("// This is a line comment"));
        assert!(!result.contains("// inline"));
        assert!(!result.contains("/* block"));
        assert!(result.contains("Hello // not a comment"));
        assert!(result.contains("/* also not a comment */"));
    }

    #[test]
    fn c_comment_removal() {
        let content = r#"
/* Header comment */
#include <stdio.h>

int main(){
  // line
  printf("Hello\n"); /* inline */
}
"#;
        let result = remove_comments(content, LanguageType::C);
        assert!(!result.contains("/* Header comment */"));
        assert!(!result.contains("// line"));
        assert!(!result.contains("/* inline */"));
    }

    #[test]
    fn js_comment_removal() {
        let content = r#"
// top
function t(){
  var x = 5; // end
  /* multi
     line */
  console.log("// not a comment");
}
"#;
        let result = remove_comments(content, LanguageType::JavaScript);
        assert!(!result.contains("// top"));
        assert!(!result.contains("// end"));
        assert!(!result.contains("/* multi"));
        assert!(result.contains("// not a comment"));
    }

    #[test]
    fn multiline_comment_blank_lines_are_removed() {
        let content = "fn main() {\n/*\n   \n*/\nprintln!(\"x\");\n}\n";
        let result = remove_comments(content, LanguageType::Rust);

        assert!(result.contains("fn main() {"));
        assert!(result.contains("println!(\"x\");"));
        assert!(!result.contains("\n\n\n"));
    }
}
