use analyze;
use analyze::definition::{Method, Param};
use analyze::resolve::scope::Scope;
use parse::tree::{
    ClassType, EnclosingType, Expr, InvocationContext, MethodCall, ParameterizedType,
    PrimitiveType, PrimitiveTypeType, Type, TypeArg, TypeParamExtend,
};
use semantics::{expr, Context};
use std::cell::RefCell;
use std::cmp::Ordering;
use std::cmp::Ordering::{Equal, Greater, Less};
use std::collections::HashMap;
use std::ops::Deref;

#[derive(Debug, PartialEq)]
enum ParamScore {
    Matched,
    Inherited,
    Coerced,
    UnknownTypeMatched,
    Unmatched,
}

#[derive(Debug, PartialEq)]
struct MethodWithScore<'a> {
    method: Method<'a>,
    param_scores: Vec<ParamScore>,
    matched_count: usize,
    inherited_count: usize,
    coerced_count: usize,
    unknown_type_matched_count: usize,
    unmatched_count: usize,
    unaccounted_param_count: usize,
    unaccounted_arg_count: usize,
}

impl<'a> MethodWithScore<'a> {
    fn new(
        method: Method<'a>,
        param_scores: Vec<ParamScore>,
        arg_count: usize,
    ) -> MethodWithScore<'a> {
        let mut matched_count = 0;
        let mut inherited_count = 0;
        let mut coerced_count = 0;
        let mut unknown_type_matched_count = 0;
        let mut unmatched_count = 0;
        let unaccounted_param_count =
            (param_scores.len() as i32 - arg_count as i32).max(0) as usize;
        let unaccounted_arg_count = (arg_count as i32 - param_scores.len() as i32).max(0) as usize;

        for param_score in &param_scores {
            match param_score {
                ParamScore::Matched => matched_count += 1,
                ParamScore::Inherited => inherited_count += 1,
                ParamScore::Coerced => coerced_count += 1,
                ParamScore::UnknownTypeMatched => unknown_type_matched_count += 1,
                ParamScore::Unmatched => unmatched_count += 1,
            }
        }

        MethodWithScore {
            method,
            param_scores,
            matched_count,
            inherited_count,
            coerced_count,
            unknown_type_matched_count,
            unmatched_count,
            unaccounted_param_count,
            unaccounted_arg_count,
        }
    }
}

pub fn apply<'def>(method_call: &mut MethodCall<'def>, context: &mut Context<'def, '_, '_>) {
    let invocation_context = InvocationContext { only_static: false };
    let methods = if let Some(prefix) = &mut method_call.prefix_opt {
        expr::apply(prefix, &Type::UnknownType, context);

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

    let mut scores = vec![];

    for method in methods {
        scores.push(compute(method, method_call, context));
    }

    scores.sort_by(|a, b| {
        if a.matched_count == b.matched_count {
            Equal
        } else {
            if b.matched_count > a.matched_count {
                Less
            } else {
                Greater
            }
        }
    });

    if let Some(selected) = scores.pop() {
        for (param, arg) in selected
            .method
            .params
            .iter()
            .zip(method_call.args.iter_mut())
        {
            expr::apply(arg, param.tpe.borrow().deref(), context);
        }
        method_call
            .def_opt
            .replace(Some(infer(selected.method, method_call)));
    } else {
        for arg in &mut method_call.args {
            expr::apply(arg, &Type::UnknownType, context);
        }
    }
}

fn infer<'def>(method: Method<'def>, call: &MethodCall<'def>) -> Method<'def> {
    if let Some(type_args) = &call.type_args_opt {
        return realize_method_with_type_args(method, type_args);
    };

    method
}

fn realize_type<'def>(tpe: &Type<'def>, map: &HashMap<String, Type<'def>>) -> Type<'def> {
    // TODO: implement
    let parameterized = if let Type::Parameterized(p) = tpe {
        p
    } else {
        return tpe.clone();
    };

    if let Some(new_type) = map.get(&parameterized.name) {
        new_type.clone()
    } else {
        tpe.clone()
    }
}

fn compute<'def>(
    method: Method<'def>,
    call: &mut MethodCall<'def>,
    context: &mut Context<'def, '_, '_>,
) -> MethodWithScore<'def> {
    // If type_arg is specified, then there's no inference.
    let method = if let Some(type_args) = &call.type_args_opt {
        realize_method_with_type_args(method, type_args)
    } else {
        method
    };

    let mut param_scores = vec![];
    let mut inferred: HashMap<String, Type<'def>> = HashMap::new();
    for (param, arg) in method.params.iter().zip(call.args.iter_mut()) {
        param_scores.push(compute_param_score(param, arg, &inferred, context));

        if let Type::Parameterized(t) = param.tpe.borrow().deref() {
            if let Some(tpe) = arg.tpe_opt() {
                if should_replace(inferred.get(&t.name)) {
                    inferred.insert(t.name.to_owned(), tpe);
                }
            }
        }
    }

    MethodWithScore::new(
        realize_method_with_type_mapping(method, &inferred),
        param_scores,
        call.args.len(),
    )
}

