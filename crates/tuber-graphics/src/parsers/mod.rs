use crate::low_level::model::Model;
use crate::parsers::obj_parser::ObjParseError;

pub mod obj_parser;

#[derive(Clone, Debug)]
pub enum ParseError {
    ObjParseError(ObjParseError),
}

pub trait ModelParser {
    fn parse_model(data: &str) -> Result<Model, ParseError>;
}
