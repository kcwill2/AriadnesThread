#[derive(Debug, Clone, PartialEq)]
pub(crate) struct Token {
    pub(crate) kind: TokenKind,
    pub(crate) line: usize,
    pub(crate) column: usize,
}

#[derive(Debug, Clone, PartialEq)]
pub(crate) enum TokenKind {
    Word(String),
    Punct(String),
}

pub(crate) fn tokenize(source: &str) -> Vec<Token> {
    let mut tokens = Vec::new();
    let mut current_word = String::new();
    let mut current_punct = String::new();
    let mut line = 1;
    let mut col = 1;

    let flush_punct = |tokens: &mut Vec<Token>, cp: &mut String, l, c| {
        if !cp.is_empty() {
            tokens.push(Token {
                kind: TokenKind::Punct(cp.clone()),
                line: l,
                column: c - cp.len(),
            });
            cp.clear();
        }
    };

    let flush_word = |tokens: &mut Vec<Token>, cw: &mut String, l, c| {
        if !cw.is_empty() {
            tokens.push(Token {
                kind: TokenKind::Word(cw.clone()),
                line: l,
                column: c - cw.len(),
            });
            cw.clear();
        }
    };

    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            flush_word(&mut tokens, &mut current_word, line, col);
            flush_punct(&mut tokens, &mut current_punct, line, col);
            while i < chars.len() && chars[i] != '\n' {
                i += 1;
                col += 1;
            }
            continue;
        }

        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            flush_word(&mut tokens, &mut current_word, line, col);
            flush_punct(&mut tokens, &mut current_punct, line, col);
            i += 2;
            col += 2;
            while i < chars.len() {
                if chars[i] == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                if chars[i] == '*' && i + 1 < chars.len() && chars[i + 1] == '/' {
                    i += 2;
                    col += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        if c == '"' || c == '\'' {
            flush_word(&mut tokens, &mut current_word, line, col);
            flush_punct(&mut tokens, &mut current_punct, line, col);
            let quote = c;
            i += 1;
            col += 1;
            let mut escape = false;
            while i < chars.len() {
                let qc = chars[i];
                if qc == '\n' {
                    line += 1;
                    col = 1;
                } else {
                    col += 1;
                }
                if escape {
                    escape = false;
                } else if qc == '\\' {
                    escape = true;
                } else if qc == quote {
                    i += 1;
                    break;
                }
                i += 1;
            }
            continue;
        }

        if c == '\n' {
            flush_word(&mut tokens, &mut current_word, line, col);
            flush_punct(&mut tokens, &mut current_punct, line, col);
            line += 1;
            col = 1;
            i += 1;
            continue;
        }

        if c.is_alphanumeric() || c == '_' {
            flush_punct(&mut tokens, &mut current_punct, line, col);
            current_word.push(c);
        } else if c.is_whitespace() {
            flush_word(&mut tokens, &mut current_word, line, col);
            flush_punct(&mut tokens, &mut current_punct, line, col);
        } else {
            flush_word(&mut tokens, &mut current_word, line, col);
            current_punct.push(c);
            flush_punct(&mut tokens, &mut current_punct, line, col + 1); // single char punct for now
        }
        col += 1;
        i += 1;
    }
    flush_word(&mut tokens, &mut current_word, line, col);
    flush_punct(&mut tokens, &mut current_punct, line, col);
    tokens
}

#[cfg(test)]
mod characterization_tests;
