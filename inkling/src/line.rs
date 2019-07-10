use crate::{
    consts::{
        CHOICE_MARKER, DIVERT_MARKER, GATHER_MARKER, GLUE_MARKER, STICKY_CHOICE_MARKER, TAG_MARKER,
    },
    error::{LineError, ParseError},
};

use std::str::FromStr;

#[derive(Clone, Debug, PartialEq)]
/// A single line of text used in a story. Can contain diverts to new knots, which should
/// be followed when walking through the story.
pub struct LineData {
    /// Text contained in the line.
    pub text: String,
    /// Contains what result following the line will have, either being a regular line
    /// or a divert to another part of the story.
    pub kind: LineKind,
    /// Tags marking the line. Can be used to process the line before displaying it.
    pub tags: Vec<String>,
    /// Glue represents how the line connects to a prior or following line. If there is glue,
    /// no newline should be added between them. `glue_start` denotes whether the line should
    /// glue to the previous line.
    pub glue_start: bool,
    /// Glue represents how the line connects to a prior or following line. If there is glue,
    /// no newline should be added between them. `glue_end` denotes whether the line should
    /// glue to the following line.
    pub glue_end: bool,
}

#[derive(Clone, Debug, PartialEq)]
/// A single choice in a (usually) set of choices presented to the user.
pub struct Choice {
    /// Text presented to the user to represent the choice.
    pub displayed: LineData,
    /// Text that the choice produces when selected, replacing the `displayed` line.
    /// Can be empty, in which case the presented text is removed before the story flow
    /// continues to the next line.
    pub line: LineData,
    /// Number of times the choice has been selected so far in the story.
    pub num_visited: u32,
    /// By default a choice will be filtered after being visited once. If it is marked
    /// as sticky it will stick around.
    pub is_sticky: bool,
}

#[derive(Clone, Debug, PartialEq)]
/// What action that is prompted by following a story.
pub enum LineKind {
    /// Move on with the story.
    Regular,
    /// Divert to a new knot with the given name.
    Divert(String),
}

#[derive(Clone, Debug)]
/// Denotes the behavior of a parsed line, used when constructing the graph of a set
/// of parsed lines.
pub enum ParsedLine {
    /// Parsed line is a choice presented to the user, with a set nesting `level`.
    Choice { level: u8, choice: Choice },
    /// Parsed line is a gather point for choices at set nesting `level`. All nodes
    /// with equal to or higher nesting `level`s will collapse here.
    Gather { level: u8, line: LineData },
    /// Regular line, which can still divert to other knots and have formatting.
    Line(LineData),
}

impl FromStr for ParsedLine {
    type Err = ParseError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        parse_choice(line)
            .or_else(|| parse_gather(line))
            .unwrap_or_else(|| parse_line(line))
    }
}

fn parse_line(line: &str) -> Result<ParsedLine, ParseError> {
    LineData::from_str(line).map(|line| ParsedLine::Line(line))
}

fn parse_choice(line: &str) -> Option<Result<ParsedLine, ParseError>> {
    let (level, is_sticky, line_text) = match parse_choice_markers_and_text(line)? {
        Ok(result) => result,
        Err(err) => {
            return Some(Err(err));
        }
    };

    if line_text.is_empty() {
        return Some(Err(LineError::NoDisplayText.into()));
    }

    match LineData::from_str(line_text) {
        Ok(line) => {
            let choice = Choice {
                displayed: line.clone(),
                line,
                num_visited: 0,
                is_sticky,
            };

            Some(Ok(ParsedLine::Choice { level, choice }))
        }
        Err(err) => Some(Err(err)),
    }
}

/// Split choice markers (sticky or non-sticky) from a line. If they are present, ensure
/// that the line does not have both sticky and non-sticky markers. Return the number
/// of markers along with whether the choice was sticky and the remaining line.
fn parse_choice_markers_and_text(line: &str) -> Option<Result<(u8, bool, &str), ParseError>> {
    let choice_parse = parse_markers_and_text(line, CHOICE_MARKER);
    let is_sticky = choice_parse.is_none();

    let (num_markers, remaining_line) =
        choice_parse.or_else(|| parse_markers_and_text(line, STICKY_CHOICE_MARKER))?;

    if remaining_line.starts_with(|c| c == CHOICE_MARKER || c == STICKY_CHOICE_MARKER) {
        return Some(Err(LineError::MultipleChoiceType {
            line: line.to_string(),
        }
        .into()));
    }

    Some(Ok((num_markers, is_sticky, remaining_line)))
}

