use analyze;
use std::cell::{Cell, Ref, RefCell};
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
    Class(*const Class<'a>),
}

#[derive(Debug, PartialEq, Clone)]
pub enum ImportPrefixDef<'a> {
    Package(*const analyze::definition::Package<'a>),
    Class(*const Class<'a>),
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
pub enum Type<'a> {
    Class(ClassType<'a>),
    Primitive(PrimitiveType<'a>),
    Array(ArrayType<'a>),
    Parameterized(ParameterizedType<'a>),
    Void(Void<'a>),
    UnknownType,
}

impl<'a> Type<'a> {
    pub fn to_type_arg(self) -> TypeArg<'a> {
        match self {
            Type::Array(arr) => TypeArg::Array(arr),
            Type::Class(class) => TypeArg::Class(class),
            Type::Parameterized(parameterized) => TypeArg::Parameterized(parameterized),
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
            Type::Void(_) => panic!(),
            Type::Primitive(_) => panic!(),
            Type::UnknownType => panic!(),
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct Void<'a> {
    pub span: Span<'a>,
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
    pub name: Span<'a>,
    pub def: *const analyze::definition::Package<'a>,
}

impl<'a> PackagePrefix<'a> {
    pub fn find(&self, name: &Span<'a>) -> Option<EnclosingType<'a>> {
        let def = unsafe { &(*self.def) };
        def.find(name.fragment).map(|e| e.to_type(name))
    }
}

#[derive(Debug, PartialEq, Clone)]
pub enum EnclosingType<'a> {
    Package(PackagePrefix<'a>),
    Class(ClassType<'a>),
    Parameterized(ParameterizedType<'a>),
}

impl<'a> EnclosingType<'a> {
    pub fn get_name(&self) -> &Span<'a> {
        match self {
            EnclosingType::Package(package) => &package.name,
            EnclosingType::Class(class) => &class.name,
            EnclosingType::Parameterized(p) => &p.name,
        }
    }
    pub fn find(&self, name: &Span<'a>) -> Option<EnclosingType<'a>> {
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
                name: package.name.clone(),
                def: package.def,
            }),
            EnclosingType::Class(class) => EnclosingType::Class(ClassType {
                prefix_opt: prefix_opt.map(Box::new),
                name: class.name.clone(),
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
}

#[derive(Debug, PartialEq, Clone)]
pub struct WildcardType<'a> {
    pub name: Span<'a>,
    pub extends: Vec<ReferenceType<'a>>,
    pub super_opt: Option<Box<ReferenceType<'a>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct PrimitiveType<'a> {
    pub name: Span<'a>,
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
    pub name: Span<'a>,
    pub type_args_opt: Option<Vec<TypeArg<'a>>>,
    pub def_opt: Option<*const analyze::definition::Class<'a>>,
}

impl<'a> ClassType<'a> {
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

    pub fn find_inner_class(&self, name: &Span<'a>) -> Option<ClassType<'a>> {
        let class = if let Some(class) = self.def_opt {
            unsafe { &(*class) }
        } else {
            return None;
        };

        match class.find(name.fragment) {
            Some(found) => {
                // TODO: transfer type args
                return Some(found.to_type(name));
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
}

#[derive(Debug, PartialEq, Clone)]
pub struct ArrayType<'a> {
    pub tpe: Box<Type<'a>>,
    pub size_opt: Option<Box<Expr<'a>>>,
}

#[derive(Debug, PartialEq, Clone)]
pub struct ParameterizedType<'a> {
    pub name: Span<'a>,
    pub def: *const analyze::definition::TypeParam<'a>,
}

impl<'a> ParameterizedType<'a> {
    pub fn find_inner_class(&self, name: &Span<'a>) -> Option<ClassType<'a>> {
        let type_param = unsafe { &(*self.def) };

        for extend in type_param.extends.borrow().iter() {
            if let Some(inner) = extend.find_inner_class(name) {
                return Some(inner);
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
    String(LiteralString<'a>),
    Super(Super<'a>),
    SuperConstructorCall(SuperConstructorCall<'a>),
    This(This<'a>),
    ThisConstructorCall(ThisConstructorCall<'a>),
    Ternary(Ternary<'a>),
    UnaryOperation(UnaryOperation<'a>),
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
    pub expr: Box<Expr<'a>>,
    pub field: Name<'a>,
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
