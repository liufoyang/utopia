use std::string::ToString;

#[derive(Debug)]
#[derive(Clone)]
enum Lexer_State {
    Initial,
    Identifier,

    Plus,   // +
    Minus,  // -
    Star,   // *
    Slash,  // /

    GE,     // >=
    GT,     // >
    EQ,     // ==
    NOTEQ,     // !=
    NOT,    // !
    LE,     // <=
    LT,     // <

    SemiColon, // ;
    Comma,     // ,
    Enter,     // \n

    LeftParen, // (
    RightParen,// )
    LeftBrace, //{
    RightBrace, //}

    Assignment,// =

    IntLiteral, // 22329
    StringLiteral,  //字符串字面量
    PrimaryType, // :bool :i32  :i64  :f32  :f64  :string
    Function,    // f
    Function2,    // fn
    FuncationReturn, // ->
}

impl PartialEq for Lexer_State {

    fn eq(&self, other: &Self) -> bool {
        let a_value = self.clone() as u32;
        let b_value = other.clone() as u32;

        return a_value == b_value;
    }

}

#[derive(Debug)]
#[derive(Clone)]
pub enum Token_Type {

    KeyWord,
    Identifier,

    Plus,   // +
    Minus,  // -
    Star,   // *
    Slash,  // /

    GE,     // >=
    GT,     // >
    EQ,     // ==
    NOTEQ,     // !=
    NOT,    // !
    LE,     // <=
    LT,     // <

    SemiColon, // ;
    Comma,     // ,
    Enter,     // \n
    LeftParen, // (
    RightParen,// )
    LeftBrace, //{
    RightBrace, //}

    Assignment,// =

    IntLiteral,
    StringLiteral,  //字符串字面量
    bool,       //:bool
    i32,        //:i32
    i64,        //:i64
    f32,        //:f32
    f64,        //:f64
    string,     //:string
    PrimaryType, // :bool :i32  :i64  :f32  :f64  :string
    Function,    // fn
    FuncationReturn, // ->
}

impl PartialEq for Token_Type {

    fn eq(&self, other: &Self) -> bool {
        let a_value = self.clone() as u32;
        let b_value = other.clone() as u32;

        return a_value == b_value;
    }

}

#[derive(Debug)]
#[derive(Clone)]
pub struct Token {
    text:String,
    token_type:Token_Type,
    line_num:i32,
    column_num:i32,
}

impl ToString for Token {
    fn to_string(&self) -> String {

        let format_str = format!("{}, line:{}, column:{}", self.text, self.line_num, self.column_num);
        let to_str = String::from(format_str);
        return to_str;
    }
}

impl Token {
    pub fn get_type(&self) -> Token_Type {
        return self.token_type.clone();
    }

    pub fn get_text(&self) -> &str {
        return self.text.as_str();
    }
}

#[derive(Debug)]
pub struct TokenList {
    tokens: Vec<Token>,
    position: usize,
}

impl TokenList {
    pub fn new() -> TokenList {
        let list = TokenList {
            tokens: Vec::new(),
            position: 0,
        };

        return list;
    }

    pub fn addToken(&mut self, token:Token) {
        self.tokens.push(token);
    }

    pub fn next(&mut self) -> Option<Token> {
        if self.position >= self.tokens.len() {
            return None;
        }

        let token = self.tokens.get(self.position).unwrap();
        self.position+=1;
        return Some(token.clone());
    }

    pub fn getPosition(& self) -> usize {
        return self.position.clone();
    }

    pub fn setPosition(&mut self, _position:usize)  {
        self.position = _position;
    }

    pub fn preRead(& self) -> Option<Token> {
        if self.position >= self.tokens.len() {
            return None;
        }

        let token = self.tokens.get(self.position).unwrap();
        return Some(token.clone());
    }

    pub fn len(&self) -> usize {
        return self.tokens.len();
    }


}

struct Lexer_Parser {
    line_num:i32,
    column_num:i32,
    key_words:Vec<String>,
    state:Lexer_State,
    tokens:TokenList,
    token_text:String,
}

