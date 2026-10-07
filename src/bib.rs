//! Reading BibTeX.

/// One @article{...}, @book{...} and so on.
#[derive(Clone, Debug, PartialEq)]
pub struct Entry {
    /// The entry type in lower case: "article", "book", "inproceedings".
    pub kind: String,
    pub key: String,
    /// Field names in lower case, values with the outer braces or quotes
    /// removed, in the order they appeared.
    pub fields: Vec<(String, String)>,
}

impl Entry {
    pub fn get(&self, name: &str) -> Option<&str> {
        self.fields.iter().find(|(n, _)| n == name).map(|(_, v)| v.as_str())
    }
}

const MONTHS: [(&str, &str); 12] = [
    ("jan", "January"),
    ("feb", "February"),
    ("mar", "March"),
    ("apr", "April"),
    ("may", "May"),
    ("jun", "June"),
    ("jul", "July"),
    ("aug", "August"),
    ("sep", "September"),
    ("oct", "October"),
    ("nov", "November"),
    ("dec", "December"),
];

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
    /// @string abbreviations seen so far, with lower-case names.
    macros: Vec<(String, String)>,
}

impl Parser<'_> {
    fn peek(&self) -> Option<u8> {
        self.src.get(self.pos).copied()
    }

    fn skip_space(&mut self) {
        while self.peek().is_some_and(|c| c.is_ascii_whitespace()) {
            self.pos += 1;
        }
    }

    fn expect(&mut self, c: u8) -> Result<(), String> {
        self.skip_space();
        if self.peek() == Some(c) {
            self.pos += 1;
            Ok(())
        } else {
            Err(format!("expected '{}'", c as char))
        }
    }

    /// An entry type, citation key or field name.
    fn word(&mut self) -> String {
        self.skip_space();
        let start = self.pos;
        while self.peek().is_some_and(|c| !c.is_ascii_whitespace() && !b"{}(),=#\"@".contains(&c)) {
            self.pos += 1;
        }
        String::from_utf8_lossy(&self.src[start..self.pos]).into_owned()
    }

    /// The inside of a {...} value, which can nest.
    fn braced(&mut self) -> Result<String, String> {
        self.expect(b'{')?;
        let start = self.pos;
        let mut depth = 1;
        while let Some(c) = self.peek() {
            self.pos += 1;
            match c {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Ok(String::from_utf8_lossy(&self.src[start..self.pos - 1]).into_owned());
                    }
                }
                _ => {}
            }
        }
        Err("a brace is never closed".into())
    }

    /// The inside of a "..." value. A quote inside braces doesn't end it,
    /// which is how BibTeX lets you write {"} in a quoted title.
    fn quoted(&mut self) -> Result<String, String> {
        self.expect(b'"')?;
        let start = self.pos;
        let mut depth = 0;
        while let Some(c) = self.peek() {
            self.pos += 1;
            match c {
                b'{' => depth += 1,
                b'}' => depth -= 1,
                b'"' if depth == 0 => {
                    return Ok(String::from_utf8_lossy(&self.src[start..self.pos - 1]).into_owned());
                }
                _ => {}
            }
        }
        Err("a quote is never closed".into())
    }

    /// One piece of a value: braces, quotes, a number or an @string name.
    fn part(&mut self) -> Result<String, String> {
        self.skip_space();
        match self.peek() {
            Some(b'{') => self.braced(),
            Some(b'"') => self.quoted(),
            Some(c) if c.is_ascii_digit() => Ok(self.word()),
            _ => {
                let name = self.word().to_lowercase();
                if name.is_empty() {
                    return Err("expected a value in braces, quotes, a number or an @string name".into());
                }
                let defined = self.macros.iter().find(|(n, _)| *n == name).map(|(_, v)| v.as_str());
                let month = MONTHS.iter().find(|(n, _)| *n == name).map(|(_, v)| *v);
                defined
                    .or(month)
                    .map(str::to_string)
                    .ok_or_else(|| format!("{name} isn't defined with @string"))
            }
        }
    }

    /// A whole value, which can join several parts with #.
    fn value(&mut self) -> Result<String, String> {
        let mut out = self.part()?;
        loop {
            self.skip_space();
            if self.peek() != Some(b'#') {
                return Ok(out);
            }
            self.pos += 1;
            out += &self.part()?;
        }
    }

    /// @string{name = value}, which can also use parentheses.
    fn string_def(&mut self) -> Result<(), String> {
        self.skip_space();
        let close = match self.peek() {
            Some(b'{') => b'}',
            Some(b'(') => b')',
            _ => return Err("expected '{' after @string".into()),
        };
        self.pos += 1;
        let name = self.word().to_lowercase();
        self.expect(b'=')?;
        let value = self.value()?;
        self.expect(close)?;
        self.macros.retain(|(n, _)| *n != name);
        self.macros.push((name, value));
        Ok(())
    }

    fn entry(&mut self, kind: String) -> Result<Entry, String> {
        self.expect(b'{')?;
        let key = self.word();
        let mut fields = Vec::new();
        loop {
            self.skip_space();
            match self.peek() {
                Some(b',') => self.pos += 1,
                Some(b'}') => {
                    self.pos += 1;
                    break;
                }
                None => return Err(format!("entry {key} is never closed")),
                _ => {}
            }
            self.skip_space();
            if self.peek() == Some(b'}') {
                continue;
            }
            let name = self.word().to_lowercase();
            if name.is_empty() {
                return Err(format!("expected a field name in {key}"));
            }
            self.expect(b'=')?;
            let value = self.value()?;
            fields.push((name, value));
        }
        Ok(Entry { kind, key, fields })
    }
}

/// Reads every entry in a .bib file. Text outside entries is ignored, the way
/// BibTeX itself ignores it.
pub fn parse(text: &str) -> Result<Vec<Entry>, String> {
    let mut p = Parser {
        src: text.as_bytes(),
        pos: 0,
        macros: Vec::new(),
    };
    let mut out = Vec::new();
    while let Some(at) = text[p.pos..].find('@') {
        p.pos += at + 1;
        let kind = p.word().to_lowercase();
        if kind == "string" {
            p.string_def()?;
        } else {
            out.push(p.entry(kind)?);
        }
    }
    Ok(out)
}
