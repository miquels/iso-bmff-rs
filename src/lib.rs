extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::Span;
use syn::{Ident, Lit, LitInt, LitStr, Token, spanned::Spanned, Result, Error};
use syn::{parse_macro_input, token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

// Helper macros, since the original versions from the `syn` crate
// are a bit more obtuse. This un-obstuses them.
macro_rules! bracketed {
    ($input:expr) => {{ let content; syn::bracketed!(content in $input); content }}
}
macro_rules! braced {
    ($input:expr) => {{ let content; syn::braced!(content in $input); content }}
}
macro_rules! parenthesized {
    ($input:expr) => {{ let content; syn::parenthesized!(content in $input); content }}
}

// Keywords that we reckognize and use in this module.
// Note, these are not all the keywords; some of them are
// interpreted as Idents an parsed on-the-fly.
mod kw {
    syn::custom_keyword!(extends);
    syn::custom_keyword!(optional);
    syn::custom_keyword!(rust_type);
}

// Parse a simple u32 integer.
fn parse_int(input: ParseStream) -> Result<(u32, Span)> {
    let lit: Lit = input.parse()?;
    match lit {
        Lit::Int(lit) => Ok((lit.base10_parse::<u32>()?, lit.span())),
        lit => Err(Error::new(lit.span(), "expected `number`")),
    }
}
    
// aligned(8)
#[derive(Debug)]
struct Aligned(u32);

// aligned(8)
impl Parse for Aligned {
    fn parse(input: ParseStream) -> Result<Self> {
        let content = parenthesized!(input);
        let (value, _) = parse_int(&content)?;
        Ok(Aligned(value))
    }
}

// A value: number, string, or variable.
#[derive(Debug)]
enum Value {
    Number(u64),
    String(String),
    Variable(String),
}

// A value, with a Span for error reporting.
#[derive(Debug)]
struct ValueIdent {
    span:   Span,
    value:  Value,
}

// A value, with a Span for error reporting.
impl ValueIdent {
    fn span(&self) -> Span {
        self.span.clone()
    }
}

// A value: number, string, or variable.
impl Parse for ValueIdent {
    fn parse(input: ParseStream) -> Result<Self> {

        let lookahead = input.lookahead1();
        if lookahead.peek(LitInt) {
            let val: LitInt = input.parse()?;
            return Ok(ValueIdent{
                span: val.span(),
                value: Value::Number(val.base10_parse::<u64>()?),
            });
        }
        if lookahead.peek(LitStr) {
            let val: LitStr = input.parse()?;
            return Ok(ValueIdent{
                span: val.span(),
                value: Value::String(val.value()),
            });
        }
        if lookahead.peek(Ident) {
            let val: Ident = input.parse()?;
            return Ok(ValueIdent{
                span: val.span(),
                value: Value::Variable(val.to_string()),
            });
        }
        Err(lookahead.error())
    }
}

// [optional] [template] unsigned int(8)[16] [0|var|var=0]
#[derive(Debug)]
struct VarDecl {
    optional:   bool,
    template:   bool,
    iso_type:   String,
    rust_type:  String,
    size:       u32,
    array:      Option<u32>,
    name:       String,
    default:    Option<Value>,
}

// [optional] [template] unsigned int(8)[16] [0|var|var=0]
impl Parse for VarDecl {
    fn parse(input: ParseStream) -> Result<Self> {

        let mut optional = false;
        let mut template = false;
        let mut unsigned = false;
        let mut signed = false;

        let typ = loop {
            if !input.peek(Ident) {
                return Err(input.error("expected `type`"));
            }
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "optional" => optional = true,
                "template" => template = true,
                "unsigned" => unsigned = true,
                "signed" => signed = true,
                "bit" => {
                    if signed || unsigned {
                        return Err(Error::new(ident.span(), "cannot be signed/unsigned"));
                    }
                    break ident;
                },
                "int" => {
                    if !signed && !unsigned {
                        return Err(Error::new(ident.span(), "signed/unsigned missing"));
                    }
                    break ident;
                },
                _ => return Err(Error::new(ident.span(), "expected `type`")),
            }
        };

        // Translate typ to iso_type.
        let mut iso_type = String::new();
        if optional {
            iso_type.push_str("optional ");
        }
        if template {
            iso_type.push_str("template ");
        }
        if signed {
            iso_type.push_str("signed ");
        }
        if unsigned {
            iso_type.push_str("unsigned ");
        }
        iso_type.push_str(&typ.to_string());

        // parentheses must follow.
        let content = parenthesized!(input);
        let (size, size_span) = parse_int(&content)?;

        // Translate to a rust type.
        let rust_type = match (typ.to_string().as_str(), signed, size) {
            ("int", true, 8) => "i8",
            ("int", true, 16) => "i16",
            ("int", true, 32) => "i32",
            ("int", true, 64) => "i64",
            ("int", false, 8) => "u8",
            ("int", false, 16) => "u16",
            ("int", false, 32) => "u32",
            ("int", false, 64) => "u64",
            ("int", _, size) => return Err(Error::new(size_span, format!("unsupported int({})", size))),
            ("bit", _, 1) => "bool",
            ("bit", _, 2..=8) => "u8",
            ("bit", _, 9..=16) => "u16",
            ("bit", _, 17..=23) => "u32",
            ("bit", _, 24) => "Flags",
            ("bit", _, 25..=32) => "u32",
            ("bit", _, size) => return Err(Error::new(size_span, format!("unsupported bit({})", size))),
            _ => return Err(Error::new(typ.span(), "expected `type`")),
        }.to_string();

        // It might be an array, see if a bracket follows
        let mut array = None;
        if input.peek(token::Bracket) {
            let content = bracketed!(input);
            let (n, _) = parse_int(&content)?;
            array = Some(n);
        }

        // Then the name of the variable.
        // This might be a rust keyword ...
        let name = if input.peek(token::Type) {
            let _ = input.parse::<Token![type]>()?;
            "type".to_string()
        } else {
            input.parse::<Ident>()?.to_string()
        };

        // An equals sign might be ahead.
        let mut default = None;
        if input.peek(Token![=]) {
            let _: Token![=] = input.parse()?;
            let d: ValueIdent = input.parse()?;
            default = Some(d.value);
        }

        Ok(VarDecl{
            optional,
            template,
            iso_type,
            rust_type,
            size,
            array,
            name,
            default,
        })
    }
}