impl Lexer_Parser {
    fn parse(&mut self, code:&str) {

        let mut chars = code.chars();
        println!("source chars {:?}", chars);
        let mut char_result = chars.next();
        while char_result.is_some() {
            let ch = char_result.unwrap();
            match self.state {
                Lexer_State::Initial => {
                    self.parseInit(ch);
                }

                Lexer_State::RightParen => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::LeftParen => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::RightBrace => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::LeftBrace => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::EQ => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::NOTEQ => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::GT => {
                    if ch == '=' {
                        self.state = Lexer_State::GE;
                        self.token_text.push(ch);
                    } else {
                        self.endToken();
                        self.parseInit(ch);
                    }
                },
                Lexer_State::LT => {
                    if ch == '=' {
                        self.state = Lexer_State::LE;
                        self.token_text.push(ch);
                    } else {
                        self.endToken();
                        self.parseInit(ch);
                    }
                },

                Lexer_State::LE => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::GE => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::Identifier => {
                    if ch.is_alphabetic()||ch.is_ascii_digit()||ch == '_' {
                        self.token_text.push(ch);
                    } else if ch.is_whitespace(){
                        self.endToken();
                    } else if ch == ','||ch == '+' || ch == '-'||ch == '*'||ch == '/'||ch == '>' || ch == '='||ch == '<'||ch == ';'||ch == ')'||ch == '('||ch == '\n'||ch == ':'||ch.is_whitespace(){
                        self.endToken();
                        self.parseInit(ch);
                    } else{
                        panic!{"not valid Identifier {}", self.token_text};
                    }
                },
                Lexer_State::IntLiteral => {
                    if ch.is_ascii_digit() {
                        self.token_text.push(ch);
                    } else if ch.is_whitespace(){
                        self.endToken();
                    }else if  ch == '+' || ch == '-'||ch == '*'||ch == '/'||ch == '>' || ch == '='||ch == '<'||ch == ';'||ch == ')'||ch == '('||ch == '\n' ||ch.is_whitespace(){
                        self.endToken();
                        self.parseInit(ch);
                    } else{
                        panic!{"not valid IntLiteral {}", self.token_text};
                    }
                },
                Lexer_State::Assignment => {
                    if ch == '=' {
                        self.state = Lexer_State::EQ;
                        self.token_text.push(ch);
                    } else {
                        self.endToken();
                        self.parseInit(ch);
                    }
                },
                Lexer_State::NOT => {
                    if ch == '=' {
                        self.state = Lexer_State::NOTEQ;
                        self.token_text.push(ch);
                    } else {
                        self.endToken();
                        self.parseInit(ch);
                    }
                },
                Lexer_State::Plus => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::Minus => {
                    if ch=='>' {
                        self.state = Lexer_State::FuncationReturn;
                        self.token_text.push(ch);
                    } else {
                        self.endToken();
                        self.parseInit(ch);
                    }

                },
                Lexer_State::Star => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::Slash => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::SemiColon => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::Comma => {
                    self.endToken();
                    self.parseInit(ch);
                },
                Lexer_State::Enter => {
                    self.endToken();
                    self.line_num +=1;
                    self.column_num = 0;
                    self.parseInit(ch);
                },
                Lexer_State::StringLiteral => {
                    if ch == '"' {
                        self.token_text.push(ch);
                        self.endToken();
                        self.parseInit(ch);
                    } else {
                        self.parseInit(ch);
                    }
                },
                Lexer_State::PrimaryType => {
                    if ch.is_alphabetic()||ch.is_ascii_digit() {
                        self.token_text.push(ch);
                        self.state == Lexer_State::PrimaryType;
                    } else if ch =='=' || ch==';' || ch==')' || ch=='\n'|| ch=='{'|| ch==','||ch.is_whitespace(){
                        self.endToken();
                        self.parseInit(ch);
                    } else {
                        panic!("not value PrimaryType token {}{}", self.token_text, ch);
                    }
                },

                Lexer_State::Function => {
                    println!("fn state {}", ch);
                    if ch == 'n' {
                        self.state == Lexer_State::Function2;
                        self.token_text.push(ch);

                    } else if ch.is_alphabetic()||ch.is_ascii_digit()||ch == '_' {
                        self.state == Lexer_State::Identifier;
                        self.token_text.push(ch);
                    } else if ch == '+' || ch == '-'||ch == '*'||ch == '/'||ch == '>' || ch == '='||ch == '<'||ch == ';'||ch == ')'||ch == '('||ch == '\n'||ch.is_whitespace(){
                        self.state == Lexer_State::Identifier;
                        self.endToken();
                        self.parseInit(ch);
                    } else{
                        panic!{"not valid Identifier {}", self.token_text};
                    }
                },
                Lexer_State::Function2 => {
                    if ch.is_whitespace(){
                        self.endToken();
                        self.parseInit(ch);
                    } else {
                        panic!{"not valid function definer {}", self.token_text};
                    }
                },
                Lexer_State::FuncationReturn => {
                    self.endToken();
                    self.parseInit(ch);
                },
            }

            char_result = chars.next();
            self.column_num +=1;
        }

        if self.token_text.len() > 0 {
            match self.state {
                Lexer_State::StringLiteral => {
                   panic!{"not end StringLiteral {}", self.token_text};
                }
                _ => {
                    self.endToken();
                }
            }
        }

    }

