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

    let chars: Vec<char> = source.chars().collect();
    let mut i = 0;

    while i < chars.len() {
        let c = chars[i];

        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '/' {
            flush_word(&mut tokens, &mut current_word, line, col);
            flush_punct(&mut tokens, &mut current_punct, line, col);
            scan_line_comment(&chars, &mut i, &mut col);
            continue;
        }

        if c == '/' && i + 1 < chars.len() && chars[i + 1] == '*' {
            flush_word(&mut tokens, &mut current_word, line, col);
            flush_punct(&mut tokens, &mut current_punct, line, col);
            scan_block_comment(&chars, &mut i, &mut line, &mut col);
            continue;
        }

        if c == '"' || c == '\'' {
            flush_word(&mut tokens, &mut current_word, line, col);
            flush_punct(&mut tokens, &mut current_punct, line, col);
            scan_string_literal(&chars, c, &mut i, &mut line, &mut col);
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

fn scan_line_comment(chars: &[char], i: &mut usize, col: &mut usize) {
    while *i < chars.len() && chars[*i] != '\n' {
        *i += 1;
        *col += 1;
    }
}

fn scan_block_comment(chars: &[char], i: &mut usize, line: &mut usize, col: &mut usize) {
    *i += 2;
    *col += 2;
    while *i < chars.len() {
        if chars[*i] == '\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
        if chars[*i] == '*' && *i + 1 < chars.len() && chars[*i + 1] == '/' {
            *i += 2;
            *col += 1;
            break;
        }
        *i += 1;
    }
}

fn scan_string_literal(
    chars: &[char],
    quote: char,
    i: &mut usize,
    line: &mut usize,
    col: &mut usize,
) {
    *i += 1;
    *col += 1;
    let mut escape = false;
    while *i < chars.len() {
        let quote_char = chars[*i];
        if quote_char == '\n' {
            *line += 1;
            *col = 1;
        } else {
            *col += 1;
        }
        if escape {
            escape = false;
        } else if quote_char == '\\' {
            escape = true;
        } else if quote_char == quote {
            *i += 1;
            break;
        }
        *i += 1;
    }
}

fn flush_word(tokens: &mut Vec<Token>, word: &mut String, line: usize, column: usize) {
    if !word.is_empty() {
        tokens.push(Token {
            kind: TokenKind::Word(word.clone()),
            line,
            column: column - word.len(),
        });
        word.clear();
    }
}

fn flush_punct(tokens: &mut Vec<Token>, punct: &mut String, line: usize, column: usize) {
    if !punct.is_empty() {
        tokens.push(Token {
            kind: TokenKind::Punct(punct.clone()),
            line,
            column: column - punct.len(),
        });
        punct.clear();
    }
}

#[cfg(test)]
mod characterization_tests;