// An informative comment.
#[derive(Debug)]
struct InformativeComment {
    optional:   bool,
    rust_type:  Option<String>,
}

// An informative comment.
impl Parse for InformativeComment {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut res = InformativeComment {
            optional:   false,
            rust_type:  None,
        };
        input.parse::<Token![#]>()?;
        loop {
            let lookahead = input.lookahead1();
            if lookahead.peek(kw::optional) {
                // "optional"
                input.parse::<kw::optional>()?;
                res.optional = true;
            } else if lookahead.peek(kw::rust_type) {
                // "rust_type: SomeType"
                input.parse::<kw::rust_type>()?;
                input.parse::<Token![:]>()?;
                if !input.peek(Ident) {
                    return Err(input.error("expected simple rust type"));
                }
                let i: Ident = input.parse()?;
                res.rust_type = Some(i.to_string());
            } else {
                return Err(lookahead.error());
            }
            if !input.peek(Token![;]) {
                break;
            }
            input.parse::<Token![;]>()?;
        }
        Ok(res)
    }
}

// extends Box(arg, arg ..)
#[derive(Debug)]
struct Extends {
    class:     String,
    args:       Vec<ExtendsArg>,
}

// extends Box(arg, arg ..)
impl Parse for Extends {
    fn parse(input: ParseStream) -> Result<Self> {
        // extends
        input.parse::<kw::extends>()?;
        // class name
        let class: Ident = input.parse()?;
        let content = parenthesized!(input);
        let args: Punctuated<ExtendsArg, Token![,]> = content.parse_terminated(ExtendsArg::parse)?;
        Ok(Extends {
            class:  class.to_string(),
            args:   args.into_iter().collect(),
        })
    }
}

// arguemets for extends Box(arg, arg ..)
#[derive(Debug)]
struct ExtendsArg {
    varname:    Option<String>,
    value:      Option<Value>,
}

// arguemets for extends Box(arg, arg ..)
impl Parse for ExtendsArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut varname = None;
        let mut value = None;
        let n: ValueIdent = input.parse()?;
        if input.is_empty() {
            match n.value {
                Value::Variable(var) => varname = Some(var),
                other => value = Some(other),
            }
        } else {
            match n.value {
                Value::Variable(var) => varname = Some(var),
                _ => return Err(Error::new(n.span(), "expected identifier")),
            }
            let _: token::EqEq = input.parse()?;
            let v = input.parse::<ValueIdent>()?;
            match v.value {
                Value::Variable(_) => return Err(Error::new(v.span(), "expected value")),
                other => value = Some(other),
            }
        }
        Ok(ExtendsArg {
            varname,
            value,
        })
    }
}

// Header of a class, i.e. the definitions before the body.
#[derive(Debug)]
struct ClassHeader {
    name:       Ident,
    aligned:    Option<u32>,
    args:       Vec<VarDecl>,
    extends:    Option<Extends>,
}

// Header of a class, i.e. the definitions before the body.
impl Parse for ClassHeader {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut aligned = None;