    fn parseInit(&mut self, ch:char) {
        self.token_text = String::new();
        //println!("init char {}", ch);
        if ch == 'f' {
            self.state = Lexer_State::Function;
            self.token_text.push(ch);

            //println!("f char found {:?} {:?}", self.state, self.token_text);
            return;
        }
        if ch.is_alphabetic() {
            self.state = Lexer_State::Identifier;
            self.token_text.push(ch);
            return;
        }

        if ch.is_ascii_digit() {
            self.state = Lexer_State::IntLiteral;
            self.token_text.push(ch);
            return;
        }

        if ch == '<' {
            self.state = Lexer_State::LT;
            self.token_text.push(ch);
            return;
        }

        if ch == '>' {
            self.state = Lexer_State::GT;
            self.token_text.push(ch);
            return;
        }

        if ch == '=' {
            self.state = Lexer_State::Assignment;
            self.token_text.push(ch);
            return;
        }

        if ch == '(' {
            self.state = Lexer_State::LeftParen;
            self.token_text.push(ch);
            return;
        }

        if ch == ')' {
            self.state = Lexer_State::RightParen;
            self.token_text.push(ch);
            return;
        }

        if ch == '{' {
            self.state = Lexer_State::LeftBrace;
            self.token_text.push(ch);
            return;
        }

        if ch == '}' {
            self.state = Lexer_State::RightBrace;
            self.token_text.push(ch);
            return;
        }

        if ch == '+' {
            self.state = Lexer_State::Plus;
            self.token_text.push(ch);
            return;
        }

        if ch == '-' {
            self.state = Lexer_State::Minus;
            self.token_text.push(ch);
            return;
        }

        if ch == '*' {
            self.state = Lexer_State::Star;
            self.token_text.push(ch);
            return;
        }
        if ch == '/' {
            self.state = Lexer_State::Slash;
            self.token_text.push(ch);
            return;
        }
        if ch == ',' {
            self.state = Lexer_State::Comma;
            self.token_text.push(ch);
            return;
        }
        if ch == ';' {
            self.state = Lexer_State::SemiColon;
            self.token_text.push(ch);
            return;
        }
        if ch == '!' {
            self.state = Lexer_State::NOT;
            self.token_text.push(ch);
            return;
        }

        // not need Enter token
        if ch == '\n' {
            self.state = Lexer_State::Initial;
            //println!("enter char {}", ch);
            //self.token_text.push(ch);
            return;
        }
        if ch == ':' {
            self.state = Lexer_State::PrimaryType;
            self.token_text.push(ch);
            return;
        }

        if ch.is_whitespace(){
            self.state = Lexer_State::Initial;
            return;
        }
        panic!("wrong token begin chart for{}char", ch.is_whitespace());
    }

    fn endToken(&mut self) {

        let token_type = self.state2TokenType();
        let text = self.token_text.clone();

        let token = Token {
            text:text,
            token_type:token_type,
            line_num:self.line_num.clone(),
            column_num:self.column_num.clone(),
        };

        self.tokens.addToken(token);

        //println!("new token {}", self.column_num);

        self.state = Lexer_State::Initial;
        self.token_text = String::new();
    }

