use crate::types::FeatureDecorator;
use rustpython_ast::{ExprKind, KeywordData, Located, StmtKind};
use semver::Version;
use slog_scope::*;
use std::path::Path;

#[derive(Debug, Default)]
pub struct DecoratorArgs {
    pub name: Option<String>,
    pub version: Option<Version>,
    pub old: Option<String>,
    pub new: Option<String>,
    pub author: Option<String>,
    pub description: Option<String>,
}

pub fn as_string_constant(x: &ExprKind) -> Option<String> {
    match x {
        ExprKind::Constant {
            value: rustpython_ast::Constant::Str(value),
            kind: _,
        } => Some(value.clone()),
        _ => None,
    }
}

pub fn as_name(x: &ExprKind) -> Option<String> {
    match x {
        ExprKind::Name { id, ctx: _ } => Some(id.clone()),
        _ => None,
    }
}

pub fn parse_decorator_args(
    args: &[Located<ExprKind>],
    keywords: &[Located<KeywordData>],
) -> Result<DecoratorArgs, &'static str> {
    if !args.is_empty() {
        return Err("@bring.feature(..) does not accept positional argument");
    }
    let mut parsed = DecoratorArgs::default();
    for kwarg in keywords {
        let arg_name = kwarg.node.arg.clone().unwrap();
        let value = &kwarg.node.value.node;
        match arg_name.as_str() {
            "name" => {
                if parsed.name.is_some() {
                    return Err("@bring.feature(..) has two name keyword arguments");
                }
                if let Some(s) = as_string_constant(value) {
                    parsed.name = Some(s);
                } else {
                    return Err("@bring.feature(..)'s name only accepts literal string");
                }
            }
            "version" => {
                if parsed.version.is_some() {
                    return Err("@bring.feature(..) has two version keyword arguments");
                }
                let version_string = if let Some(s) = as_string_constant(value) {
                    s
                } else {
                    return Err("@bring.feature(..)'s version only accepts literal string");
                };
                match Version::parse(&version_string) {
                    Ok(version) => {
                        if version.pre.is_empty() {
                            parsed.version = Some(version)
                        } else {
                            return Err("@brint.feature(..)'s version keyword argument does not support prerelease");
                        }
                    }
                    Err(_e) => {
                        return Err("@bring.feature(..)'s version keyword argument has invalid semver version");
                    }
                }
            }
            "author" => {
                if parsed.author.is_some() {
                    return Err("@bring.feature(..) has two author keyword arguments");
                }
                if let Some(s) = as_string_constant(value) {
                    parsed.author = Some(s);
                } else {
                    return Err("@bring.feature(..)'s author only accepts literal string");
                }
            }
            "description" => {
                if parsed.description.is_some() {
                    return Err("@bring.feature(..) has two description keyword arguments");
                }
                if let Some(s) = as_string_constant(value) {
                    parsed.description = Some(s);
                } else {
                    return Err("@bring.feature(..)'s description only accepts literal string");
                }
            }
            "new" => {
                if parsed.new.is_some() {
                    return Err("@bring.feature(..) has two new keyword arguments");
                }
                if let Some(s) = as_name(value) {
                    parsed.new = Some(s);
                } else {
                    return Err("@bring.feature(..)'s new only accepts literal string");
                }
            }
            "old" => {
                if parsed.old.is_some() {
                    return Err("@bring.feature(..) has two old keyword arguments");
                }
                if let Some(s) = as_name(value) {
                    parsed.old = Some(s);
                } else {
                    return Err("@bring.feature(..)'s old only accepts literal string");
                }
            }
            _ => {
                return Err("@bring.feature(..) has unknown keyword argument");
            }
        }
    }

    // check required arguments are present
    if parsed.version.is_none() {
        return Err("@bring.feature(..) has no version keyword argument");
    }

    if parsed.name.is_none() {
        return Err("@bring.feature(..) has no name keyword argument");
    }

    if parsed.old.is_some() && parsed.new.is_some() {
        return Err("@bring.feature(..) has both old and new keyword arguments");
    }

    Ok(parsed)
}

pub fn process_code(path: &Path, code: &str) -> Option<Vec<FeatureDecorator>> {
    let p = format!("{}", path.display());
    let log = logger().new(slog::o!("path" => p));

    let program = match rustpython_parser::parser::parse_program(&code) {
        Ok(program) => program,
        Err(e) => {
            slog_warn!(log, "Python script has error"; "err"=>format_args!("{:?}", e));
            return None;
        }
    };

    let mut features = Vec::new();

    for stmt in &program {
        let name_and_decorators: Option<(String, &[Located<ExprKind>])> = match &stmt.node {
            StmtKind::FunctionDef {
                name: func_name,
                decorator_list,
                ..
            } => Some((format!("def {}", func_name), decorator_list.as_slice())),
            StmtKind::AsyncFunctionDef {
                name: func_name,
                decorator_list,
                ..
            } => Some((
                format!("async def {}", func_name),
                decorator_list.as_slice(),
            )),
            StmtKind::ClassDef {
                name: cls_name,
                decorator_list,
                ..
            } => Some((format!("class {}", cls_name), decorator_list.as_slice())),
            _ => None,
        };

        if name_and_decorators.is_none() {
            continue;
        }
        let (name, decorator_list) = name_and_decorators.unwrap();

        for expr in decorator_list {
            if let rustpython_ast::ExprKind::Call {
                func,
                args,
                keywords,
            } = &expr.node
            {
                if let rustpython_ast::ExprKind::Attribute {
                    value,
                    attr,
                    ctx: rustpython_ast::ExprContext::Load,
                } = &func.node
                {
                    if let rustpython_ast::ExprKind::Name {
                        id,
                        ctx: rustpython_ast::ExprContext::Load,
                    } = &value.node
                    {
                        if id == "brint" && attr == "feature" {
                            // we found '@brint.feature(..)'
                            match parse_decorator_args(args.as_slice(), keywords.as_slice()) {
                                Ok(args) => {
                                    let (old, new) = match (args.old, args.new) {
                                        (Some(old), None) => (Some(old), name.clone()),
                                        (None, Some(new)) => (Some(name.clone()), new),
                                        (None, None) => (None, name.clone()),
                                        _ => unreachable!(),
                                    };

                                    features.push(FeatureDecorator {
                                        path: path.to_path_buf(),
                                        line: stmt.location.row(),
                                        feature_name: args.name.unwrap(),
                                        version: args.version.unwrap(),
                                        new,
                                        old,
                                        author: args.author,
                                        description: args.description,
                                    });
                                }
                                Err(e) => {
                                    slog_warn!(log, "{}", e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    Some(features)
}