fn parse_gather(line: &str) -> Option<Result<ParsedLine, ParseError>> {
    let line_minus_diverts = line.trim_start().trim_start_matches(DIVERT_MARKER);
    let parsed_gather = parse_markers_and_text(line_minus_diverts, GATHER_MARKER);

    parsed_gather.map(|(level, line_text)| match LineData::from_str(line_text) {
        Ok(line) => Ok(ParsedLine::Gather { level, line }),
        Err(err) => Err(err),
    })
}

fn parse_markers_and_text(line: &str, marker: char) -> Option<(u8, &str)> {
    if line.trim_start().starts_with(marker) {
        let (markers, line_text) = split_markers_from_text(line, marker);
        let num = markers.matches(|c| c == marker).count() as u8;

        Some((num, line_text))
    } else {
        None
    }
}

fn split_markers_from_text(line: &str, marker: char) -> (&str, &str) {
    let split_at = line.find(|c: char| !(c == marker || c.is_whitespace()));

    match split_at {
        Some(i) => line.split_at(i),
        None => (line, ""),
    }
}

impl FromStr for LineData {
    type Err = ParseError;

    fn from_str(line: &str) -> Result<Self, Self::Err> {
        let mut text = trim_whitespace(line);

        let tags = parse_tags(&mut text);
        let divert = parse_divert(&mut text);

        text = text.trim().to_string();

        // Diverts always act as glue
        let (glue_start, glue_end) = parse_line_glue(&mut text, divert.is_some());

        let kind = if let Some(name) = divert {
            LineKind::Divert(name)
        } else {
            LineKind::Regular
        };

        Ok(LineData {
            text,
            kind,
            tags,
            glue_start,
            glue_end,
        })
    }
}

fn trim_whitespace(line: &str) -> String {
    let words: Vec<&str> = line.split_whitespace().collect();
    words.join(" ")
}

/// Parse and remove glue markers from either side, retaining enclosed whitespace.
/// A divert always acts as right glue.
fn parse_line_glue(line: &mut String, has_divert: bool) -> (bool, bool) {
    let glue_left = line.starts_with(GLUE_MARKER);
    let glue_right = line.ends_with(GLUE_MARKER);

    if glue_left {
        *line = line.trim_start_matches(GLUE_MARKER).to_string();
    }

    if glue_right {
        *line = line.trim_end_matches(GLUE_MARKER).to_string();
    }

    if has_divert && !glue_right {
        line.push(' ');
    }

    (glue_left, glue_right || has_divert)
}

/// Split diverts off the given line and return it separately if found.
fn parse_divert(line: &mut String) -> Option<String> {
    match line.find(DIVERT_MARKER) {
        Some(i) => {
            let part = line.split_off(i);

            part.trim_start_matches(DIVERT_MARKER)
                .split(DIVERT_MARKER)
                .map(|knot_name| knot_name.trim().to_string())
                .next()
        }
        None => None,
    }
}

/// Split any found tags off the given line and return them separately.
fn parse_tags(line: &mut String) -> Vec<String> {
    match line.find(TAG_MARKER) {
        Some(i) => {
            let part = line.split_off(i);

            part.trim_matches(TAG_MARKER)
                .split(TAG_MARKER)
                .map(|tag| tag.to_string())
                .collect::<Vec<_>>()
        }
        None => Vec::new(),
    }
}

#[cfg(test)]
pub(crate) mod tests {
    use super::*;

    impl ParsedLine {
        fn choice(self) -> (u8, Choice) {
            match self {
                ParsedLine::Choice { level, choice } => (level, choice),
                _ => panic!("tried to unwrap into a `Choice`, but was `{:?}`", &self),
            }
        }

        fn gather(self) -> (u8, LineData) {
            match self {
                ParsedLine::Gather { level, line } => (level, line),
                _ => panic!("tried to unwrap into a `Gather`, but was `{:?}`", &self),
            }
        }

        fn line(self) -> LineData {
            match self {
                ParsedLine::Line(line) => line,
                _ => panic!("tried to unwrap into a `LineData`, but was `{:?}`", &self),
            }
        }
    }

    impl Choice {
        pub fn empty() -> Self {
            let line = LineData::empty();
            Choice {
                displayed: line.clone(),
                line: line.clone(),
                num_visited: 0,
                is_sticky: false,
            }
        }
    }

    pub struct ChoiceBuilder {
        line: LineData,
        displayed: LineData,
        num_visited: u32,
        is_sticky: bool,
    }

    impl ChoiceBuilder {
        pub fn empty() -> Self {
            let line = LineData::empty();

            ChoiceBuilder {
                displayed: line.clone(),
                line,
                num_visited: 0,
                is_sticky: false,
            }
        }

