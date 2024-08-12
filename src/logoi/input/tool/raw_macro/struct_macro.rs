use crate::logoi::input::tool::{FunctionCall, FunctionParameter, FunctionType};

pub fn struct_to_fn_call(struct_serialised: &str) -> Result<FunctionCall, String> {
    let name = extract_struct_name(struct_serialised)?;
    let parameters = extract_parameters(struct_serialised)?;

    Ok(FunctionCall {
        name,
        description: None,
        parameters,
    })
}

fn extract_struct_name(struct_serialised: &str) -> Result<String, String> {
    let struct_name_start = 0;
    let struct_name_end = struct_serialised.find(" {")
        .ok_or_else(|| "Invalid struct serialization: missing opening brace".to_string())?;

    Ok(to_snake_case(struct_serialised[struct_name_start..struct_name_end].trim()))
}

fn extract_parameters(struct_serialised: &str) -> Result<Vec<FunctionParameter>, String> {
    let params_str = extract_parameters_string(struct_serialised)?;

    params_str.split(',')
        .map(|param| parse_parameter(param.trim()))
        .collect()
}

fn extract_parameters_string(struct_serialised: &str) -> Result<&str, String> {
    let params_start = struct_serialised.find('{')
        .ok_or_else(|| "Invalid struct serialization: missing opening brace".to_string())? + 1;

    let params_str = struct_serialised[params_start..].trim();

    if params_str.is_empty() {
        return Err("Invalid struct serialization: no parameters found".to_string());
    }

    Ok(params_str)
}

fn parse_parameter(param: &str) -> Result<FunctionParameter, String> {
    let param_parts: Vec<&str> = param.split(':').map(|p| p.trim()).collect();
    if param_parts.len() != 2 {
        return Err(format!("Invalid parameter format in struct serialization: {}", param));
    }

    let param_name = param_parts[0].to_string();
    let param_type = map_type(param_parts[1])?;

    Ok(FunctionParameter {
        name: param_name,
        _type: param_type,
        description: None,
    })
}

fn map_type(param_type_str: &str) -> Result<FunctionType, String> {
    if param_type_str.starts_with("Vec < ") && param_type_str.ends_with('>') {
        let inner_type_str = &param_type_str[5..param_type_str.len() - 1].trim();
        let inner_type = map_type(inner_type_str)?;
        return Ok(FunctionType::Array(Box::new(inner_type)));
    }

    match param_type_str {
        "u8" | "u16" | "u32" | "u64" | "u128"
        | "i8" | "i16" | "i32" | "i64" | "i128"
        | "f32" | "f64" => Ok(FunctionType::Number),
        "String" => Ok(FunctionType::String),
        "bool" => Ok(FunctionType::Boolean),
        _ => Err(format!("Unsupported parameter type: {}", param_type_str)),
    }
}


fn to_snake_case(s: &str) -> String {
    let mut snake_case = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i != 0 {
                snake_case.push('_');
            }
            snake_case.push_str(c.to_string().to_lowercase().as_str());
        } else {
            snake_case.push(c.to_lowercase().next().unwrap());
        }
    }
    return snake_case
}
