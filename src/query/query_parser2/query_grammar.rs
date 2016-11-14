use combine::*;
use combine::char::*;
use super::user_input_ast::*;
use schema::{Schema, Field};

fn literal<I>(input: I) -> ParseResult<UserInputAST, I> 
    where I: Stream<Item = char> {
    let term_val = || {
        let word = many1(satisfy(|c: char| c.is_alphanumeric()));
        let phrase =
            (char('"'), many1(satisfy(|c| c != '"')), char('"'),)
            .map(|(_, s, _)| s);
        phrase.or(word)
    };
    let field = many1(letter());
    let term_query = (field, char(':'), term_val())
        .map(|(field_name,_, phrase)| {
            UserInputLiteral {
                field_name: Some(field_name),
                phrase: phrase
            }
        });
    let term_default_field = term_val()
        .map(|phrase| {
            UserInputLiteral {
                field_name: None,
                phrase: phrase
            }
        });
    try(term_query).or(term_default_field)
    .map(|query_literal| UserInputAST::from(query_literal)) 
    .parse_stream(input)
} 


fn leaf<I>(input: I) -> ParseResult<UserInputAST, I>  
    where I: Stream<Item = char> {
        (char('-'), parser(literal)).map(|(_, expr)| UserInputAST::Not(box expr))
    .or((char('+'), parser(literal)).map(|(_, expr)| UserInputAST::Must(box expr)))
    .or(parser(literal))
    .parse_stream(input)
}


pub fn parse_to_ast<I>(input: I) -> ParseResult<UserInputAST, I>
    where I: Stream<Item = char>
{   
    sep_by(parser(leaf), spaces())
    .map(|subqueries: Vec<UserInputAST>| {
        if subqueries.len() == 1 {
            subqueries.into_iter().next().unwrap()
        }
        else {
            UserInputAST::Clause(subqueries.into_iter().map(Box::new).collect())
        }
    })
    .parse_stream(input)
}


#[cfg(test)]
mod test {
    
    use combine::*;
    use super::*;
    
    fn test_parse_query_to_ast_helper(query: &str, expected: &str) {
        let query = parse_to_ast(query).unwrap().0;
        let query_str = format!("{:?}", query);
        assert_eq!(query_str, expected);
    }

    #[test]
    pub fn test_parse_query_to_ast() {
        test_parse_query_to_ast_helper("abc:toto", "abc:\"toto\"");
        test_parse_query_to_ast_helper("+abc:toto", "+(abc:\"toto\")");
        test_parse_query_to_ast_helper("+abc:toto -titi", "+(abc:\"toto\") -(\"titi\")");
        test_parse_query_to_ast_helper("-abc:toto", "-(abc:\"toto\")");
        test_parse_query_to_ast_helper("abc:a b", "abc:\"a\" \"b\"");
        test_parse_query_to_ast_helper("abc:\"a b\"", "abc:\"a b\"");
    }
}