        pub fn build(self) -> Choice {
            Choice {
                displayed: self.displayed,
                line: self.line,
                num_visited: self.num_visited,
                is_sticky: self.is_sticky,
            }
        }

        pub fn is_sticky(mut self) -> Self {
            self.is_sticky = true;
            self
        }

        pub fn with_displayed(mut self, line: LineData) -> Self {
            self.displayed = line;
            self
        }

        pub fn with_line(mut self, line: LineData) -> Self {
            self.line = line;
            self
        }

        pub fn with_num_visited(mut self, num_visited: u32) -> Self {
            self.num_visited = num_visited;
            self
        }
    }

    impl LineData {
        pub fn empty() -> Self {
            LineData {
                text: String::new(),
                kind: LineKind::Regular,
                tags: Vec::new(),
                glue_start: false,
                glue_end: false,
            }
        }
    }

    pub struct LineBuilder {
        text: String,
        kind: LineKind,
        tags: Vec<String>,
        glue_start: bool,
        glue_end: bool,
    }

    impl LineBuilder {
        pub fn new(text: &str) -> Self {
            LineBuilder {
                text: text.to_string(),
                kind: LineKind::Regular,
                tags: Vec::new(),
                glue_start: false,
                glue_end: false,
            }
        }

        pub fn build(self) -> LineData {
            LineData {
                text: self.text,
                kind: self.kind,
                tags: self.tags,
                glue_start: self.glue_start,
                glue_end: self.glue_end,
            }
        }

        pub fn with_divert(mut self, to_knot: &str) -> Self {
            self.kind = LineKind::Divert(to_knot.to_string());
            self
        }

        pub fn with_glue_start(mut self) -> Self {
            self.glue_start = true;
            self
        }

        pub fn with_glue_end(mut self) -> Self {
            self.glue_end = true;
            self
        }

        pub fn with_tags(mut self, tags: Vec<String>) -> Self {
            self.tags = tags;
            self
        }
    }

    #[test]
    fn simple_line_parses_to_line() {
        let text = "Hello, world!";

        let line = ParsedLine::from_str(text).unwrap().line();
        assert_eq!(&line.text, text);
    }

    #[test]
    fn line_with_choice_markers_parses_into_choice_with_correct_level() {
        let line_text = "Hello, world!";

        let text1 = format!("* {}", line_text);
        let (level, choice) = ParsedLine::from_str(&text1).unwrap().choice();

        assert_eq!(level, 1);
        assert_eq!(choice.line, LineData::from_str(line_text).unwrap());

        let text2 = format!("** {}", line_text);
        let (level, choice) = ParsedLine::from_str(&text2).unwrap().choice();

        assert_eq!(level, 2);
        assert_eq!(choice.line, LineData::from_str(line_text).unwrap());
    }

    #[test]
    fn parsing_choice_sets_displayed_and_line() {
        let line_text = "Hello, world!";
        let choice_text = format!("* {}", line_text);

        let (_, choice) = parse_choice(&choice_text).unwrap().unwrap().choice();

        assert_eq!(&choice.displayed, &choice.line);
    }

    #[test]
    fn choices_are_initialized_with_zero_visits() {
        let line_text = "Hello, world!";
        let choice_text = format!("* {}", line_text);

        let (_, choice) = parse_choice(&choice_text).unwrap().unwrap().choice();

        assert_eq!(choice.num_visited, 0);
    }

    #[test]
    fn line_with_gather_markers_counts_them() {
        let line_text = "Hello, world!";

        let text1 = format!("- {}", line_text);
        let (level, line) = ParsedLine::from_str(&text1).unwrap().gather();

        assert_eq!(level, 1);
        assert_eq!(line, LineData::from_str(line_text).unwrap());

        let text2 = format!("-- {}", line_text);
        let (level, line) = ParsedLine::from_str(&text2).unwrap().gather();

        assert_eq!(level, 2);
        assert_eq!(line, LineData::from_str(line_text).unwrap());
    }

    #[test]
    fn asterix_choice_marker_returns_non_sticky_choice() {
        let (_, choice) = ParsedLine::from_str("* Non-sticky choice")
            .unwrap()
            .choice();
        assert!(!choice.is_sticky);
    }

    #[test]
    fn plus_choice_marker_returns_sticky_choice() {
        let (_, choice) = ParsedLine::from_str("+ Non-sticky choice")
            .unwrap()
            .choice();
        assert!(choice.is_sticky);
    }