    fn state2TokenType(&self) -> Token_Type{
        match self.state {
            Lexer_State::RightParen => Token_Type::RightParen,
            Lexer_State::LeftParen => Token_Type::LeftParen,
            Lexer_State::LeftBrace => Token_Type::LeftBrace,
            Lexer_State::RightBrace => Token_Type::RightBrace,
            Lexer_State::EQ => Token_Type::EQ,
            Lexer_State::NOT => Token_Type::NOT,
            Lexer_State::NOTEQ => Token_Type::NOTEQ,
            Lexer_State::GT => Token_Type::GT,
            Lexer_State::LT => Token_Type::LT,
            Lexer_State::LE => Token_Type::LE,
            Lexer_State::GE => Token_Type::GE,
            Lexer_State::Identifier => {
                if self.key_words.contains(&self.token_text) {
                    if self.token_text.as_str() == "i32" {
                        return Token_Type::i32;
                    }else if self.token_text.as_str() == "i64" {
                        return Token_Type::i64;
                    }else if self.token_text.as_str() == "f32" {
                        return Token_Type::f32;
                    }else if self.token_text.as_str() == "f64" {
                        return Token_Type::f64;
                    }else if self.token_text.as_str() == "bool" {
                        return Token_Type::bool;
                    }else if self.token_text.as_str() == "string" {
                        return Token_Type::string;
                    }

                    return Token_Type::KeyWord;
                } else {
                    return Token_Type::Identifier;
                }
            },
            Lexer_State::IntLiteral => Token_Type::IntLiteral,
            Lexer_State::Assignment => Token_Type::Assignment,
            Lexer_State::Plus => Token_Type::Plus,
            Lexer_State::Minus => Token_Type::Minus,
            Lexer_State::Star => Token_Type::Star,
            Lexer_State::Slash => Token_Type::Slash,
            Lexer_State::SemiColon => Token_Type::SemiColon,
            Lexer_State::Comma => Token_Type::Comma,
            Lexer_State::Enter => Token_Type::Enter,
            Lexer_State::StringLiteral => Token_Type::StringLiteral,

            Lexer_State::PrimaryType => {
                let tmp_text = self.token_text.replace(":","");
                if tmp_text.as_str() == "i32" {
                    return Token_Type::i32;
                }else if tmp_text.as_str() == "i64" {
                    return Token_Type::i64;
                }else if tmp_text.as_str() == "f32" {
                    return Token_Type::f32;
                }else if tmp_text.as_str() == "f64" {
                    return Token_Type::f64;
                }else if tmp_text.as_str() == "bool" {
                    return Token_Type::bool;
                }else if tmp_text.as_str() == "string" {
                    return Token_Type::string;
                } else {
                    panic!("not value PrimaryType token {}", self.token_text);
                    return Token_Type::string;
                }
            },
            Lexer_State::Function => Token_Type::Function,
            Lexer_State::Function2 => Token_Type::Function,
            Lexer_State::FuncationReturn => Token_Type::FuncationReturn,
            Lexer_State::Initial => {
                panic!("Initial can not have end token type")
            }
        }
    }



}

pub fn lexerParse(code:&str) -> TokenList {
    let keyword_vec = vec!["let".to_string(), "if".to_string(), "else".to_string(), "for".to_string(), "while".to_string(), "break".to_string(), "continue".to_string()
    ,"i32".to_string(),"i64".to_string(),"bool".to_string(), "f32".to_string(), "f64".to_string(), "string".to_string(), "return".to_string()];
    let mut parser = Lexer_Parser{
        line_num:1,
        column_num:1,
        state:Lexer_State::Initial,
        tokens:TokenList::new(),
        token_text:String::new(),
        key_words:keyword_vec,
    };

    parser.parse(code);

    println!("the tokens {:?}", parser.tokens);
    return parser.tokens;
}


#[cfg(test)]
mod tests {
    use super::TokenList;

    #[test]
    fn test_assign_express() {
        let code = String::from("let a:i64 = 100; ");

        let tokens = super::lexerParse(code.as_str());

        assert_eq!(5, tokens.len());
    }

    #[test]
    fn test_cal_express() {
        let code = String::from("let a:i32 = 100; if a>0 a=(a-1)*15; if a == 2 a=a+2; if a != 4 a=a+4;");

        let tokens = super::lexerParse(code.as_str());

        assert_eq!(39, tokens.len());
    }

}
