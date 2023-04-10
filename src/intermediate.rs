pub mod dictionary {
    use crate::{
        lexer::tokenizer::{Tokens, Operators},
        tree_walker::tree_walker::{self, ArgNodeType, Err, Node},
    };
    use core::panic;
    use std::{collections::HashMap, fs::DirEntry};

    use super::AnalyzationError::{self, ErrType};

    pub fn from_ast(ast: &HashMap<String, tree_walker::ArgNodeType>) {
        let mut global_dict = Dictionary::new();
        let mut errors = Vec::new();
        if let Some(ArgNodeType::Array(entry)) = ast.get("nodes") {
            load_dictionary(entry, &mut global_dict, &mut errors)
        }
        println!("global: {global_dict:?}");
        println!("errors: {errors:?}");
        use crate::type_check::TypesCheck;
        TypesCheck::index_types(&mut global_dict);
    }
    fn load_dictionary(nodes: &Vec<Node>, dictionary: &mut Dictionary, errors: &mut Vec<ErrType>) {
        for node in nodes {
            load_node(node, dictionary, errors);
        }
    }
    fn load_node(node: &Node, dictionary: &mut Dictionary, errors: &mut Vec<ErrType>) {
        let name = if let Tokens::Text(name) = &node.name {
            name
        } else {
            panic!()
        };
        match name.as_str() {
            "KWEnum" => {
                let mut result = Enum {
                    identifier: get_ident(&node),
                    keys: vec![],
                    src_loc: 0,
                    methods: vec![],
                    overloads: vec![],
                };
                for enum_value in step_inside_arr(&node, "values") {
                    let n = if let Tokens::Number(n, _, _) = get_token(&enum_value, "default") {
                        *n
                    } else {
                        let len = result.keys.len() - 1;
                        result.keys[len].1 + 1
                    };
                    let ident = get_ident(&enum_value);
                    for variant in &result.keys {
                        if variant.1 == n {
                            errors.push(ErrType::EnumVariantAssignedNumber(n, (0, 0)))
                        }
                        if variant.0 == ident {
                            errors
                                .push(ErrType::EnumVariantAssignedIdent(ident.to_string(), (0, 0)))
                        }
                    }
                    result.keys.push((ident, n));
                }
                if dictionary.register_id(result.identifier.to_string(), IdentifierKinds::Enum) {
                    dictionary.enums.push(result);
                } else {
                    errors.push(ErrType::ConflictingNames(result.identifier.to_string()))
                }
            }
            "KWType" => {
                let name = get_ident(&node);
                if dictionary.register_id(name.to_string(), IdentifierKinds::Type) {
                    dictionary.types.push(TypeDef {
                        kind: get_type(step_inside_val(&node, "type"), errors),
                        identifier: name,
                        generics: get_generics_decl(&node, errors),
                        overloads: vec![],
                        methods: vec![],
                        public: public(&node),
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(name.to_string()))
                }
            }
            "KWStruct" => {
                let mut result = Struct {
                    identifier: get_ident(node),
                    fields: Vec::new(),
                    src_loc: 0,
                    generics: get_generics_decl(node, errors),
                    traits: Vec::new(),
                    public: public(&node),
                };
                for key in step_inside_arr(node, "keys") {
                    let ident = get_ident(&key);
                    for field in &result.fields {
                        if *field.0 == ident {
                            errors.push(ErrType::StructVariantAssignedIdent(
                                ident.to_string(),
                                (0, 0),
                            ))
                        }
                    }
                    result.fields.push((
                        get_ident(key),
                        get_type(step_inside_val(key, "type"), errors),
                    ))
                }
                if dictionary.register_id(result.identifier.to_string(), IdentifierKinds::Struct) {
                    dictionary.structs.push(result);
                } else {
                    errors.push(ErrType::ConflictingNames(result.identifier.to_string()))
                }
            }
            "KWImport" => {
                let path = if let Tokens::String(path) = get_token(node, "path") {
                    path
                } else {
                    panic!("nemozne")
                };
                let name = if let Some(txt) = try_get_ident(node) {
                    Some(txt)
                } else {
                    None
                };
                match name {
                    Some(name) => {
                        if dictionary.register_id(name.to_string(), IdentifierKinds::Namespace) {
                            // TODO: read file and compile it into dictionary
                        } else {
                            errors.push(ErrType::ConflictingNames(name.to_string()))
                        }
                    }
                    None => {
                        // TODO: read file and compile it into dictionary
                    }
                }
            }
            "KWFun" => {
                let fun = get_fun_siginifier(&node, errors);
                println!("fun: {fun:?}");
                let name = fun
                    .identifier
                    .clone()
                    .expect("global function cannot be anonymous");
                if dictionary.register_id(String::from(&name), IdentifierKinds::Function) {
                    dictionary.functions.push(fun);
                } else {
                    errors.push(ErrType::ConflictingNames(String::from(&name)))
                }
            }
            "KWLet" => {
                let identifier = get_ident(&node);
                let undefKind = true;
                let kind = if let Tokens::Text(txt) = &step_inside_val(node, "type").name {
                    if txt == "type_specifier" {
                        Some(get_type(
                            step_inside_val(step_inside_val(node, "type"), "type"),
                            errors,
                        ))
                    } else {
                        None
                    }
                } else {
                    None
                };
                if dictionary.register_id(identifier.to_string(), IdentifierKinds::Variable) {
                    dictionary.variables.push(Variable {
                        kind,
                        identifier,
                        location: 0,
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(identifier.to_string()))
                }
            }
            "KWConst" => {
                let identifier = get_ident(&node);
                let kind = get_type(
                    step_inside_val(&step_inside_val(&node, "type"), "type"),
                    errors,
                );
                if dictionary.register_id(identifier.to_string(), IdentifierKinds::Variable) {
                    dictionary.constants.push(Constant {
                        kind,
                        identifier,
                        location: 0,
                        public: public(&node),
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(identifier.to_string()))
                }
            }
            "KWImpl" => {
                let ident = get_nested_ident(&step_inside_val(&node, "identifier"), errors);
                let mut functions = Vec::new();
                let mut overloads = Vec::new();
                let traits = get_traits(&node, errors);
                for method in step_inside_arr(&node, "methods") {
                    if let Tokens::Text(txt) = &method.name {
                        match txt.as_str() {
                            "KWOverload" => {
                                overloads.push(get_overload_siginifier(&method, errors))
                            }
                            "KWFun" => {
                                functions.push(get_fun_siginifier(&method, errors));
                            }
                            _ => {}
                        }
                    }
                }
                dictionary.implementations.push(Implementation {
                    target: ident,
                    traits,
                    functions,
                    overloads,
                    src_loc: 0,
                })
            }
            "KWTrait" => {
                let is_pub = public(&node);
                let identifier = get_ident(&node);
                let mut functions = Vec::new();
                let mut overloads = Vec::new();
                let traits = get_traits(&node, errors);
                for method in step_inside_arr(&node, "methods") {
                    if let Tokens::Text(txt) = &method.name {
                        match txt.as_str() {
                            "KWOverload" => {
                                overloads.push(get_overload_siginifier(&method, errors))
                            }
                            "KWFun" => {
                                functions.push(get_fun_siginifier(&method, errors));
                            }
                            _ => {}
                        }
                    }
                }
                if dictionary.register_id(identifier.to_string(), IdentifierKinds::Trait) {
                    dictionary.traits.push(Trait {
                        identifier,
                        methods: functions,
                        overloads,
                        traits,
                        src_loc: 0,
                        public: is_pub,
                    })
                } else {
                    errors.push(ErrType::ConflictingNames(identifier.to_string()))
                }
            }
            "expression" => {}
            _ => {}
        }
    }
    fn get_traits(node: &Node, errors: &mut Vec<ErrType>) -> Vec<NestedIdent> {
        let mut result = vec![];
        for tr in step_inside_arr(&node, "traits") {
            result.push(get_nested_ident(tr, errors));
        }
        result
    }
    fn get_nested_ident(node: &Node, errors: &mut Vec<ErrType>) -> NestedIdent {
        let mut result = vec![];
        for nd in step_inside_arr(node, "nodes") {
            if let Tokens::Text(txt) = &step_inside_val(nd, "identifier").name {
                result.push(txt.to_string());
            } else {
                panic!()
            }
        }
        result
    }
    fn get_overload_siginifier(node: &Node, errors: &mut Vec<ErrType>) -> Overload {
        let operator = get_operator(step_inside_val(&node, "op"));
        let generics = get_generics_decl(&node, errors);
        let kind = if let Some(kind) = try_step_inside_val(step_inside_val(&node, "type"), "type") {
            Some(get_type(kind, errors))
        } else {
            None
        };
        let arg = step_inside_val(&node, "arg");
        Overload {
            operator,
            arg: (
                get_ident(&arg),
                get_type(step_inside_val(&arg, "type"), errors),
            ),
            stack_size: None,
            location: None,
            return_type: kind,
            generics,
            src_loc: 0,
            public: public(&node),
        }
    }
    fn get_fun_siginifier(node: &Node, errors: &mut Vec<ErrType>) -> Function {
        let identifier = if node.nodes.contains_key("identifier") {
            Some(get_ident(&node))
        } else {
            None
        };
        let generics = get_generics_decl(&node, errors);
        let kind = if let Some(kind) = try_step_inside_val(step_inside_val(&node, "type"), "type") {
            Some(get_type(kind, errors))
        } else {
            None
        };
        let mut args = Vec::new();
        for arg in step_inside_arr(node, "arguments") {
            if let Tokens::Text(name) = &arg.name {
                match name.as_str() {
                    "self_arg" => {
                        args.push((
                            String::from("self"),
                            ShallowType {
                                is_fun: None,
                                arr_len: None,
                                refs: count_refs(&arg),
                                main: vec![String::from("Self")],
                                generics: Vec::new(),
                            },
                        ));
                    }
                    "arg" => {
                        let ident = get_ident(arg);
                        for (arg_ident, _) in &args {
                            if *arg_ident == ident {
                                errors.push(ErrType::ConflictingArgsName(ident.to_string()));
                            }
                        }
                        args.push((ident, get_type(step_inside_val(&arg, "type"), errors)))
                    }
                    _ => {
                        panic!("this should never happen")
                    }
                }
            }
        }
        let can_yeet = step_inside_val(&node, "errorable").name
            == Tokens::Operator(crate::lexer::tokenizer::Operators::Not);
        let public = if node.nodes.contains_key("public") {
            public(&node)
        } else {
            true
        };
        /*

        to read the dictionary, you need to do this:

        let mut dict = Dictionary::new();
        load_dictionary(step_inside_arr(step_inside_val(node, "code"), "nodes"), &mut dict, &mut vec![]);
         */
        Function {
            can_yeet,
            identifier,
            args,
            stack_size: None,
            location: None,
            return_type: kind,
            generics,
            src_loc: 0,
            public: false,
        }
    }
    fn public(node: &Node) -> bool {
        if let Tokens::Text(txt) = &step_inside_val(node, "public").name {
            return txt == "pub";
        }
        false
    }
    fn get_operator(node: &Node) -> Tokens {
        step_inside_val(node, "op").name.clone()
    }
    fn get_ident(node: &Node) -> String {
        if let Tokens::Text(txt) =
            &step_inside_val(&step_inside_val(&node, "identifier"), "identifier").name
        {
            return txt.to_string();
        }
        panic!();
    }
    fn try_get_ident(node: &Node) -> Option<String> {
        if let Some(val) = try_step_inside_val(&step_inside_val(&node, "identifier"), "identifier")
        {
            if let Tokens::Text(txt) = &val.name {
                return Some(txt.to_string());
            }
        }
        None
    }
    fn count_refs(node: &Node) -> usize {
        let mut refs = 0;
        if let Some(arr) = try_step_inside_arr(&step_inside_val(&node, "ref"), "refs") {
            for ref_type in arr {
                if let Tokens::Operator(Operators::Ampersant) = ref_type.name {
                    refs += 1;
                }
                if let Tokens::Operator(crate::lexer::tokenizer::Operators::And) = ref_type.name {
                    refs += 2;
                }
            }
        }
        refs
    }
    fn get_type(node: &Node, errors: &mut Vec<ErrType>) -> ShallowType {
        let main = step_inside_val(&node, "main");
        if main.name == Tokens::Text(String::from("function_head")) {
            let fun = get_fun_siginifier(&main, errors);
            let refs = count_refs(&node);
            return ShallowType {
                is_fun: Some(Box::new(fun)),
                arr_len: None,
                refs,
                main: vec![],
                generics: Vec::new(),
            };
        }
        let mut arr_len = None;
        let mut refs = count_refs(node);
        let main = if let Some(type_ident) =
            try_step_inside_arr(step_inside_val(&node, "main"), "nodes")
        {
            let mut main = Vec::new();
            for path_part in type_ident {
                if let Tokens::Text(txt) = get_token(path_part, "identifier") {
                    main.push(txt.to_string())
                }
            }
            main
        } else {
            let mut main = vec![];
            let arr = step_inside_val(&node, "arr");
            if let Some(arr) = try_step_inside_arr(
                step_inside_val(step_inside_val(&arr, "type"), "main"),
                "nodes",
            ) {
                for path_part in arr {
                    if let Tokens::Text(txt) = get_token(path_part, "identifier") {
                        main.push(txt.to_string())
                    }
                }
            }
            // length will be calculated later since it might be a constant or an expression with constant value
            // consts will be evaluated after the dictionary is loaded
            arr_len = Some(0);
            main
        };
        ShallowType {
            is_fun: None,
            arr_len,
            refs,
            main,
            generics: get_generics_expr(node, errors),
        }
    }
    fn get_generics_expr(node: &Node, errors: &mut Vec<ErrType>) -> GenericExpr {
        let mut result = Vec::new();
        if let Some(arr) = try_step_inside_arr(step_inside_val(node, "generic"), "types") {
            for generic_expr in arr {
                result.push(get_type(generic_expr, errors));
            }
        }
        result
    }
    fn get_generics_decl<'a>(node: &'a Node, errors: &mut Vec<ErrType>) -> Vec<GenericDecl> {
        let mut generics = Vec::new();
        if let Some(arr) = try_step_inside_arr(step_inside_val(&node, "generic"), "identifiers") {
            for generic in arr {
                let mut traits = Vec::new();
                for ident in step_inside_arr(generic, "traits") {
                    traits.push(get_nested_ident(&ident, errors));
                }
                generics.push(GenericDecl {
                    identifier: get_ident(generic),
                    traits,
                })
            }
        }
        generics
    }
    fn get_token<'a>(node: &'a Node, ident: &'a str) -> &'a Tokens {
        return &step_inside_val(&node, ident).name;
    }
    fn step_inside_val<'a>(node: &'a Node, ident: &'a str) -> &'a Node {
        node.nodes.get(ident).unwrap().get_value()
    }
    fn try_step_inside_val<'a>(node: &'a Node, ident: &'a str) -> Option<&'a Node> {
        match node.nodes.get(ident) {
            Some(arr) => Some(arr.get_value()),
            None => None,
        }
    }
    fn step_inside_arr<'a>(node: &'a Node, ident: &'a str) -> &'a Vec<Node> {
        node.nodes.get(ident).unwrap().get_array()
    }
    fn try_step_inside_arr<'a>(node: &'a Node, ident: &'a str) -> Option<&'a Vec<Node>> {
        match node.nodes.get(ident) {
            Some(arr) => Some(arr.get_array()),
            None => None,
        }
    }
    /// all of the defined types/variables (enum, struct, function) in the current scope will be registered here
    #[derive(Debug)]
    pub struct Dictionary {
        pub functions: Vec<Function>,
        pub types: Vec<TypeDef>,
        pub enums: Vec<Enum>,
        pub structs: Vec<Struct>,
        pub variables: Vec<Variable>,
        pub constants: Vec<Constant>,
        pub identifiers: Vec<(String, IdentifierKinds)>,
        pub imports: Vec<Dictionary>,
        pub implementations: Vec<Implementation>,
        pub traits: Vec<Trait>,
    }
    #[derive(Debug)]
    pub struct Trait {
        pub identifier: String,
        pub methods: Vec<Function>,
        pub overloads: Vec<Overload>,
        // dependences
        pub traits: Vec<NestedIdent>,
        pub src_loc: usize,
        pub public: bool,
    }
    #[derive(Debug, Clone)]
    pub enum IdentifierKinds {
        Function,
        Type,
        Enum,
        Struct,
        Variable,
        Namespace,
        Trait,
    }
    #[derive(Debug)]
    pub struct TypeDef {
        pub kind: ShallowType,
        pub identifier: String,
        pub generics: Vec<GenericDecl>,
        pub public: bool,
        pub overloads: Vec<Overload>,
        pub methods: Vec<Function>,
    }
    #[derive(Debug)]
    pub struct GenericDecl {
        pub identifier: String,
        pub traits: Vec<NestedIdent>,
    }
    #[derive(Debug)]
    pub struct Function {
        /// function identifiers will be changed to allow for function overload
        /// name mangler rules: "{identifier}:{args.foreach("{typeof}:")}"
        /// example:
        /// fun myFun(n: int, type: char): int
        /// fun nothing()
        /// translates to:
        /// "myFun:int:char"
        /// "nothing:"
        pub identifier: Option<String>,
        /// type of args in order
        pub args: Vec<(String, ShallowType)>,
        /// size needed to allocate on stack while function call (args.len() included)
        pub stack_size: Option<usize>,
        /// location in bytecode, so runtime knows where to jump
        pub location: Option<usize>,
        pub return_type: Option<ShallowType>,
        pub can_yeet: bool,
        pub generics: Vec<GenericDecl>,
        /// location in source code
        pub src_loc: usize,
        pub public: bool,
    }
    #[derive(Debug)]
    pub struct Overload {
        pub operator: Tokens,
        /// type of args in order
        pub arg: (String, ShallowType),
        /// size needed to allocate on stack while function call (args.len() included)
        pub stack_size: Option<usize>,
        /// location in bytecode, so runtime knows where to jump
        pub location: Option<usize>,
        pub return_type: Option<ShallowType>,
        pub generics: Vec<GenericDecl>,
        /// location in source code
        pub src_loc: usize,
        pub public: bool,
    }
    #[derive(Debug)]
    pub struct Enum {
        pub identifier: String,
        /// enum values and their offset
        /// enum ErrCode { Continue = 100, SwitchingProtocols, ..., Ok = 200, ... }
        pub keys: Vec<(String, usize)>,
        /// location in source code
        pub src_loc: usize,
        pub methods: Vec<Function>,
        pub overloads: Vec<Overload>,
    }
    pub type NestedIdent = Vec<String>;
    #[derive(Debug)]
    pub struct Struct {
        pub generics: Vec<GenericDecl>,
        pub identifier: String,
        pub fields: Vec<(String, ShallowType)>,
        /// location in source code
        pub src_loc: usize,
        pub traits: Vec<NestedIdent>,
        pub public: bool,
    }
    #[derive(Debug)]
    pub struct Implementation {
        pub target: NestedIdent,
        pub traits: Vec<NestedIdent>,
        pub functions: Vec<Function>,
        pub overloads: Vec<Overload>,
        /// location in source code
        pub src_loc: usize,
    }
    #[derive(Debug)]
    pub struct Variable {
        pub kind: Option<ShallowType>,
        pub identifier: String,
        /// location on stack
        pub location: usize,
    }
    #[derive(Debug)]
    pub struct Constant {
        pub kind: ShallowType,
        pub identifier: String,
        /// location on stack
        pub location: usize,
        pub public: bool,
    }
    /// identifiers can not contain these characters: + - * / = % ; : , . ({<[]>}) & | ! ? " '
    /// map: let i: Int = 32; i = i + 63;
    ///     - match {keyword? => keyword(?), value? => value(?)} => keyword(let), identifier("i"), match {: => Type, = => None} => Type(Int), operator(=), value(32);
    ///     - match {keyword? => keyword(?), value? => value} => value, value("i"), operator(=), value("i"), operator(+), value(63);
    #[derive(Debug)]
    pub enum Types {
        Int,
        Float,
        Usize,
        Char,
        Byte,
        Bool,
        Null,
        /// refference type
        Pointer(Box<Types>),
        /// type of an array, lenght
        Array(Box<Types>, usize),
        /// non-primmitive types holding their identifiers
        Function(String, GenericExpr),
        Enum(String, GenericExpr),
        Struct(String, GenericExpr),
    }
    type GenericExpr = Vec<ShallowType>;

    #[derive(Debug)]
    pub struct ShallowType {
        is_fun: Option<Box<Function>>,
        /// if Some then it is an array of that length
        arr_len: Option<usize>,
        refs: usize,
        main: NestedIdent,
        generics: GenericExpr,
    }
    impl ShallowType {
        pub fn empty() -> Self {
            ShallowType {
                is_fun: None,
                arr_len: None,
                refs: 0,
                main: vec![],
                generics: vec![],
            }
        }
    }

    impl Dictionary {
        pub fn new() -> Self {
            Dictionary {
                functions: vec![],
                types: vec![],
                enums: vec![],
                structs: vec![],
                variables: vec![],
                constants: vec![],
                identifiers: vec![],
                imports: vec![],
                implementations: vec![],
                traits: vec![],
            }
        }
        fn index_of(&self, identifier: String) -> Option<usize> {
            let mut i = 0;
            loop {
                if i >= self.identifiers.len() {
                    return None;
                }
                if self.identifiers[i].0 == identifier {
                    return Some(i);
                }
                i += 1;
            }
        }
        fn type_of(&self, identifier: &str) -> Option<&IdentifierKinds> {
            for (ident, kind) in &self.identifiers {
                if ident == identifier {
                    return Some(kind);
                }
            }
            None
        }
        fn register_id(&mut self, name: String, kind: IdentifierKinds) -> bool {
            if self.contains(&name) {
                return false;
            }
            self.identifiers.push((name, kind));
            true
        }
        fn force_id(&mut self, name: String, kind: IdentifierKinds) {
            self.identifiers.push((name, kind));
        }
        fn contains(&self, name: &String) -> bool {
            for id in &self.identifiers {
                if id.0 == *name {
                    return true;
                }
            }
            false
        }
    }
}
pub mod AnalyzationError {
    use super::dictionary::IdentifierKinds;

    #[derive(Debug)]
    pub enum ErrType {
        /// assigned_number line col | occurs when you try to assign same number to two or more enum variants
        EnumVariantAssignedNumber(usize, (usize, usize)),
        /// variant_ident line col | occurs when you try to assign same identifier to two or more enum variants
        EnumVariantAssignedIdent(String, (usize, usize)),
        /// name | occurs when you try to assign same identifier twice
        ConflictingNames(String),
        /// name | occurs when you try to assign same identifier for two or more arguments
        ConflictingArgsName(String),
        /// name kind | occurs when you try to implement on non implementable identifier (implementable: enum, struct, type)
        BadImpl(String, IdentifierKinds),
        /// name kind | occurs when you try to use identifier that has not been declared
        NonExistentIdentifier(String),
        /// field line col | occurs when you try to assign same identifier to two or more struct fields
        StructVariantAssignedIdent(String, (usize, usize)),
    }
}
