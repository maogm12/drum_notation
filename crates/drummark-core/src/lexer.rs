use logos::Logos;

#[derive(Logos, Debug, PartialEq, Clone, Copy)]
pub enum Token {
    #[token("\n")]
    Newline,

    #[token(" ")]
    Space,

    #[regex(r"#[^\n]*")]
    Comment,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokenize_empty() {
        let mut lexer = Token::lexer("");
        assert_eq!(lexer.next(), None);
    }

    #[test]
    fn tokenize_newline() {
        let mut lexer = Token::lexer("\n");
        assert_eq!(lexer.next(), Some(Ok(Token::Newline)));
    }

    #[test]
    fn tokenize_comment() {
        let mut lexer = Token::lexer("# this is a comment\n");
        assert_eq!(lexer.next(), Some(Ok(Token::Comment)));
        assert_eq!(lexer.next(), Some(Ok(Token::Newline)));
    }
}
