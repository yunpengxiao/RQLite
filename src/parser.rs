use nom::bytes::complete::{tag, tag_no_case, take_while1};
use nom::character::complete::{alpha1, multispace0, multispace1};
use nom::character::is_alphanumeric;
use nom::combinator::{map, opt};
use nom::multi::{many0, many1};
use nom::sequence::{delimited, preceded, terminated, tuple};
use nom::IResult;
use nom::branch::alt;

#[derive(Debug)]
pub enum SqlStatement {
    SELECT(SelectStatement),
    CREATE(CreateStatement),
}

#[derive(Debug)]
pub struct SelectStatement {
    pub table: String,
    pub fields: Vec<String>,
    pub condition: String,
}

#[derive(Debug)]
pub struct CreateStatement {
    pub table_name: String,
    pub cols: Vec<String>,
}

fn selection(input: &str) -> IResult<&str, SelectStatement> {
    let (remaining,
        (_, _, fields, _, _, _, table, _, condition),
    ) = tuple((
        tag_no_case("select".as_bytes()),
        multispace1,
        alpha1,
        multispace1,
        tag_no_case("from".as_bytes()),
        multispace1,
        alpha1,
        opt(multispace1),
        opt(where_condition)
    ))(input)?;
    let condition = match condition {
        Some(c) => c,
        None => "".to_string(),
    };
    Ok((
        remaining,
        SelectStatement {
            table: table.to_owned(), 
            fields: Vec::from([fields.to_owned()]),
            condition,
        }
    ))
}

fn creation(input: &str) -> IResult<&str, CreateStatement> {
    let (remaining,
        (_, _, _, _, table_name, _, _, _, cols, _, _)
    ) = tuple((
        tag_no_case("create".as_bytes()),
        multispace1,
        tag_no_case("table".as_bytes()),
        multispace1,
        take_while1(is_sql_identifier),
        multispace0,
        tag("(".as_bytes()),
        multispace0, 
        field_specification_list,
        multispace0,
        tag(")".as_bytes())
    ))(input)?;
    Ok((
        remaining, 
        CreateStatement {
            table_name: table_name.to_string(),
            cols,
        }
    ))
}

pub fn sql_query(input: &str) -> IResult<&str, SqlStatement> {
    alt((
        map(selection, |r| SqlStatement::SELECT(r)),
        map(creation, |r| SqlStatement::CREATE(r)),
    ))(input)
}

pub fn where_condition(input: &str) -> IResult<&str, String> {
    let (remaining, condition) = 
        preceded(tag_no_case("where".as_bytes()), 
                preceded(multispace1, alpha1))(input)?;
    Ok((remaining, condition.to_string()))
}

fn field_specification_list(i: &str) -> IResult<&str, Vec<String>> {
    many1(terminated(field_specification, opt(ws_sep_comma)))(i)
}

fn field_specification(i: &str) -> IResult<&str, String> {
    let (remaining_input, (col, _, _, _)) = tuple((
        alpha1,
        opt(delimited(multispace1, type_identifier, multispace0)),
        many0(column_constraint),
        opt(ws_sep_comma),
    ))(i)?;

    Ok((
        remaining_input,
        col.to_string(),
    ))
}

fn type_identifier(i: &str) -> IResult<&str, String> {
    alt((
        map(tag_no_case("text".as_bytes()), |_| "text".to_string()),
        map(tag_no_case("integer".as_bytes()), |_| "integer".to_string()),
    ))(i)
}

pub fn column_constraint(i: &str) -> IResult<&str, String> {
    alt((
        map(delimited(multispace0, tag_no_case("autoincrement"), multispace0), |_| "".to_string()),
        map(delimited(multispace0, tag_no_case("primary key"), multispace0), |_| "".to_string()),
    ))(i)
}

fn ws_sep_comma(i: &str) -> IResult<&str,&str> {
    delimited(multispace0, tag(","), multispace0)(i)
}

pub fn is_sql_identifier(chr: char) -> bool {
    is_alphanumeric(chr as u8) || chr == '_' || chr == '@'
}