use std::num::{ParseFloatError, ParseIntError};
use std::str::FromStr;

use crate::low_level::mesh::Mesh;
use crate::low_level::model::Model;
use crate::low_level::primitives::{Index, Vertex};
use crate::parsers::obj_parser::ObjParseError::{
    CoordinateNotFound, NotParsingMesh, UnsupportedKeyword, VertexIndexNotFound,
};
use crate::parsers::{ModelParser, ParseError};

pub struct ObjParser;

#[derive(Clone, Debug)]
pub enum ObjParseError {
    UnsupportedKeyword,
    CoordinateNotFound,
    CoordinateParsingFailed(ParseFloatError),
    VertexIndexNotFound,
    VertexIndexParsingFailed(ParseIntError),
    NotParsingMesh,
}

impl From<ObjParseError> for ParseError {
    fn from(obj_parse_error: ObjParseError) -> Self {
        ParseError::ObjParseError(obj_parse_error)
    }
}

impl ObjParser {
    pub fn parse_line(line: &str, model: &mut Model) -> Result<(), ObjParseError> {
        let mut split_line = line.split_whitespace();
        let keyword = split_line.next();
        if keyword.is_none() {
            return Ok(());
        }

        let keyword = keyword.unwrap();

        match keyword {
            "o" => Self::parse_object(model),
            "v" => {
                model
                    .meshes
                    .last_mut()
                    .ok_or(NotParsingMesh)?
                    .vertices
                    .push(Self::parse_vertex(&mut split_line)?);
                Ok(())
            }
            "f" => {
                model
                    .meshes
                    .last_mut()
                    .ok_or(NotParsingMesh)?
                    .indices
                    .extend_from_slice(&(Self::parse_face(&mut split_line)?));
                Ok(())
            }
            "#" | "s" => Ok(()),
            _ => Err(UnsupportedKeyword),
        }
    }

    pub fn parse_object(model: &mut Model) -> Result<(), ObjParseError> {
        model.meshes.push(Mesh::default());
        Ok(())
    }

    pub fn parse_vertex<'a>(
        split_line: &mut impl Iterator<Item = &'a str>,
    ) -> Result<Vertex, ObjParseError> {
        let x = Self::parse_coordinate(split_line)?;
        let y = Self::parse_coordinate(split_line)?;
        let z = Self::parse_coordinate(split_line)?;

        Ok(Vertex {
            position: [x, y, z],
            color: [1.0, 1.0, 1.0],
            texture_coordinates: [0.0, 0.0],
        })
    }

    pub fn parse_face<'a>(
        split_line: &mut impl Iterator<Item = &'a str>,
    ) -> Result<[Index; 3], ObjParseError> {
        let first = Self::parse_index(split_line)?;
        let second = Self::parse_index(split_line)?;
        let third = Self::parse_index(split_line)?;

        Ok([first, second, third])
    }

    fn parse_coordinate<'a>(
        split_line: &mut impl Iterator<Item = &'a str>,
    ) -> Result<f32, ObjParseError> {
        let coordinate = split_line.next().ok_or(CoordinateNotFound)?;
        f32::from_str(coordinate).map_err(ObjParseError::CoordinateParsingFailed)
    }
    fn parse_index<'a>(
        split_line: &mut impl Iterator<Item = &'a str>,
    ) -> Result<Index, ObjParseError> {
        let coordinate = split_line.next().ok_or(VertexIndexNotFound)?;
        Ok(Index::from_str(coordinate).map_err(ObjParseError::VertexIndexParsingFailed)? - 1)
    }
}

impl ModelParser for ObjParser {
    fn parse_model(data: &str) -> Result<Model, ParseError> {
        let mut model = Model { meshes: vec![] };

        for line in data.lines() {
            Self::parse_line(line, &mut model)?;
        }

        Ok(model)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    pub fn parse_triangle_mesh() -> Result<(), ParseError> {
        let obj_data = " \n
        o triangle \n
        v 0.5 1.0 0.0 \n
        v 0.0 -1.0 0.0 \n
        v 1.0 -1.0 0.0 \n
        f 1 2 3 \n
        \n";

        let result = ObjParser::parse_model(obj_data)?;
        assert_eq!(result.meshes.len(), 1);
        assert_eq!(result.meshes[0].vertices.len(), 3);

        assert_eq!(result.meshes[0].vertices[0].position[0], 0.5);
        assert_eq!(result.meshes[0].vertices[0].position[1], 1.0);
        assert_eq!(result.meshes[0].vertices[0].position[2], 0.0);

        assert_eq!(result.meshes[0].vertices[1].position[0], 0.0);
        assert_eq!(result.meshes[0].vertices[1].position[1], -1.0);
        assert_eq!(result.meshes[0].vertices[1].position[2], 0.0);

        assert_eq!(result.meshes[0].vertices[2].position[0], 1.0);
        assert_eq!(result.meshes[0].vertices[2].position[1], -1.0);
        assert_eq!(result.meshes[0].vertices[2].position[2], 0.0);

        assert_eq!(result.meshes[0].indices.len(), 3);
        assert_eq!(result.meshes[0].indices[0], 0);
        assert_eq!(result.meshes[0].indices[1], 1);
        assert_eq!(result.meshes[0].indices[2], 2);
        Ok(())
    }
}
