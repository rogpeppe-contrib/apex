//! What a double-click needs to know of a file's language: where its
//! string literals and comments are. A click at a quote then takes the
//! literal that quote belongs to, and brackets inside a literal or a
//! comment don't count against those outside (`node::double_click_lang`).
//! It is a lexer for literals, comments and escapes, not a parser.

use crate::text::Text;

/// A language with a lexer.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Lang {
    Go,
}

/// The languages with a lexer, by file extension. apex-tool-lsp's
/// `BUILTIN` table maps the same extensions to its language servers;
/// a language added here should take the extensions it has there.
const EXTS: &[(Lang, &[&str])] = &[(Lang::Go, &["go"])];

/// How far back of a click the lexer starts at most, in runes: from
/// the start of the text when the click is no further in than this,
/// so the lexer's state is exact for any file of ordinary size.
pub const LEX_LIMIT: usize = 1 << 20;

impl Lang {
    /// The language of the file at `path`, by its extension.
    pub fn of_path(path: &str) -> Option<Lang> {
        let name = path.rsplit('/').next()?;
        let (_, ext) = name.rsplit_once('.')?;
        EXTS.iter().find(|(_, exts)| exts.contains(&ext)).map(|&(l, _)| l)
    }
}

/// What a span of the text is.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    /// An interpreted string, `"..."`, with backslash escapes.
    Str,
    /// A raw string, `` `...` ``: no escapes, newlines and all.
    Raw,
    /// A rune literal, `'...'`, with backslash escapes.
    Rune,
    /// `//` to the end of the line.
    LineComment,
    /// `/* ... */`.
    BlockComment,
}

impl Kind {
    /// A literal between quotes (not a comment).
    pub fn quoted(self) -> bool {
        matches!(self, Kind::Str | Kind::Raw | Kind::Rune)
    }
}

/// A literal or a comment: its opening delimiter at `open`, its
/// contents `body`, and `end` just past its closing delimiter. One left
/// unclosed (a string at the end of its line, a raw string or a block
/// comment at the end of the text) ends where its body does; so does a
/// line comment, the newline not being part of it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Span {
    pub kind: Kind,
    pub open: usize,
    pub body: (usize, usize),
    pub end: usize,
    pub closed: bool,
}

/// The spans of a text from a point on, in order.
pub struct Lexer<'a> {
    lang: Lang,
    chars: std::iter::Peekable<Box<dyn Iterator<Item = char> + 'a>>,
    pos: usize,
}

impl<'a> Lexer<'a> {
    /// A lexer starting at `from`, taken to be in code.
    pub fn new(t: &'a Text, lang: Lang, from: usize) -> Lexer<'a> {
        let from = from.min(t.len());
        let chars: Box<dyn Iterator<Item = char> + 'a> = Box::new(t.chars_from(from));
        Lexer { lang, chars: chars.peekable(), pos: from }
    }

    /// Where the lexer has got to: past the last span it gave.
    pub fn pos(&self) -> usize {
        self.pos
    }

    fn bump(&mut self) -> Option<char> {
        let c = self.chars.next()?;
        self.pos += 1;
        Some(c)
    }

    fn peek(&mut self) -> Option<char> {
        self.chars.peek().copied()
    }

    fn span(&self, kind: Kind, open: usize, body: (usize, usize), closed: bool) -> Span {
        Span { kind, open, body, end: self.pos, closed }
    }

    /// A string or rune whose opening `quote` is at `open`: to the next
    /// unescaped `quote`, or unclosed at the end of the line.
    fn quoted(&mut self, kind: Kind, quote: char, open: usize) -> Span {
        let b0 = self.pos;
        loop {
            match self.peek() {
                None | Some('\n') => return self.span(kind, open, (b0, self.pos), false),
                Some(c) if c == quote => {
                    let b1 = self.pos;
                    self.bump();
                    return self.span(kind, open, (b0, b1), true);
                }
                Some('\\') => {
                    self.bump();
                    if self.peek().is_some_and(|c| c != '\n') {
                        self.bump();
                    }
                }
                Some(_) => {
                    self.bump();
                }
            }
        }
    }

    fn go(&mut self) -> Option<Span> {
        loop {
            let c = self.bump()?;
            let open = self.pos - 1;
            match c {
                '"' => return Some(self.quoted(Kind::Str, '"', open)),
                '\'' => return Some(self.quoted(Kind::Rune, '\'', open)),
                '`' => {
                    let b0 = self.pos;
                    loop {
                        match self.peek() {
                            None => return Some(self.span(Kind::Raw, open, (b0, self.pos), false)),
                            Some('`') => {
                                let b1 = self.pos;
                                self.bump();
                                return Some(self.span(Kind::Raw, open, (b0, b1), true));
                            }
                            Some(_) => {
                                self.bump();
                            }
                        }
                    }
                }
                '/' if self.peek() == Some('/') => {
                    self.bump();
                    let b0 = self.pos;
                    while self.peek().is_some_and(|c| c != '\n') {
                        self.bump();
                    }
                    return Some(self.span(Kind::LineComment, open, (b0, self.pos), false));
                }
                '/' if self.peek() == Some('*') => {
                    self.bump();
                    let b0 = self.pos;
                    loop {
                        match self.bump() {
                            None => return Some(self.span(Kind::BlockComment, open, (b0, self.pos), false)),
                            Some('*') if self.peek() == Some('/') => {
                                let b1 = self.pos - 1;
                                self.bump();
                                return Some(self.span(Kind::BlockComment, open, (b0, b1), true));
                            }
                            Some(_) => {}
                        }
                    }
                }
                _ => {}
            }
        }
    }
}

impl Iterator for Lexer<'_> {
    type Item = Span;

