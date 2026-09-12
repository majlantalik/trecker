//! RFC 4180 reading and writing.
//!
//! Hand-rolled rather than taken as a dependency, for the same reason the uuid and date
//! helpers in `repo/mod.rs` are: this is a hundred lines of well-specified behaviour that
//! is fully covered by tests, against a crate that would pull its own serde derive
//! machinery into a binary optimised for size.
//!
//! Records are separated by `\n`. RFC 4180 asks for CRLF and every reader worth naming
//! accepts either; a file written here is read on the machine that wrote it far more
//! often than it is read by Excel on Windows.

/// Appends one record. Fields are quoted only where the content forces it, so a normal
/// row stays readable in a text editor.
pub fn write_record<'a>(out: &mut String, fields: impl IntoIterator<Item = &'a str>) {
    let mut first = true;
    for field in fields {
        if !first {
            out.push(',');
        }
        first = false;
        write_field(out, field);
    }
    out.push('\n');
}

fn write_field(out: &mut String, field: &str) {
    if !field.contains([',', '"', '\n', '\r']) {
        out.push_str(field);
        return;
    }
    out.push('"');
    for c in field.chars() {
        if c == '"' {
            out.push('"');
        }
        out.push(c);
    }
    out.push('"');
}

/// Parses a whole document into records.
///
/// Blank lines are skipped rather than read as a record of one empty field, because a
/// trailing newline and a stray blank line look identical and neither is a release.
pub fn parse(input: &str) -> Result<Vec<Vec<String>>, String> {
    // Excel writes a byte order mark. Stripping it costs one line and its absence costs a
    // first column header that never matches anything.
    let input = input.strip_prefix('\u{feff}').unwrap_or(input);

    let mut records: Vec<Vec<String>> = Vec::new();
    let mut record: Vec<String> = Vec::new();
    let mut field = String::new();
    let mut chars = input.chars().peekable();
    let mut in_quotes = false;
    let mut quoted = false;
    let mut started = false;
    let mut line = 1usize;

    while let Some(c) = chars.next() {
        if in_quotes {
            match c {
                '"' => {
                    if chars.peek() == Some(&'"') {
                        chars.next();
                        field.push('"');
                    } else {
                        in_quotes = false;
                    }
                }
                '\n' => {
                    line += 1;
                    field.push(c);
                }
                _ => field.push(c),
            }
            continue;
        }

        match c {
            '"' if quoted => {
                return Err(format!("line {line}: text after a closing quote"));
            }
            '"' if field.is_empty() => {
                in_quotes = true;
                quoted = true;
                started = true;
            }
            '"' => return Err(format!("line {line}: a quote inside an unquoted field")),
            ',' => {
                record.push(std::mem::take(&mut field));
                quoted = false;
                started = true;
            }
            '\r' if chars.peek() == Some(&'\n') => {}
            '\n' => {
                line += 1;
                if started {
                    record.push(std::mem::take(&mut field));
                    records.push(std::mem::take(&mut record));
                }
                quoted = false;
                started = false;
            }
            _ if quoted => return Err(format!("line {line}: text after a closing quote")),
            _ => {
                field.push(c);
                started = true;
            }
        }
    }

    if in_quotes {
        return Err("the file ends inside a quoted value".into());
    }
    if started {
        record.push(field);
        records.push(record);
    }
    Ok(records)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn row(out: &mut String, fields: &[&str]) {
        write_record(out, fields.iter().copied());
    }

    #[test]
    fn writes_plain_fields_unquoted() {
        let mut s = String::new();
        row(&mut s, &["Slint", "Spiderland", "1991"]);
        assert_eq!(s, "Slint,Spiderland,1991\n");
    }

    #[test]
    fn quotes_only_what_needs_it() {
        let mut s = String::new();
        row(&mut s, &["Earth, Wind & Fire", "plain", "say \"hi\"", "two\nlines"]);
        assert_eq!(
            s,
            "\"Earth, Wind & Fire\",plain,\"say \"\"hi\"\"\",\"two\nlines\"\n"
        );
    }

    #[test]
    fn round_trips_everything_awkward() {
        // The four things that break a naive split(','): the delimiter, a quote, a
        // newline, and an empty value.
        let fields = vec![
            "Godspeed You! Black Emperor".to_string(),
            "Lift Your Skinny Fists, Like Antennas to Heaven".to_string(),
            "he said \"no\"".to_string(),
            "line one\nline two".to_string(),
            String::new(),
        ];
        let mut s = String::new();
        write_record(&mut s, fields.iter().map(String::as_str));
        assert_eq!(parse(&s).unwrap(), vec![fields]);
    }

    #[test]
    fn reads_crlf_and_lf_the_same() {
        let lf = parse("a,b\n1,2\n").unwrap();
        let crlf = parse("a,b\r\n1,2\r\n").unwrap();
        assert_eq!(lf, crlf);
        assert_eq!(lf, vec![vec!["a", "b"], vec!["1", "2"]]);
    }

    #[test]
    fn keeps_a_carriage_return_that_is_not_a_line_ending() {
        assert_eq!(parse("a\rb,c\n").unwrap(), vec![vec!["a\rb", "c"]]);
    }

    #[test]
    fn skips_blank_lines_including_the_trailing_one() {
        assert_eq!(parse("a\n\n\nb\n").unwrap(), vec![vec!["a"], vec!["b"]]);
    }

    #[test]
    fn a_row_of_empty_fields_is_still_a_row() {
        // Distinct from a blank line: the commas say there are three columns.
        assert_eq!(parse(",,\n").unwrap(), vec![vec!["", "", ""]]);
    }

    #[test]
    fn strips_the_byte_order_mark() {
        assert_eq!(parse("\u{feff}artist,title\n").unwrap(), vec![vec!["artist", "title"]]);
    }

    #[test]
    fn a_quoted_empty_value_is_a_field() {
        assert_eq!(parse("\"\",x\n").unwrap(), vec![vec!["", "x"]]);
    }

    #[test]
    fn rejects_a_file_that_ends_mid_quote() {
        // Truncation is the realistic cause, and reading it as a valid short field would
        // import a mangled note without saying so.
        assert!(parse("a,\"unterminated\n").is_err());
    }

    #[test]
    fn rejects_text_after_a_closing_quote() {
        assert!(parse("\"abc\"def\n").unwrap_err().contains("line 1"));
    }

    #[test]
    fn reports_the_line_a_problem_is_on() {
        let err = parse("a,b\nc,d\ne\"f\n").unwrap_err();
        assert!(err.contains("line 3"), "{err}");
    }

    #[test]
    fn counts_lines_across_a_quoted_newline() {
        let err = parse("a\n\"two\nlines\"\nx\"y\n").unwrap_err();
        assert!(err.contains("line 4"), "{err}");
    }
}
