use nom::bytes::complete::tag_no_case;
use nom::character::complete::{alpha1, multispace1};
use nom::combinator::map;
use nom::sequence::tuple;
use nom::IResult;
use nom::branch::alt;

pub enum SqlStatement {
    SELECT(SelectStatement),
    CREATE(CreateStatement),
}

pub struct SelectStatement {
    pub table: String,
    pub fields: Vec<String>,
    pub condition: String,
}

pub struct CreateStatement {
    pub nothing: String,
}

fn selection(input: &str) -> IResult<&str, SelectStatement> {
    let (remaining,
        (_, _, fields, _, _, _, table, _, _, _, condition),
    ) = tuple((
        tag_no_case("select".as_bytes()),
        multispace1,
        alpha1,
        multispace1,
        tag_no_case("from".as_bytes()),
        multispace1,
        alpha1,
        multispace1,
        tag_no_case("where".as_bytes()),
        multispace1,
        alpha1,
    ))(input)?;
    Ok((
        remaining,
        SelectStatement {
            table: table.to_owned(), 
            fields: Vec::from([fields.to_owned()]),
            condition: condition.to_owned(),
        }
    ))
}

pub fn sql_query(input: &str) -> IResult<&str, SqlStatement> {
    alt((
        map(selection, |r| SqlStatement::SELECT(r)),
    ))(input)
}