    fn next(&mut self) -> Option<Span> {
        match self.lang {
            Lang::Go => self.go(),
        }
    }
}

/// The spans of a text, lexed as far as they are asked about.
pub struct Spans<'a> {
    lex: Lexer<'a>,
    spans: Vec<Span>,
    done: bool,
}

impl<'a> Spans<'a> {
    pub fn new(t: &'a Text, lang: Lang, from: usize) -> Spans<'a> {
        Spans { lex: Lexer::new(t, lang, from), spans: Vec::new(), done: false }
    }

    /// Lex until every span that starts at or before `p` is known.
    fn lex_past(&mut self, p: usize) {
        while !self.done && self.lex.pos() <= p {
            match self.lex.next() {
                Some(s) => self.spans.push(s),
                None => self.done = true,
            }
        }
    }

    /// The span `p` is in, from its opening delimiter up to (not
    /// including) its end.
    pub fn at(&mut self, p: usize) -> Option<Span> {
        self.lex_past(p);
        let i = self.spans.partition_point(|s| s.open <= p);
        self.spans[..i].last().filter(|s| p < s.end).copied()
    }

    /// The closed span whose closing delimiter `p` is just past.
    pub fn ending_at(&mut self, p: usize) -> Option<Span> {
        self.lex_past(p);
        let i = self.spans.partition_point(|s| s.open < p);
        self.spans[..i].last().filter(|s| s.end == p && s.closed).copied()
    }
}

/// Where to lex from to know what a click at `q` is in: the start of
/// the text, unless `q` is more than `LEX_LIMIT` runes in. Then it is
/// the nearest line before `q`, within the limit, that starts a
/// top-level declaration, or failing one the line the limit falls in.
/// Lexing from the start is exact; a restart is a guess, wrong only
/// when it falls inside a raw string or a comment (a raw string of Go
/// code in a test, say), and then only in a file of over a million
/// runes. The cost is bounded by the limit either way.
pub fn restart(t: &Text, lang: Lang, q: usize) -> usize {
    if q <= LEX_LIMIT {
        return 0;
    }
    let floor = t.line_of(q - LEX_LIMIT);
    let starts: &[&str] = match lang {
        Lang::Go => &["func ", "type ", "var ", "const ", "import ", "package "],
    };
    let mut n = t.line_of(q);
    while n > floor {
        let s = t.line_start(n);
        let head = t.slice(s, s + 8);
        if starts.iter().any(|k| head.starts_with(k)) {
            return s;
        }
        n -= 1;
    }
    t.line_start(floor)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn lex(s: &str) -> Vec<(Kind, String, bool)> {
        let t = Text::new(s);
        Lexer::new(&t, Lang::Go, 0).map(|sp| (sp.kind, t.slice(sp.body.0, sp.body.1), sp.closed)).collect()
    }

    fn k(kind: Kind, body: &str) -> (Kind, String, bool) {
        (kind, body.to_string(), true)
    }

    #[test]
    fn a_go_file_has_its_strings_runes_and_comments() {
        assert_eq!(Lang::of_path("/a/b/q.go"), Some(Lang::Go));
        assert_eq!(Lang::of_path("q.rs"), None);
        assert_eq!(Lang::of_path("/a.go/README"), None);
        assert_eq!(Lang::of_path(""), None);
        assert_eq!(
            lex(r#"var msg = "say \"hi\" now" + `raw "x"` + '\'' + '"' // it's "quoted"
x /* a "b" `c */ y"#),
            vec![
                k(Kind::Str, r#"say \"hi\" now"#),
                k(Kind::Raw, r#"raw "x""#),
                k(Kind::Rune, r"\'"),
                k(Kind::Rune, "\""),
                (Kind::LineComment, r#" it's "quoted""#.to_string(), false),
                k(Kind::BlockComment, r#" a "b" `c "#),
            ]
        );
    }

    #[test]
    fn a_raw_string_spans_lines_and_has_no_escapes() {
        assert_eq!(lex("a := `one\\`\nb := `two\nthree`\n"), vec![k(Kind::Raw, "one\\"), k(Kind::Raw, "two\nthree")]);
        // a backslash before the closing quote escapes it in a string
        assert_eq!(lex(r#""a\\" "b\"c""#), vec![k(Kind::Str, r"a\\"), k(Kind::Str, r#"b\"c"#)]);
        // comment openers inside literals are not comments
        assert_eq!(lex(r#""//x" `/*` '/'"#), vec![k(Kind::Str, "//x"), k(Kind::Raw, "/*"), k(Kind::Rune, "/")]);
        // nor quotes inside comments
        assert_eq!(lex("/* \" */ \"a\" // `\n`b`"), vec![k(Kind::BlockComment, " \" "), k(Kind::Str, "a"), (Kind::LineComment, " `".to_string(), false), k(Kind::Raw, "b")]);
        assert_eq!(lex("/**/ a/b"), vec![k(Kind::BlockComment, "")]);
    }

    #[test]
    fn an_unclosed_literal_ends_at_its_line_or_the_text() {
        let t = Text::new("s := \"abc\nt := 'x\\\n`raw\nmore");
        let spans: Vec<Span> = Lexer::new(&t, Lang::Go, 0).collect();
        assert_eq!(spans[0], Span { kind: Kind::Str, open: 5, body: (6, 9), end: 9, closed: false });
        assert_eq!(spans[1], Span { kind: Kind::Rune, open: 15, body: (16, 18), end: 18, closed: false });
        assert_eq!(spans[2], Span { kind: Kind::Raw, open: 19, body: (20, 28), end: 28, closed: false });
        assert_eq!(lex("/* never"), vec![(Kind::BlockComment, " never".to_string(), false)]);
        assert_eq!(lex("x // y"), vec![(Kind::LineComment, " y".to_string(), false)]);
    }

    #[test]
    fn spans_are_found_by_position() {
        //                    0123456789012345
        let t = Text::new(r#"f("a", "b") // c"#);
        let mut s = Spans::new(&t, Lang::Go, 0);
        assert_eq!(s.at(1), None);
        assert_eq!(s.at(2).map(|s| s.body), Some((3, 4)));
        assert_eq!(s.at(4).map(|s| s.body), Some((3, 4)));
        assert_eq!(s.at(5), None);
        assert_eq!(s.ending_at(5).map(|s| s.body), Some((3, 4)));
        assert_eq!(s.ending_at(6), None);
        assert_eq!(s.at(7).map(|s| s.body), Some((8, 9)));
        assert_eq!(s.at(12).map(|s| s.kind), Some(Kind::LineComment));
        assert_eq!(s.at(16), None);
        assert_eq!(s.ending_at(16), None, "a line comment is not closed");
    }

    #[test]
    fn a_click_far_into_a_big_file_lexes_from_a_declaration() {
        assert_eq!(restart(&Text::new("package p\n"), Lang::Go, 5), 0);
        let filler = "x = 1\n".repeat(LEX_LIMIT / 6 + 10);
        let s = format!("package p\n{filler}func f() {{\n\t\"x\"\n}}\n");
        let t = Text::new(&s);
        let func = s.find("func").unwrap();
        assert_eq!(restart(&t, Lang::Go, t.len() - 3), func);
        // with no declaration near enough, the line the limit falls in
        let t = Text::new(&format!("package p\n{filler}{filler}"));
        let q = t.len() - 2;
        assert_eq!(restart(&t, Lang::Go, q), t.line_start(t.line_of(q - LEX_LIMIT)));
    }
}
