use crate::logoi::input::tool::{FunctionCall, FunctionParameter, FunctionType};

#[derive(Debug, PartialEq)]
pub struct FunctionCallRaw<'a> {
    pub name: &'a str,
    pub description: &'a str,
    pub parameters: [&'a str; 100]
}

impl<'a> FunctionCallRaw<'a> {
    pub fn to_fn_call(self) -> Result<FunctionCall, String> {
        let name = self.name.to_string();
        let description = match self.description {
            raw if raw.len() > 0 => Some(raw.to_string()),
            _ => None
        };
        let parameters = parse_raw_params(self.parameters)?;

        return Ok(FunctionCall {
            name,
            description,
            parameters
        })
    }
}


fn parse_raw_params(parameters: [&str; 100]) -> Result<Vec<FunctionParameter>, String> {
    let mut refined_parameters: Vec<FunctionParameter> = vec![];

    for param in parameters {
        let (name, _type) = match param.split_once(':') {
            Some((name, _type)) => (name.trim(), _type.trim()),
            None => continue,
        };

        println!("Name: {name}, _type: {_type}");

        let refined_parameter = FunctionParameter {
            name: name.to_string(),
            _type: parse_raw_type(_type)?,
            description: None
        };

        refined_parameters.push(refined_parameter);
    }

    return Ok(refined_parameters)
}

fn parse_raw_type(_type: &str) -> Result<FunctionType, String> {
    match _type.to_lowercase().as_str() {
        "string" => Ok(FunctionType::String),
        "i64" | "i32" | "i16" | "i8"
        | "f64" | "f32" | "f16" | "f8"
        | "number" => Ok(FunctionType::Number),
        "bool" => Ok(FunctionType::Boolean),
        _ => return Err("Not yet supported".to_string())
    }
}