use analyze;
use analyze::definition::{Field, FieldDef, FieldGroup};
use std::borrow::Borrow;
use std::cell::{Cell, Ref, RefCell};
use std::collections::HashMap;
use std::ops::Deref;
use tokenize::span::Span;

#[derive(Debug, PartialEq, Clone)]
pub struct Comment<'a> {
    pub content: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct CompilationUnit<'a> {
    pub package_opt: Option<Package<'a>>,
    pub imports: Vec<Import<'a>>,
    pub items: Vec<CompilationUnitItem<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum CompilationUnitItem<'a> {
    Class(Class<'a>),
    Interface(Interface<'a>),
    Annotation(Annotation<'a>),
    Enum(Enum<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Import<'a> {
    pub prefix_opt: Option<Box<ImportPrefix<'a>>>,
    pub is_static: bool,
    pub is_wildcard: bool,
    pub name: Span<'a>,
    pub def_opt: RefCell<Option<ImportDef<'a>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ImportPrefix<'a> {
    pub prefix_opt: Option<Box<ImportPrefix<'a>>>,
    pub name: Span<'a>,
    pub def_opt: RefCell<Option<ImportPrefixDef<'a>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ImportDef<'a> {
    Package(*const analyze::definition::Package<'a>),
    Class(*const analyze::definition::Class<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ImportPrefixDef<'a> {
    Package(*const analyze::definition::Package<'a>),
    Class(*const analyze::definition::Class<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Package<'a> {
    pub prefix_opt: Option<Box<Package<'a>>>,
    pub annotateds: Vec<Annotated<'a>>,
    pub name: Span<'a>,
    pub def_opt: Option<*const analyze::definition::Package<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Annotated<'a> {
    Normal(NormalAnnotated<'a>),
    Marker(MarkerAnnotated<'a>),
    Single(SingleAnnotated<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct NormalAnnotated<'a> {
    pub class: ClassType<'a>,
    pub params: Vec<AnnotatedParam<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MarkerAnnotated<'a> {
    pub class: ClassType<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct SingleAnnotated<'a> {
    pub class: ClassType<'a>,
    pub value: AnnotatedValue<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AnnotatedParam<'a> {
    pub name: Span<'a>,
    pub value: AnnotatedValue<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AnnotatedValue<'a> {
    Expr(Expr<'a>),
    Annotated(Box<Annotated<'a>>),
    Array(AnnotatedValueArray<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct AnnotatedValueArray<'a> {
    pub items: Vec<AnnotatedValue<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Class<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub name: Span<'a>,
    pub type_params: Vec<TypeParam<'a>>,
    pub extend_opt: Option<ClassType<'a>>,
    pub implements: Vec<ClassType<'a>>,
    pub body: ClassBody<'a>,
    pub def_opt: RefCell<Option<*const analyze::definition::Class<'a>>>,
    pub id: String,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Enum<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub name: Span<'a>,
    pub implements: Vec<ClassType<'a>>,
    pub constants: Vec<EnumConstant<'a>>,
    pub body_opt: Option<ClassBody<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct EnumConstant<'a> {
    pub annotateds: Vec<Annotated<'a>>,
    pub name: Span<'a>,
    pub args_opt: Option<Vec<Expr<'a>>>,
    pub body_opt: Option<ClassBody<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Interface<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub name: Span<'a>,
    pub type_params: Vec<TypeParam<'a>>,
    pub extends: Vec<ClassType<'a>>,
    pub body: ClassBody<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Annotation<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub name: Span<'a>,
    pub body: AnnotationBody<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct AnnotationBody<'a> {
    pub items: Vec<AnnotationBodyItem<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum AnnotationBodyItem<'a> {
    Param(AnnotationParam<'a>),
    FieldDeclarators(FieldDeclarators<'a>),
    Class(Class<'a>),
    Interface(Interface<'a>),
    Enum(Enum<'a>),
    Annotation(Annotation<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct AnnotationParam<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub tpe: Type<'a>,
    pub name: Span<'a>,
    pub default_opt: Option<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassBody<'a> {
    pub items: Vec<ClassBodyItem<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ClassBodyItem<'a> {
    Method(Method<'a>),
    FieldDeclarators(FieldDeclarators<'a>),
    Class(Class<'a>),
    Interface(Interface<'a>),
    Enum(Enum<'a>),
    Annotation(Annotation<'a>),
    StaticInitializer(Block<'a>),
    Constructor(Constructor<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct FieldDeclarators<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub declarators: Vec<VariableDeclarator<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Modifier<'a> {
    Annotated(Annotated<'a>),
    Keyword(Keyword<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Block<'a> {
    pub stmts: Vec<Statement<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InvocationContext {
    pub only_static: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Type<'a> {
    Class(ClassType<'a>),
    Primitive(PrimitiveType<'a>),
    Array(ArrayType<'a>),
    Parameterized(ParameterizedType<'a>),
    Void(Void<'a>),
    Wildcard(WildcardType<'a>),
    UnknownType,
}

impl<'a> Type<'a> {
    pub fn span_opt(&self) -> Option<&Span<'a>> {
        match self {
            Type::Array(arr) => None,
            Type::Class(class) => class.span_opt.as_ref(),
            Type::Parameterized(parameterized) => parameterized.span_opt.as_ref(),
            Type::Wildcard(w) => w.span_opt.as_ref(),
            Type::Void(v) => v.span_opt.as_ref(),
            Type::Primitive(p) => p.span_opt.as_ref(),
            Type::UnknownType => panic!(),
        }
    }

    pub fn to_type_arg(self) -> TypeArg<'a> {
        match self {
            Type::Array(arr) => TypeArg::Array(arr),
            Type::Class(class) => TypeArg::Class(class),
            Type::Parameterized(parameterized) => TypeArg::Parameterized(parameterized),
            Type::Wildcard(w) => TypeArg::Wildcard(w),
            Type::Void(_) => panic!(),
            Type::Primitive(p) => TypeArg::Primitive(p),
            Type::UnknownType => panic!(),
        }
    }

    pub fn to_enclosing_type(self) -> EnclosingType<'a> {
        match self {
            Type::Class(class) => EnclosingType::Class(class),
            Type::Parameterized(parameterized) => EnclosingType::Parameterized(parameterized),
            Type::Wildcard(w) => panic!(),
            Type::Array(arr) => panic!(),
            Type::Void(_) => panic!(),
            Type::Primitive(_) => panic!(),
            Type::UnknownType => panic!(),
        }
    }

    pub fn to_reference_type(self) -> ReferenceType<'a> {
        match self {
            Type::Array(arr) => ReferenceType::Array(arr),
            Type::Class(class) => ReferenceType::Class(class),
            Type::Parameterized(parameterized) => ReferenceType::Parameterized(parameterized),
            Type::Wildcard(w) => panic!(),
            Type::Void(_) => panic!(),
            Type::Primitive(_) => panic!(),
            Type::UnknownType => panic!(),
        }
    }

    pub fn find_field(&self, name: &str, context: &InvocationContext) -> Option<Field<'a>> {
        match self {
            Type::Class(c) => c.find_field(name, context),
            Type::Parameterized(p) => p.find_field(name, context),
            Type::Array(a) => a.find_field(name, context),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Void<'a> {
    pub span_opt: Option<Span<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum ReferenceType<'a> {
    Class(ClassType<'a>),
    Array(ArrayType<'a>),
    Parameterized(ParameterizedType<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct PackagePrefix<'a> {
    pub prefix_opt: Option<Box<EnclosingType<'a>>>,
    pub name: String,
    pub span_opt: Option<Span<'a>>,
    pub def: *const analyze::definition::Package<'a>,
}

impl<'a> PackagePrefix<'a> {
    pub fn set_span_opt(&mut self, span_opt: Option<&Span<'a>>) {
        self.span_opt = span_opt.map(|s| s.clone());
    }
    pub fn find(&self, name: &str) -> Option<EnclosingType<'a>> {
        let def = unsafe { &(*self.def) };
        def.find(name).map(|e| e.to_type())
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum StaticType<'a> {
    Class(ClassType<'a>),
    Parameterized(ParameterizedType<'a>),
}

impl<'a> StaticType<'a> {
    pub fn find_field(&self, name: &str, context: &InvocationContext) -> Option<Field<'a>> {
        match self {
            StaticType::Class(c) => c.find_field(name, context),
            StaticType::Parameterized(p) => p.find_field(name, context),
        }
    }
    pub fn find_inner_class(&self, name: &str) -> Option<ClassType<'a>> {
        match self {
            StaticType::Class(c) => c.find_inner_class(name),
            StaticType::Parameterized(p) => p.find_inner_class(name),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum EnclosingType<'a> {
    Package(PackagePrefix<'a>),
    Class(ClassType<'a>),
    Parameterized(ParameterizedType<'a>),
}

impl<'a> EnclosingType<'a> {
    pub fn set_span_opt(&mut self, span_opt: Option<&Span<'a>>) {
        match self {
            EnclosingType::Class(c) => c.set_span_opt(span_opt),
            EnclosingType::Parameterized(p) => p.set_span_opt(span_opt),
            EnclosingType::Package(p) => p.set_span_opt(span_opt),
        };
    }

    pub fn get_name(&self) -> &str {
        match self {
            EnclosingType::Package(package) => &package.name,
            EnclosingType::Class(class) => &class.name,
            EnclosingType::Parameterized(p) => &p.name,
        }
    }
    pub fn find(&self, name: &str) -> Option<EnclosingType<'a>> {
        match self {
            EnclosingType::Package(package) => package.find(name),
            EnclosingType::Class(class) => class
                .find_inner_class(name)
                .map(|c| EnclosingType::Class(c)),
            EnclosingType::Parameterized(parameterized) => parameterized
                .find_inner_class(name)
                .map(|c| EnclosingType::Class(c)),
        }
    }
    pub fn get_prefix_opt(&self) -> Option<&Option<Box<EnclosingType<'a>>>> {
        match self {
            EnclosingType::Package(package) => Some(&package.prefix_opt),
            EnclosingType::Class(class) => Some(&class.prefix_opt),
            EnclosingType::Parameterized(_) => None,
        }
    }
    pub fn set_prefix_opt(&self, prefix_opt: Option<EnclosingType<'a>>) -> EnclosingType<'a> {
        match self {
            EnclosingType::Package(package) => EnclosingType::Package(PackagePrefix {
                prefix_opt: prefix_opt.map(Box::new),
                name: package.name.to_owned(),
                span_opt: package.span_opt,
                def: package.def,
            }),
            EnclosingType::Class(class) => EnclosingType::Class(ClassType {
                prefix_opt: prefix_opt.map(Box::new),
                name: class.name.to_owned(),
                span_opt: class.span_opt,
                type_args_opt: class.type_args_opt.clone(),
                def_opt: class.def_opt.clone(),
            }),
            EnclosingType::Parameterized(_) => panic!(),
        }
    }

    pub fn to_type(self) -> Type<'a> {
        match self {
            EnclosingType::Class(class) => Type::Class(class),
            EnclosingType::Parameterized(p) => Type::Parameterized(p),
            EnclosingType::Package(package) => panic!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum TypeArg<'a> {
    Class(ClassType<'a>),
    Parameterized(ParameterizedType<'a>),
    Array(ArrayType<'a>),
    Wildcard(WildcardType<'a>),
    Primitive(PrimitiveType<'a>),
}

impl<'a> TypeArg<'a> {
    pub fn to_type(&self) -> Type<'a> {
        match self {
            TypeArg::Parameterized(p) => Type::Parameterized(p.clone()),
            TypeArg::Class(c) => Type::Class(c.clone()),
            TypeArg::Array(a) => Type::Array(a.clone()),
            TypeArg::Wildcard(w) => Type::Wildcard(w.clone()),
            TypeArg::Primitive(p) => Type::Primitive(p.clone()),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct WildcardType<'a> {
    pub span_opt: Option<Span<'a>>,
    pub extends: Vec<ReferenceType<'a>>,
    pub super_opt: Option<Box<ReferenceType<'a>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrimitiveType<'a> {
    pub span_opt: Option<Span<'a>>,
    pub tpe: PrimitiveTypeType,
}

#[derive(Debug, PartialEq, Clone)]
pub enum PrimitiveTypeType {
    Boolean,
    Byte,
    Char,
    Double,
    Float,
    Int,
    Long,
    Short,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassType<'a> {
    pub prefix_opt: Option<Box<EnclosingType<'a>>>,
    pub name: String,
    pub span_opt: Option<Span<'a>>,
    pub type_args_opt: Option<Vec<TypeArg<'a>>>,
    pub def_opt: Option<*const analyze::definition::Class<'a>>,
}

impl<'a> ClassType<'a> {
    pub fn set_span_opt(&mut self, span_opt: Option<&Span<'a>>) {
        self.span_opt = span_opt.map(|s| s.clone());
    }

    pub fn get_extend_opt(&self) -> Option<ClassType<'a>> {
        let extend_class_opt = if let Some(def) = self.def_opt {
            let def = unsafe { &(*def) };
            def.extend_opt.borrow()
        } else {
            return None;
        };

        let extend_class = if let Some(extend_class) = extend_class_opt.as_ref() {
            extend_class
        } else {
            return None;
        };

        Some(extend_class.clone())
        //        Some(extend_class.substitute_type_args_from(self))
    }

    // Example:
    // class Current<T> {}
    // class Subclass<A> extends Current<A> {}
    //
    // We get Current<T> where the value of T is assigned with A.
    //    pub fn substitute_type_args_from(&self, subclass: &ClassType<'a>) -> ClassType<'a> {}

    pub fn find_inner_class(&self, name: &str) -> Option<ClassType<'a>> {
        let class = if let Some(class) = self.def_opt {
            unsafe { &(*class) }
        } else {
            return None;
        };

        match class.find(name) {
            Some(found) => {
                // TODO: transfer type args
                return Some(found.to_type());
            }
            None => {
                match class.extend_opt.borrow().as_ref() {
                    Some(extend) => {
                        if let Some(found) = extend.find_inner_class(name) {
                            // TODO: transfer type args
                            return Some(found);
                        }
                    }
                    None => (),
                }
            }
        };

        None
    }

    pub fn find_field(&self, name: &str, context: &InvocationContext) -> Option<Field<'a>> {
        let def = if let Some(def) = self.def_opt {
            unsafe { &*def }
        } else {
            return None;
        };

        for group in &def.field_groups {
            if context.only_static
                && !group
                    .modifiers
                    .contains(&analyze::definition::Modifier::Static)
            {
                continue;
            }

            for item in &group.items {
                if &item.name == name {
                    return Some(Field {
                        tpe: self.realize(item.tpe.borrow().deref()),
                        def: item,
                    });
                }
            }
        }

        if let Some(extend) = def.extend_opt.borrow().as_ref() {
            if let Some(field) = extend.find_field(name, context) {
                return Some(field);
            }
        }

        None
    }

    fn realize_enclosing(&self, tpe: &EnclosingType<'a>) -> EnclosingType<'a> {
        match tpe {
            EnclosingType::Package(_) => tpe.clone(),
            EnclosingType::Class(c) => EnclosingType::Class(self.realize_class(c)),
            EnclosingType::Parameterized(p) => self.realize_parameterized(p).to_enclosing_type(),
        }
    }

    fn realize_type_arg(&self, type_arg: &TypeArg<'a>) -> TypeArg<'a> {
        match type_arg {
            TypeArg::Class(c) => TypeArg::Class(self.realize_class(c)),
            TypeArg::Parameterized(p) => self.realize_parameterized(p).to_type_arg(),
            TypeArg::Array(a) => TypeArg::Array(self.realize_array(a)),
            TypeArg::Wildcard(w) => TypeArg::Wildcard(self.realize_wildcard(w)),
            TypeArg::Primitive(p) => TypeArg::Primitive(p.clone()),
        }
    }

    fn realize_wildcard(&self, wildcard: &WildcardType<'a>) -> WildcardType<'a> {
        WildcardType {
            span_opt: wildcard.span_opt,
            extends: {
                let mut extends = vec![];
                for ex in &wildcard.extends {
                    extends.push(self.realize_reference(ex));
                }
                extends
            },
            super_opt: wildcard
                .super_opt
                .as_ref()
                .map(|s| Box::new(self.realize_reference((*s).as_ref()))),
        }
    }

    fn realize_reference(&self, reference: &ReferenceType<'a>) -> ReferenceType<'a> {
        match reference {
            ReferenceType::Parameterized(p) => self.realize_parameterized(p).to_reference_type(),
            ReferenceType::Class(c) => ReferenceType::Class(self.realize_class(c)),
            ReferenceType::Array(a) => ReferenceType::Array(self.realize_array(a)),
        }
    }

    fn realize_parameterized_from_prefix(
        &self,
        parameterized: &ParameterizedType<'a>,
    ) -> Option<Type<'a>> {
        match &self.prefix_opt {
            Some(prefix) => match (*prefix).as_ref() {
                EnclosingType::Package(_) => None,
                EnclosingType::Parameterized(_) => None,
                EnclosingType::Class(c) => Some(c.realize_parameterized(parameterized)),
            },
            None => None,
        }
    }

    fn realize_parameterized_from_current(
        &self,
        parameterized: &ParameterizedType<'a>,
    ) -> Option<Type<'a>> {
        let def = if let Some(def) = self.def_opt {
            unsafe { &*def }
        } else {
            return None;
        };

        let type_args = if let Some(type_args) = &self.type_args_opt {
            type_args
        } else {
            return None;
        };

        for (param, arg) in def.type_params.iter().zip(type_args.iter()) {
            if &param.name == &parameterized.name {
                return Some(arg.to_type());
            }
        }

        None
    }

    fn realize_parameterized(&self, parameterized: &ParameterizedType<'a>) -> Type<'a> {
        if let Some(realized) = self.realize_parameterized_from_current(parameterized) {
            return realized;
        }

        if let Some(realized) = self.realize_parameterized_from_prefix(parameterized) {
            return realized;
        }

        return Type::Parameterized(parameterized.clone());
    }

    fn realize_class(&self, class: &ClassType<'a>) -> ClassType<'a> {
        ClassType {
            prefix_opt: class
                .prefix_opt
                .as_ref()
                .map(|p| Box::new(self.realize_enclosing(&p))),
            name: class.name.to_owned(),
            span_opt: class.span_opt,
            type_args_opt: class.type_args_opt.as_ref().map(|type_args| {
                let mut realizeds = vec![];
                for t in type_args {
                    realizeds.push(self.realize_type_arg(t))
                }
                realizeds
            }),
            def_opt: class.def_opt.clone(),
        }
    }

    fn realize_array(&self, array: &ArrayType<'a>) -> ArrayType<'a> {
        ArrayType {
            tpe: Box::new(self.realize(&array.tpe)),
            size_opt: array.size_opt.clone(),
            underlying: self.realize_class(&array.underlying),
        }
    }

    pub fn realize(&self, tpe: &Type<'a>) -> Type<'a> {
        match tpe {
            Type::Parameterized(p) => self.realize_parameterized(p),
            Type::Class(c) => Type::Class(self.realize_class(c)),
            Type::Array(a) => Type::Array(self.realize_array(a)),
            Type::Wildcard(w) => Type::Wildcard(self.realize_wildcard(w)),
            Type::Primitive(_) => tpe.clone(),
            Type::Void(_) => tpe.clone(),
            Type::UnknownType => Type::UnknownType,
        }
    }
}

pub static NATIVE_ARRAY_CLASS_NAME: &str = "NATIVE:Array";

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayType<'a> {
    pub tpe: Box<Type<'a>>,
    pub size_opt: Option<Box<Expr<'a>>>,
    pub underlying: ClassType<'a>,
}

impl<'a> ArrayType<'a> {
    pub fn find_field(&self, name: &str, context: &InvocationContext) -> Option<Field<'a>> {
        self.underlying.find_field(name, context)
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParameterizedType<'a> {
    pub name: String,
    pub span_opt: Option<Span<'a>>,
    pub def: *const analyze::definition::TypeParam<'a>,
}

impl<'a> ParameterizedType<'a> {
    pub fn set_span_opt(&mut self, span_opt: Option<&Span<'a>>) {
        self.span_opt = span_opt.map(|s| s.clone());
    }

    pub fn find_inner_class(&self, name: &str) -> Option<ClassType<'a>> {
        let type_param = unsafe { &(*self.def) };

        for extend in type_param.extends.borrow().iter() {
            if let Some(inner) = extend.find_inner_class(name) {
                return Some(inner);
            }
        }

        None
    }

    pub fn find_field(&self, name: &str, context: &InvocationContext) -> Option<Field<'a>> {
        let def = unsafe { &*self.def };

        for extend in def.extends.borrow().deref() {
            if let Some(field) = extend.find_field(name, context) {
                return Some(field);
            }
        }

        None
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct TypeParam<'a> {
    pub name: Span<'a>,
    pub extends: Vec<ClassType<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Param<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub tpe: Type<'a>,
    pub is_varargs: bool,
    pub name: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Constructor<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub type_params: Vec<TypeParam<'a>>,
    pub name: Span<'a>,
    pub params: Vec<Param<'a>>,
    pub throws: Vec<ClassType<'a>>,
    pub block: Block<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Method<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub type_params: Vec<TypeParam<'a>>,
    pub return_type: Type<'a>,
    pub name: Span<'a>,
    pub params: Vec<Param<'a>>,
    pub throws: Vec<ClassType<'a>>,
    pub block_opt: Option<Block<'a>>,
    pub def_opt: RefCell<Option<*const analyze::definition::Method<'a>>>,
    pub id: String,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Statement<'a> {
    Assert(Assert<'a>),
    Block(Block<'a>),
    Break(Break<'a>),
    Class(Class<'a>),
    Continue(Continue<'a>),
    Empty,
    DoWhile(DoWhile<'a>),
    Expr(Expr<'a>),
    ForLoop(ForLoop<'a>),
    Foreach(Foreach<'a>),
    IfElse(IfElse<'a>),
    Labeled(Labeled<'a>),
    Return(ReturnStmt<'a>),
    Switch(Switch<'a>),
    Synchronized(Synchronized<'a>),
    Throw(Throw<'a>),
    Try(Try<'a>),
    WhileLoop(WhileLoop<'a>),
    VariableDeclarators(VariableDeclarators<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assert<'a> {
    pub expr: Expr<'a>,
    pub error_opt: Option<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Labeled<'a> {
    pub label: Span<'a>,
    pub statement: Box<Statement<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Continue<'a> {
    pub identifier_opt: Option<Span<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Break<'a> {
    pub identifier_opt: Option<Span<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Switch<'a> {
    pub expr: Box<Expr<'a>>,
    pub cases: Vec<Case<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Case<'a> {
    pub label_opt: Option<Box<Expr<'a>>>,
    pub stmts: Vec<Statement<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct DoWhile<'a> {
    pub block: Block<'a>,
    pub cond: Box<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct WhileLoop<'a> {
    pub cond: Box<Expr<'a>>,
    pub block: Block<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Try<'a> {
    pub try: Block<'a>,
    pub resources: Vec<TryResource<'a>>,
    pub catches: Vec<Catch<'a>>,
    pub finally_opt: Option<Block<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum TryResource<'a> {
    Name(Name<'a>),
    Declarator(StandaloneVariableDeclarator<'a>),
    FieldAccess(FieldAccess<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Catch<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub param_name: Span<'a>,
    pub class_types: Vec<ClassType<'a>>,
    pub block: Block<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Synchronized<'a> {
    pub expr: Box<Expr<'a>>,
    pub block: Block<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Throw<'a> {
    pub expr: Expr<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Foreach<'a> {
    pub declarator: StandaloneVariableDeclarator<'a>,
    pub expr: Expr<'a>,
    pub block: Block<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ForLoop<'a> {
    pub inits: Vec<Statement<'a>>,
    pub cond_opt: Option<Expr<'a>>,
    pub updates: Vec<Statement<'a>>,
    pub block: Block<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct IfElse<'a> {
    pub cond: Expr<'a>,
    pub block: Block<'a>,
    pub else_block_opt: Option<Block<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ReturnStmt<'a> {
    pub expr_opt: Option<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VariableDeclarators<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub declarators: Vec<VariableDeclarator<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct StandaloneVariableDeclarator<'a> {
    pub modifiers: Vec<Modifier<'a>>,
    pub tpe: RefCell<Type<'a>>,
    pub name: Span<'a>,
    pub expr_opt: Option<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct VariableDeclarator<'a> {
    pub tpe: RefCell<Type<'a>>,
    pub name: Span<'a>,
    pub expr_opt: Option<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Expr<'a> {
    ArrayAccess(ArrayAccess<'a>),
    ArrayInitializer(ArrayInitializer<'a>),
    Assignment(Assignment<'a>),
    BinaryOperation(BinaryOperation<'a>),
    Boolean(Boolean<'a>),
    Cast(Cast<'a>),
    Char(Char<'a>),
    ConstructorReference(ConstructorReference<'a>),
    Double(Double<'a>),
    FieldAccess(FieldAccess<'a>),
    Float(Float<'a>),
    Hex(Hex<'a>),
    InstanceOf(InstanceOf<'a>),
    Int(Int<'a>),
    Lambda(Lambda<'a>),
    Long(Long<'a>),
    MethodCall(MethodCall<'a>),
    MethodReference(MethodReference<'a>),
    Name(Name<'a>),
    NewArray(NewArray<'a>),
    NewObject(NewObject<'a>),
    Null(Null<'a>),
    Class(ClassExpr<'a>),
    StaticClass(StaticClass<'a>),
    String(LiteralString<'a>),
    Super(Super<'a>),
    SuperConstructorCall(SuperConstructorCall<'a>),
    This(This<'a>),
    ThisConstructorCall(ThisConstructorCall<'a>),
    Ternary(Ternary<'a>),
    UnaryOperation(UnaryOperation<'a>),
}

impl<'a> Expr<'a> {
    pub fn tpe_opt(&self) -> Option<Type<'a>> {
        match self {
            Expr::FieldAccess(f) => f.def_opt.borrow().as_ref().map(|f| f.tpe.clone()),
            Expr::Name(n) => {
                if let Some(resolved) = n.resolved_opt.get() {
                    resolved.tpe_opt()
                } else {
                    None
                }
            }
            Expr::StaticClass(s) => None,
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct StaticClass<'a> {
    pub tpe: StaticType<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct This<'a> {
    pub tpe_opt: Option<Type<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Super<'a> {
    pub tpe_opt: Option<Type<'a>>,
    pub span: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Char<'a> {
    pub value: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ConstructorReference<'a> {
    pub tpe: ReferenceType<'a>,
    pub type_args_opt: Option<Vec<TypeArg<'a>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodReference<'a> {
    pub primary: MethodReferencePrimary<'a>,
    pub type_args_opt: Option<Vec<TypeArg<'a>>>,
    pub name: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum MethodReferencePrimary<'a> {
    Class(ClassType<'a>),
    Array(ArrayType<'a>),
    Expr(Box<Expr<'a>>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Ternary<'a> {
    pub cond: Box<Expr<'a>>,
    pub true_expr: Box<Expr<'a>>,
    pub false_expr: Box<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayAccess<'a> {
    pub expr: Box<Expr<'a>>,
    pub index: Box<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Keyword<'a> {
    pub name: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Name<'a> {
    pub name: Span<'a>,
    pub resolved_opt: Cell<Option<ResolvedName<'a>>>,
}

#[derive(Debug, PartialEq, Copy, Clone)]
pub enum ResolvedName<'def> {
    Package(*const analyze::definition::Package<'def>),
    Class(*const analyze::definition::Class<'def>),
    Variable(*const VariableDeclarator<'def>),
    TypeParam(*const analyze::definition::TypeParam<'def>),
}

impl<'def> ResolvedName<'def> {
    pub fn tpe_opt(&self) -> Option<Type<'def>> {
        match self {
            ResolvedName::Variable(v) => Some(unsafe { &**v }.tpe.borrow().deref().clone()),
            _ => None,
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct SuperConstructorCall<'a> {
    pub prefix_opt: Option<Box<Expr<'a>>>,
    pub type_args_opt: Option<Vec<TypeArg<'a>>>,
    pub name: Span<'a>,
    pub args: Vec<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ThisConstructorCall<'a> {
    pub type_args_opt: Option<Vec<TypeArg<'a>>>,
    pub name: Span<'a>,
    pub args: Vec<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct MethodCall<'a> {
    pub prefix_opt: Option<Box<Expr<'a>>>,
    pub name: Span<'a>,
    pub type_args_opt: Option<Vec<TypeArg<'a>>>,
    pub args: Vec<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Lambda<'a> {
    pub params: Vec<Param<'a>>,
    pub expr_opt: Option<Box<Expr<'a>>>,
    pub block_opt: Option<Block<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NewArray<'a> {
    pub tpe: ArrayType<'a>,
    pub initializer_opt: Option<ArrayInitializer<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayInitializer<'a> {
    pub items: Vec<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct NewObject<'a> {
    pub prefix_opt: Option<Box<Expr<'a>>>,
    pub tpe: ClassType<'a>,
    pub constructor_type_args_opt: Option<Vec<TypeArg<'a>>>,
    pub args: Vec<Expr<'a>>,
    pub body_opt: Option<ClassBody<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Hex<'a> {
    pub value: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Double<'a> {
    pub value: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Float<'a> {
    pub value: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Long<'a> {
    pub value: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Int<'a> {
    pub value: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Boolean<'a> {
    pub value: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Null<'a> {
    pub value: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct LiteralString<'a> {
    pub value: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum Assigned<'a> {
    Name(Name<'a>),
    ArrayAccess(ArrayAccess<'a>),
    Field(FieldAccess<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct ClassExpr<'a> {
    pub tpe: Type<'a>,
    pub span: Span<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct FieldAccess<'a> {
    pub prefix: RefCell<Box<FieldAccessPrefix<'a>>>,
    pub name: Span<'a>,
    pub def_opt: RefCell<Option<Field<'a>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub enum FieldAccessPrefix<'a> {
    Package(PackagePrefix<'a>),
    Expr(Expr<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub struct Assignment<'a> {
    pub assigned: Box<Assigned<'a>>,
    pub operator: Span<'a>,
    pub expr: Box<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct InstanceOf<'a> {
    pub expr: Box<Expr<'a>>,
    pub operator: Span<'a>,
    pub tpe: Type<'a>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct BinaryOperation<'a> {
    pub left: Box<Expr<'a>>,
    pub operator: Span<'a>,
    pub right: Box<Expr<'a>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct UnaryOperation<'a> {
    pub expr: Box<Expr<'a>>,
    pub operator: Span<'a>,
    pub is_post: bool,
}

#[derive(Debug, PartialEq, Clone)]
pub struct Cast<'a> {
    pub tpes: Vec<Type<'a>>,
    pub expr: Box<Expr<'a>>,
}