fn should_replace(current_type_opt: Option<&Type>) -> bool {
    let current_type = if let Some(current_type) = current_type_opt {
        current_type
    } else {
        return true;
    };

    match current_type {
        Type::Parameterized(_) => true,
        Type::Wildcard(_) => true,
        Type::UnknownType => true,
        _ => false,
    }
}

fn realize_method_with_type_args<'def>(
    method: Method<'def>,
    type_args: &Vec<TypeArg<'def>>,
) -> Method<'def> {
    let mut map: HashMap<String, Type<'def>> = HashMap::new();
    for (type_param, type_arg) in method.type_params.iter().zip(type_args.iter()) {
        map.insert(type_param.name.to_owned(), type_arg.to_type());
    }

    realize_method_with_type_mapping(method, &map)
}

fn realize_method_with_type_mapping<'def>(
    method: Method<'def>,
    type_mapping: &HashMap<String, Type<'def>>,
) -> Method<'def> {
    let mut params = vec![];
    for param in method.params {
        params.push(Param {
            tpe: RefCell::new(realize_type(param.tpe.borrow().deref(), &type_mapping)),
            name: param.name,
            is_varargs: param.is_varargs,
        })
    }
    Method {
        type_params: method.type_params.clone(),
        params,
        return_type: realize_type(&method.return_type, &type_mapping),
        depth: method.depth,
        def: method.def,
    }
}

fn compute_param_score<'def>(
    param: &Param<'def>,
    arg: &mut Expr<'def>,
    inferred: &HashMap<String, Type<'def>>,
    context: &mut Context<'def, '_, '_>,
) -> ParamScore {
    if let Type::Parameterized(p) = param.tpe.borrow().deref() {
        if let Some(found) = inferred.get(&p.name) {
            param.tpe.replace(found.clone());
        }
    }

    expr::apply(arg, param.tpe.borrow().deref(), context);
    let arg_tpe = if let Some(arg_tpe) = arg.tpe_opt() {
        arg_tpe
    } else {
        return ParamScore::UnknownTypeMatched;
    };
    match param.tpe.borrow().deref() {
        Type::Class(c) => compute_class_param_score(c, &arg_tpe, context),
        Type::Primitive(p) => compute_prim_param_score(p, &arg_tpe),
        Type::Parameterized(p) => compute_parameterized_param_score(p, &arg_tpe),
        Type::Array(_) => panic!(),
        Type::UnknownType => ParamScore::UnknownTypeMatched,
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

fn compute_class_param_score<'def>(
    class: &ClassType<'def>,
    arg: &Type<'def>,
    context: &Context<'def, '_, '_>,
) -> ParamScore {
    match arg {
        Type::Class(c) => {
            if c.def_opt.is_some() && c.def_opt == class.def_opt {
                return ParamScore::Matched;
                //            } else if c.inherits(class) {
                //                return ParamScore::Inherited;
            }
        }
        Type::Parameterized(p) => {
            let type_param = unsafe { &*p.def };

            if type_param.extends.borrow().is_empty() {
                return ParamScore::Inherited;
            }

            for extend in type_param.extends.borrow().iter() {
                let result = match extend {
                    TypeParamExtend::Class(c) => compute_class_param_score(class, arg, context),
                    TypeParamExtend::Parameterized(p) => compute_parameterized_param_score(p, arg),
                };

                if result != ParamScore::Unmatched {
                    return ParamScore::Inherited;
                }
            }
        }
        Type::Primitive(p) => {
            if can_coerce_from_prim_to_class(p, class, context) {
                return ParamScore::Coerced;
            }
        }
        Type::Array(a) => (),
        Type::UnknownType => return ParamScore::UnknownTypeMatched,
        Type::Void(_) => panic!(),
        Type::Wildcard(_) => panic!(),
    };

    ParamScore::Unmatched
}

fn can_coerce_from_prim_to_class<'def>(
    prim: &PrimitiveType<'def>,
    class: &ClassType<'def>,
    context: &Context<'def, '_, '_>,
) -> bool {
    let coerced = coerce_from_primitive_to_class(prim, context);

    is_valid_primitive_class(class, &coerced.name)
}

