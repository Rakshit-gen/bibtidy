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

struct Parser<'a> {
    src: &'a [u8],
    pos: usize,
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

    fn entry(&mut self) -> Result<Entry, String> {
        let kind = self.word().to_lowercase();
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
            self.skip_space();
            let value = self.braced()?;
            fields.push((name, value));
        }
        Ok(Entry { kind, key, fields })
    }
}

/// Reads every entry in a .bib file. Text outside entries is ignored, the way
/// BibTeX itself ignores it.
pub fn parse(text: &str) -> Result<Vec<Entry>, String> {
    let mut p = Parser { src: text.as_bytes(), pos: 0 };
    let mut out = Vec::new();
    while let Some(at) = text[p.pos..].find('@') {
        p.pos += at + 1;
        out.push(p.entry()?);
    }
    Ok(out)
}
