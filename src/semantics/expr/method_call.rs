use analyze::definition::{Method, Param};
use analyze::resolve::scope::Scope;
use parse::tree::{
    ClassType, EnclosingType, Expr, InvocationContext, MethodCall, ParameterizedType,
    PrimitiveType, PrimitiveTypeType, Type, TypeArg, TypeParamExtend,
};
use semantics::{expr, Context};
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::Deref;

#[derive(PartialEq)]
enum ParamScore {
    Matched,
    Inherited,
    Coerced,
    Ok,
    Unmatched,
}

struct MethodWithScore<'a> {
    method: Method<'a>,
    param_scores: Vec<ParamScore>,
    unaccounted_arg_count: i32,
}

pub fn apply<'def, 'def_ref, 'scope_ref>(
    method_call: &'def_ref MethodCall<'def>,
    context: &mut Context<'def, 'def_ref, '_>,
) {
    let invocation_context = InvocationContext { only_static: false };
    let methods = if let Some(prefix) = &method_call.prefix_opt {
        expr::apply(prefix, context);

        if let Some(tpe) = prefix.tpe_opt() {
            tpe.find_methods(method_call.name.fragment, &invocation_context, 0)
        } else {
            vec![]
        }
    } else {
        context
            .scope
            .resolve_methods(method_call.name.fragment, &invocation_context)
    };

    // TODO: Score method based on args
    let mut scores = vec![];

    for method in &methods {
        scores.push(compute(method, method_call));
    }

    //    scores.sort_by(|a, b| )
}

fn realize_type<'def>(tpe: &Type<'def>, map: &HashMap<&str, Type<'def>>) -> Type<'def> {
    // TODO: implement
    tpe.clone()
}

fn compute<'def, 'def_ref>(
    method: &Method<'def>,
    call: &'def_ref MethodCall<'def>,
) -> MethodWithScore<'def> {
    // If type_arg is specified, then there's no inference.
    let method = if let Some(type_args) = &call.type_args_opt {
        realize_method_with_type_args(method, type_args)
    } else {
        method.clone()
    };

    let mut param_scores = vec![];
    for (param, arg) in method.params.iter().zip(call.args.iter()) {
        param_scores.push(compute_param_score(param, arg));
    }

    // TODO: handle var args
    let unaccounted_arg_count = (call.args.len() as i32) - (method.params.len() as i32);

    MethodWithScore {
        method,
        param_scores,
        unaccounted_arg_count,
    }
}

fn realize_method_with_type_args<'def>(
    method: &Method<'def>,
    type_args: &Vec<TypeArg<'def>>,
) -> Method<'def> {
    let mut map: HashMap<&str, Type<'def>> = HashMap::new();
    for (type_param, type_arg) in method.type_params.iter().zip(type_args.iter()) {
        map.insert(&type_param.name, type_arg.to_type());
    }
    let mut params = vec![];
    for param in &method.params {
        params.push(Param {
            tpe: RefCell::new(realize_type(param.tpe.borrow().deref(), &map)),
            name: param.name,
            is_varargs: param.is_varargs,
        })
    }
    Method {
        type_params: method.type_params.clone(),
        params,
        return_type: realize_type(&method.return_type, &map),
        depth: method.depth,
        def: method.def,
    }
}

fn compute_param_score<'def>(param: &Param<'def>, arg: &Expr<'def>) -> ParamScore {
    let arg_tpe = if let Some(arg_tpe) = arg.tpe_opt() {
        arg_tpe
    } else {
        return ParamScore::Matched;
    };
    match param.tpe.borrow().deref() {
        Type::Class(c) => compute_class_param_score(c, &arg_tpe),
        Type::Primitive(p) => compute_prim_param_score(p, &arg_tpe),
        Type::Array(_) => panic!(),
        Type::Parameterized(_) => panic!(),
        Type::UnknownType => ParamScore::Ok,
        Type::Void(_) => panic!(),
        Type::Wildcard(_) => panic!(),
    }
}

fn compute_parameterized_param_score<'def>(
    parameterized: &ParameterizedType<'def>,
    arg: &Type<'def>,
) -> ParamScore {
    ParamScore::Unmatched
}