pub fn coerce_from_primitive_to_class<'def>(
    prim: &PrimitiveType<'def>,
    context: &Context<'def, '_, '_>,
) -> ClassType<'def> {
    match prim.tpe {
        PrimitiveTypeType::Int => make_java_lang_class_type("Integer", context),
        PrimitiveTypeType::Boolean => make_java_lang_class_type("Boolean", context),
        PrimitiveTypeType::Byte => make_java_lang_class_type("Byte", context),
        PrimitiveTypeType::Char => make_java_lang_class_type("Character", context),
        PrimitiveTypeType::Double => make_java_lang_class_type("Double", context),
        PrimitiveTypeType::Float => make_java_lang_class_type("Float", context),
        PrimitiveTypeType::Long => make_java_lang_class_type("Long", context),
        PrimitiveTypeType::Short => make_java_lang_class_type("Short", context),
    }
}

fn make_java_lang_class_type<'def>(name: &str, context: &Context<'def, '_, '_>) -> ClassType<'def> {
    ClassType {
        prefix_opt: None,
        name: name.to_owned(),
        span_opt: None,
        type_args_opt: None,
        def_opt: context
            .scope
            .root
            .find_package("java")
            .and_then(|p| p.find_package("lang"))
            .and_then(|p| p.find_class(name))
            .map(|c| c as *const analyze::definition::Class<'def>),
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
    use analyze::definition::MethodDef;
    use analyze::test_common::find_class;
    use parse::tree::{
        ClassBodyItem, ClassType, CompilationUnitItem, Expr, PrimitiveType, PrimitiveTypeType,
        Statement, Type, TypeParam,
    };
    use std::ops::Deref;
    use test_common::{span, span2};
    use {analyze, semantics};

    #[test]
    fn test_simple() {
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
        let expr_stmt = unwrap!(
            Statement::Expr,
            &method.block_opt.as_ref().unwrap().stmts.get(0).unwrap()
        );
        let method_call = unwrap!(Expr::MethodCall, &expr_stmt);

        assert_eq!(
            unwrap!(ClassBodyItem::Method, &class.body.items.get(1).unwrap())
                .def_opt
                .borrow()
                .unwrap(),
            method_call.def_opt.borrow().as_ref().unwrap().def
        );
    }

    #[test]
    fn test_simple_member_method() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test {
  void method() {
    Another t;
    t.method(t);
  }
}
        "#,
            r#"
package dev;

class Another {
  void method(Another i) {
  }
  
  void method(int i) {
  }
}
        "#
        );

        let test = unwrap!(
            CompilationUnitItem::Class,
            &files.first().unwrap().unit.items.get(0).unwrap()
        );
        let method = unwrap!(ClassBodyItem::Method, &test.body.items.get(0).unwrap());
        let expr_stmt = unwrap!(
            Statement::Expr,
            &method.block_opt.as_ref().unwrap().stmts.get(1).unwrap()
        );
        let method_call = unwrap!(Expr::MethodCall, &expr_stmt);

        let another = unwrap!(
            CompilationUnitItem::Class,
            &files.get(1).unwrap().unit.items.get(0).unwrap()
        );
        assert_eq!(
            unwrap!(ClassBodyItem::Method, &another.body.items.get(0).unwrap())
                .def_opt
                .borrow()
                .unwrap(),
            method_call.def_opt.borrow().as_ref().unwrap().def
        );
    }

    #[test]
    fn test_infer() {
        let (files, root) = apply_semantics!(
            r#"
package dev;

class Test {
  void method() {
    Test t;
    method(t);
  }
  
  <T> T method(T i) {
  }
}
        "#
        );

        let class = unwrap!(
            CompilationUnitItem::Class,
            &files.first().unwrap().unit.items.get(0).unwrap()
        );
        let method = unwrap!(ClassBodyItem::Method, &class.body.items.get(0).unwrap());
        let expr_stmt = unwrap!(
            Statement::Expr,
            &method.block_opt.as_ref().unwrap().stmts.get(1).unwrap()
        );
        let method_call = unwrap!(Expr::MethodCall, &expr_stmt);

        let def = method_call.def_opt.borrow();
        assert_eq!(
            &Type::Class(ClassType {
                prefix_opt: None,
                name: "Test".to_string(),
                span_opt: Some(span2(5, 5, "Test", files.first().unwrap().deref())),
                type_args_opt: None,
                def_opt: Some(find_class(&root, "dev.Test"))
            }),
            &def.as_ref().unwrap().return_type
        )
    }
}