        loop {
            // as long as we see an ident.
            if !input.peek(Ident) {
                return Err(input.error("expected identfier"));
            }
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "aligned" => aligned = Some(Aligned::parse(input)?.0),
                "class" => break,
                _ => return Err(input.error("expected `class`")),
            }
        }

        // "name"
        let name: Ident = input.parse()?;

        // args
        let content = parenthesized!(input);
        let args:Punctuated<VarDecl, Token![,]> = content.parse_terminated(VarDecl::parse)?;

        let mut extends: Option<Extends> = None;
        if input.peek(kw::extends) {
            extends = Some(input.parse()?);
        }

        Ok(ClassHeader{
            name,
            aligned,
            args: args.into_iter().collect(),
            extends,
        })
    }
}

// Operators.
#[derive(Debug)]
enum Op {
    EqEq,
    Ne,
}

// Parse an operator. "=" or "!=".
impl Parse for Op {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![=]) {
            let _: token::EqEq = input.parse()?;
            Ok(Op::EqEq)
        } else if lookahead.peek(Token![!=]) {
            let _: token::Ne = input.parse()?;
            Ok(Op::Ne)
        } else {
            Err(lookahead.error())
        }
    }
}

// `if` expression. left == right, or left != right.
#[derive(Debug)]
struct IfExpr {
    left:   Value,
    right:  Value,
    op:     Op,
}

// `if` expression. left == right, or left != right.
impl Parse for IfExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let input = parenthesized!(input);
        let left: ValueIdent = input.parse()?;
        let op: Op = input.parse()?;
        let right: ValueIdent = input.parse()?;
        Ok(IfExpr {
            left: left.value,
            op,
            right: right.value,
        })
    }
}

// if (variable == some_value) { decls } else { decls }
#[derive(Debug)]
struct If {
    ifexpr: IfExpr,
    if_true: Stmts,
    if_false: Box<Stmts>,
}

// if (variable == some_value) { decls } else { decls }
impl Parse for If {
    fn parse(input: ParseStream) -> Result<Self> {
        // if (someting == whatever)
        let _: token::If = input.parse()?;
        let ifexpr: IfExpr = input.parse()?;

        // if true block.
        let if_true = braced!(input);
        let if_true: Stmts = if_true.parse()?;

        // else block is initiall empty.
        let mut if_false = Box::new(Stmts(Vec::new()));

        // see if there's an "else"
        if input.peek(Token![else]) {
            let _: token::Else = input.parse()?;
            // mus be followed either by "if" or a block.
            let lookahead = input.lookahead1();
            let stmts: Stmts = if lookahead.peek(token::Brace) {
                let if_false = braced!(input);
                if_false.parse()?
            } else if lookahead.peek(token::If) {
                input.parse()?
            } else {
                return Err(lookahead.error());
            };
            if_false = Box::new(stmts);
        }

        Ok(If{
            ifexpr,
            if_true,
            if_false,
        })
    }
}

// member declaration, or an "if" expresion.
#[derive(Debug)]
enum Stmt {
    VarDecl(VarDecl),
    If(If),
}

// member declaration, or an "if" expresion.
impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![if]) {
            let res: If = input.parse()?;
            Ok(Stmt::If(res))
        } else if lookahead.peek(Ident) {
            let mut decl: VarDecl = input.parse()?;
            input.parse::<Token![;]>()?;

            // We can have # optional; rust: RustType] _after_ the semicolon.
            if input.peek(Token![#]) {
                let c: InformativeComment = input.parse()?;
                if c.optional {
                    decl.optional = true;
                    if !decl.iso_type.contains("optional") {
                        decl.iso_type = format!("optional {}", decl.iso_type);
                    }
                }
                if let Some(rt) = c.rust_type {
                    decl.rust_type = rt;
                }
            }

            Ok(Stmt::VarDecl(decl))
        } else {
            Err(lookahead.error())
        }
    }
}

// Stmts is just a run of Stmt.
#[derive(Debug)]
struct Stmts(Vec<Stmt>);

// Stmts is just a run of Stmt.
impl Parse for Stmts {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut statements = Vec::new();
        while !input.is_empty() {
            let stmt: Stmt = input.parse()?;
            statements.push(stmt);
        }
        Ok(Stmts(statements))
    }
}

// The body of a class is just a run of statements.
#[derive(Debug)]
struct ClassBody {
    statements: Stmts,
}

// The body of a class is just a run of statements.
impl Parse for ClassBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let input = braced!(input);
        let statements: Stmts = input.parse()?;
        Ok(ClassBody {
            statements
        })
    }
}

// Complete class, header and body.
#[derive(Debug)]
struct Class {
    head:   ClassHeader,
    body:   ClassBody,
}

// Complete class, header and body.
impl Parse for Class {
    fn parse(input: ParseStream) -> Result<Self> {
        Ok(Class {
            head:   input.parse()?,
            body:   input.parse()?,
        })
    }
}

#[proc_macro]
pub fn def_box(input: TokenStream) -> TokenStream {
    let class = parse_macro_input!(input as Class);
    println!("item: \"{:#?}\"", class);
    TokenStream::new()
}