fn compute_prim_param_score<'def>(prim: &PrimitiveType<'def>, arg: &Type<'def>) -> ParamScore {
    match arg {
        Type::Primitive(p) => {
            if prim.tpe == p.tpe {
                return ParamScore::Matched;
            }
        }
        _ => (),
    }

    ParamScore::Unmatched
}

fn compute_class_param_score<'def>(class: &ClassType<'def>, arg: &Type<'def>) -> ParamScore {
    match arg {
        Type::Class(c) => {
            //            if c.def_opt == class.def_opt && c.def_opt.is_some() {
            //                return ParamScore::Matched;
            //            } else if c.inherits(class) {
            //                return ParamScore::Inherited;
            //            }

        }
        Type::Parameterized(p) => {
            let type_param = unsafe { &*p.def };

            if type_param.extends.borrow().is_empty() {
                return ParamScore::Inherited;
            }

            for extend in type_param.extends.borrow().iter() {
                let result = match extend {
                    TypeParamExtend::Class(c) => compute_class_param_score(class, arg),
                    TypeParamExtend::Parameterized(p) => compute_parameterized_param_score(p, arg),
                };

                if result != ParamScore::Unmatched {
                    return ParamScore::Inherited;
                }
            }
        }
        Type::Primitive(p) => {
            if can_coerce_from_prim_to_class(p, class) {
                return ParamScore::Coerced;
            }
        }
        Type::Array(a) => (),
        Type::UnknownType => return ParamScore::Ok,
        Type::Void(_) => panic!(),
        Type::Wildcard(_) => panic!(),
    };

    ParamScore::Unmatched
}

fn can_coerce_from_prim_to_class<'def>(
    prim: &PrimitiveType<'def>,
    class: &ClassType<'def>,
) -> bool {
    match prim.tpe {
        PrimitiveTypeType::Int => is_valid_primitive_class(class, "Integer"),
        PrimitiveTypeType::Boolean => false,
        PrimitiveTypeType::Byte => false,
        PrimitiveTypeType::Char => false,
        PrimitiveTypeType::Double => false,
        PrimitiveTypeType::Float => false,
        PrimitiveTypeType::Long => false,
        PrimitiveTypeType::Short => false,
    }
}

fn is_valid_primitive_class<'def>(class: &ClassType<'def>, name: &str) -> bool {
    let def = if let Some(def) = class.def_opt {
        unsafe { &*def }
    } else {
        return &class.name == name && is_java_lang(class);
    };

    def.import_path == format!("java.lang.{}", name)
}

fn is_java_lang(class: &ClassType) -> bool {
    let prefix = if let Some(prefix) = &class.prefix_opt {
        match (*prefix).as_ref() {
            EnclosingType::Package(p) => p,
            EnclosingType::Class(_) => return false, // Impossible to be java.lang.Integer
            EnclosingType::Parameterized(_) => return false, // Impossible to be java.lang.Integer
        }
    } else {
        return true; // No prefix. We assume it's good.
    };

    if let Some(prefix_of_prefix) = &prefix.prefix_opt {
        prefix_of_prefix.prefix_opt.is_none()
            && &(*prefix_of_prefix).name == "java"
            && &prefix.name == "lang"
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use analyze::test_common::find_class;
    use parse::tree::{
        ClassBodyItem, CompilationUnitItem, Expr, PrimitiveType, PrimitiveTypeType, Statement,
        Type, TypeParam,
    };
    use std::ops::Deref;
    use {analyze, semantics};

    #[test]
    fn test_choosing_method() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test {
  void method() {
    method(1);
  }
  
  void method(int i) {
  }
}
        "#
        );

        let class = unwrap!(
            CompilationUnitItem::Class,
            &files.first().unwrap().unit.items.get(0).unwrap()
        );
        let method = unwrap!(ClassBodyItem::Method, &class.body.items.get(0).unwrap());
        let method_call = unwrap!(
            Statement::MethodCall,
            &method.block_opt.as_ref().unwrap().stmts.get(0).unwrap()
        );
        println!("{:#?}", method_call);
    }
}
