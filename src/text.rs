//! Turning LaTeX-ish BibTeX text into plain text.

/// (accent command, base letter, accented letter)
#[rustfmt::skip]
const ACCENTS: &[(char, char, char)] = &[
    ('"', 'a', 'ä'), ('"', 'e', 'ë'), ('"', 'i', 'ï'), ('"', 'o', 'ö'), ('"', 'u', 'ü'), ('"', 'y', 'ÿ'),
    ('"', 'A', 'Ä'), ('"', 'E', 'Ë'), ('"', 'I', 'Ï'), ('"', 'O', 'Ö'), ('"', 'U', 'Ü'),
    ('\'', 'a', 'á'), ('\'', 'e', 'é'), ('\'', 'i', 'í'), ('\'', 'o', 'ó'), ('\'', 'u', 'ú'), ('\'', 'y', 'ý'),
    ('\'', 'c', 'ć'), ('\'', 'n', 'ń'), ('\'', 's', 'ś'), ('\'', 'z', 'ź'),
    ('\'', 'A', 'Á'), ('\'', 'E', 'É'), ('\'', 'I', 'Í'), ('\'', 'O', 'Ó'), ('\'', 'U', 'Ú'),
    ('`', 'a', 'à'), ('`', 'e', 'è'), ('`', 'i', 'ì'), ('`', 'o', 'ò'), ('`', 'u', 'ù'),
    ('`', 'A', 'À'), ('`', 'E', 'È'),
    ('^', 'a', 'â'), ('^', 'e', 'ê'), ('^', 'i', 'î'), ('^', 'o', 'ô'), ('^', 'u', 'û'),
    ('~', 'a', 'ã'), ('~', 'n', 'ñ'), ('~', 'o', 'õ'), ('~', 'N', 'Ñ'),
    ('c', 'c', 'ç'), ('c', 'C', 'Ç'),
    ('v', 'c', 'č'), ('v', 's', 'š'), ('v', 'z', 'ž'), ('v', 'r', 'ř'), ('v', 'e', 'ě'),
    ('v', 'C', 'Č'), ('v', 'S', 'Š'), ('v', 'Z', 'Ž'), ('v', 'R', 'Ř'),
    ('H', 'o', 'ő'), ('H', 'u', 'ű'),
    ('r', 'a', 'å'), ('r', 'A', 'Å'),
];

/// Letter commands that stand alone: \ss, \o, \aa and friends.
#[rustfmt::skip]
const LETTERS: &[(&str, &str)] = &[
    ("ss", "ß"), ("o", "ø"), ("O", "Ø"), ("aa", "å"), ("AA", "Å"), ("ae", "æ"), ("AE", "Æ"),
    ("oe", "œ"), ("OE", "Œ"), ("l", "ł"), ("L", "Ł"), ("i", "ı"),
    ("TeX", "TeX"), ("LaTeX", "LaTeX"), ("BibTeX", "BibTeX"),
];