    #[test]
    fn mix_of_sticky_and_non_sticky_marker_returns_error() {
        assert!(ParsedLine::from_str("+* Some choice ???").is_err());
        assert!(ParsedLine::from_str("*+ Some choice ???").is_err());
        assert!(ParsedLine::from_str("+++ * Some choice ???").is_err());
        assert!(ParsedLine::from_str("**+Some choice ???").is_err());
    }

    #[test]
    fn choice_markers_require_text() {
        assert!(ParsedLine::from_str("*").is_err());
    }

    #[test]
    fn gather_markers_do_not_require_text() {
        assert!(ParsedLine::from_str("-").is_ok());
    }

    #[test]
    fn markers_can_be_whitespace_separated() {
        let line_text = "Hello, world!";

        let text = format!("- -    - - {}", line_text);
        let (level, line) = ParsedLine::from_str(&text).unwrap().gather();

        assert_eq!(level, 4);
        assert_eq!(line, LineData::from_str(line_text).unwrap());
    }

    #[test]
    fn line_with_beginning_divert_parses_into_line_instead_of_gather() {
        let knot_name = "knot_name";
        let text = format!("    {} {}", DIVERT_MARKER, knot_name);
        let line = ParsedLine::from_str(&text).unwrap().line();

        assert_eq!(line.kind, LineKind::Divert(knot_name.to_string()));
    }

    #[test]
    fn read_simple_line() {
        let text = "Hello, world!";

        let line = LineData::from_str(text).unwrap();

        assert_eq!(&line.text, text);
        assert_eq!(line.kind, LineKind::Regular);
    }

    #[test]
    fn read_line_trims_whitespace() {
        let text = "   Hello, world!   ";
        let line = LineData::from_str(text).unwrap();

        assert_eq!(&line.text, text.trim());
    }

    #[test]
    fn line_with_glue_retains_whitespace_on_end_side() {
        let text = "Hello, world!";
        let whitespace = "    ";

        let line_with_left_glue = format!(
            "{marker}{pad}{text}",
            pad = &whitespace,
            text = &text,
            marker = GLUE_MARKER
        );

        let line_with_right_glue = format!(
            "{text}{pad}{marker}",
            text = &text,
            pad = &whitespace,
            marker = GLUE_MARKER
        );

        let line_left = LineData::from_str(&line_with_left_glue).unwrap();

        assert_eq!(line_left.text, format!(" {}", &text));
        assert!(line_left.glue_start);
        assert!(!line_left.glue_end);

        let line_right = LineData::from_str(&line_with_right_glue).unwrap();

        assert_eq!(line_right.text, format!("{} ", &text));
        assert!(!line_right.glue_start);
        assert!(line_right.glue_end);
    }

    #[test]
    fn divert_line_returns_knot_name() {
        let name = "knot_name";
        let text = format!("-> {}", name);

        let line = LineData::from_str(&text).unwrap();

        assert_eq!(line.kind, LineKind::Divert(name.to_string()));
    }

    #[test]
    fn embedded_divert_returns_knot_name() {
        let head = "Hello, world!";
        let name = "knot_name";
        let text = format!("{}->{}", head, name);

        let line = LineData::from_str(&text).unwrap();
        assert_eq!(line.kind, LineKind::Divert(name.to_string()));
    }

    #[test]
    fn diverts_in_lines_acts_as_glue() {
        let head = "Hello, world! ";
        let name = "knot_name";
        let text = format!("{}->{}", head, name);

        let line = LineData::from_str(&text).unwrap();

        assert!(line.glue_end);
    }

    #[test]
    fn lines_trim_extra_whitespace_between_words() {
        let line = LineData::from_str("Hello,      World!   ").unwrap();
        assert_eq!(&line.text, "Hello, World!");
    }

    #[test]
    fn tags_are_not_added_if_none_are_given() {
        let head = "Hello, world! ";
        let name = "knot_name";
        let text = format!("{}->{}", head, name);

        let line = LineData::from_str(&text).unwrap();
        assert!(line.tags.is_empty());
    }

    #[test]
    fn multiple_tags_can_be_specified() {
        let head = "Hello, world!";

        let tag1 = "blue colour".to_string();
        let tag2 = "transparent".to_string();
        let tag3 = "italic text".to_string();

        let text = format!(
            "{head}{sep}{}{sep}{}{sep}{}",
            tag1,
            tag2,
            tag3,
            head = head,
            sep = TAG_MARKER
        );

        let line = LineData::from_str(&text).unwrap();
        let tags = &line.tags;

        assert_eq!(tags.len(), 3);
        assert_eq!(tags[0], tag1);
        assert_eq!(tags[1], tag2);
        assert_eq!(tags[2], tag3);
    }
}