/// Readable text from a BibTeX value: accents become real letters, braces go,
/// the escapes \& \% \$ \_ \# and ~ become what they print as, and line
/// breaks and runs of spaces become single spaces.
pub fn plain(s: &str) -> String {
    latex_to_text(s)
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn latex_to_text(s: &str) -> String {
    let chars: Vec<char> = s.chars().collect();
    let mut out = String::with_capacity(s.len());
    let mut i = 0;
    while i < chars.len() {
        let c = chars[i];
        match c {
            '{' | '}' => i += 1,
            '~' => {
                out.push(' ');
                i += 1;
            }
            '\\' => i = command(&chars, i + 1, &mut out),
            _ => {
                out.push(c);
                i += 1;
            }
        }
    }
    out
}

/// Handles the command after a backslash at `i` and returns where to carry on.
fn command(chars: &[char], mut i: usize, out: &mut String) -> usize {
    let Some(&c) = chars.get(i) else { return i };
    if "&%$_#{}".contains(c) {
        out.push(c);
        return i + 1;
    }
    let accent_cmds = ['"', '\'', '`', '^', '~', 'c', 'v', 'H', 'r', '=', '.', 'u'];
    // A letter command name runs on; a symbol one is one character.
    let name: String = if c.is_ascii_alphabetic() {
        chars[i..]
            .iter()
            .take_while(|c| c.is_ascii_alphabetic())
            .collect()
    } else {
        c.to_string()
    };
    if name.chars().count() == 1 && accent_cmds.contains(&c) {
        i += 1;
        // \"o, \" o, \"{o} and \c{c} all work.
        while chars.get(i) == Some(&' ') && c.is_ascii_alphabetic() {
            i += 1;
        }
        let braced = chars.get(i) == Some(&'{');
        if braced {
            i += 1;
        }
        // \i is the dotless i, used under accents: \'{\i}.
        let mut base = chars.get(i).copied();
        if base == Some('\\') && chars.get(i + 1) == Some(&'i') {
            i += 1;
            base = Some('i');
        }
        if let Some(b) = base {
            i += 1;
            let accented = ACCENTS
                .iter()
                .find(|&&(a, l, _)| a == c && l == b)
                .map(|&(_, _, x)| x);
            out.push(accented.unwrap_or(b));
        }
        if braced && chars.get(i) == Some(&'}') {
            i += 1;
        }
        return i;
    }
    i += name.chars().count();
    if let Some((_, s)) = LETTERS.iter().find(|(n, _)| *n == name) {
        out.push_str(s);
        // The space after \ss is only there to end the name.
        if chars.get(i) == Some(&' ') {
            i += 1;
        }
    }
    // Other commands (\emph, \textit and so on) are dropped; their braced
    // argument is kept as text.
    i
}

/// Lower-case ASCII words with single spaces, for comparing titles and names
/// however they were typed.
pub fn fold(s: &str) -> String {
    let mut out = String::new();
    for c in plain(s).chars() {
        let base = ACCENTS
            .iter()
            .find(|&&(_, _, x)| x == c)
            .map(|&(_, b, _)| b)
            .unwrap_or(c);
        let base = match base {
            'ß' => "ss".to_string(),
            'ø' | 'Ø' => "o".into(),
            'æ' | 'Æ' => "ae".into(),
            'œ' | 'Œ' => "oe".into(),
            'ł' | 'Ł' => "l".into(),
            'ı' => "i".into(),
            'å' | 'Å' => "a".into(),
            other => other.to_string(),
        };
        for b in base.chars() {
            if b.is_alphanumeric() {
                out.extend(b.to_lowercase());
            } else if !out.ends_with(' ') && !out.is_empty() {
                out.push(' ');
            }
        }
    }
    out.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn accents_in_every_spelling() {
        assert_eq!(
            plain(r#"G\"{o}del, G\"odel, {\"o}, \'{e}cole, \c{c}a"#),
            "Gödel, Gödel, ö, école, ça"
        );
        assert_eq!(
            plain(r#"Erd\H{o}s and \v{S}koda, Pe\~na, na\"{\i}ve"#),
            "Erdős and Škoda, Peña, naïve"
        );
    }

    #[test]
    fn letters_escapes_and_commands() {
        assert_eq!(
            plain(r#"Stra\ss e, \o{}re, \AA{}ngstr\"om"#),
            "Straße, øre, Ångström"
        );
        assert_eq!(
            plain(r"R\&D at 50\% in \emph{one}~go"),
            "R&D at 50% in one go"
        );
        assert_eq!(plain("{DNA} {S}tructure"), "DNA Structure");
        assert_eq!(plain("Two\n      lines"), "Two lines");
    }

    #[test]
    fn folding_makes_spellings_match() {
        assert_eq!(
            fold(r#"G\"{o}del's {T}heorem: A~Proof"#),
            "godel s theorem a proof"
        );
        assert_eq!(
            fold("Gödel’s theorem -- a proof"),
            "godel s theorem a proof"
        );
        assert_eq!(fold(r"Stra\ss e"), "strasse");
    }
}
