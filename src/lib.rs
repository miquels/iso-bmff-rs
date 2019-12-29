extern crate proc_macro;

use proc_macro::TokenStream;
use syn::{Ident, Lit, LitInt, LitStr, Token, spanned::Spanned, Result, Error};
use syn::{parse_macro_input, token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

// Helper macros.
macro_rules! bracketed {
    ($input:expr) => {{ let content; syn::bracketed!(content in $input); content }}
}
macro_rules! braced {
    ($input:expr) => {{ let content; syn::braced!(content in $input); content }}
}
macro_rules! parenthesized {
    ($input:expr) => {{ let content; syn::parenthesized!(content in $input); content }}
}


// Parse a simple u32 integer.
fn parse_int(input: ParseStream) -> Result<u32> {
    let lit: Lit = input.parse()?;
    match lit {
        Lit::Int(lit) => lit.base10_parse::<u32>(),
        lit => Err(Error::new(lit.span(), "expected `number`")),
    }
}
    
#[derive(Debug)]
struct Aligned(u32);

// aligned(8)
impl Parse for Aligned {
    fn parse(input: ParseStream) -> Result<Self> {
        let content = parenthesized!(input);
        let value = parse_int(&content)?;
        Ok(Aligned(value))
    }
}

#[derive(Debug)]
enum Value {
    Number(u64),
    String(String),
    Variable(String),
}

// A value: number, string, or variable.
impl Parse for Value {
    fn parse(input: ParseStream) -> Result<Self> {

        let lookahead = input.lookahead1();
        if lookahead.peek(LitInt) {
            let val: LitInt = input.parse()?;
            return Ok(Value::Number(val.base10_parse::<u64>()?));
        }
        if lookahead.peek(LitStr) {
            let val: LitStr = input.parse()?;
            return Ok(Value::String(val.value()));
        }
        if lookahead.peek(Ident) {
            let val: Ident = input.parse()?;
            return Ok(Value::Variable(val.to_string()));
        }
        Err(lookahead.error())
    }
}

#[derive(Debug)]
struct VarDecl {
    optional:   bool,
    unsigned:   bool,
    typ:        Ident,
    size:       u32,
    array:      Option<u32>,
    name:       String,
    default:    Option<Value>,
}

// optional unsigned int(8)[16] extended_type
impl Parse for VarDecl {
    fn parse(input: ParseStream) -> Result<Self> {

        let mut optional = false;
        let mut unsigned = false;

        let typ = loop {
            if !input.peek(Ident) {
                return Err(input.error("expected `type`"));
            }
            let ident: Ident = input.parse()?;
            match ident.to_string().as_str() {
                "optional" => optional = true,
                "unsigned" => unsigned = true,
                "int" => break ident,
                _ => return Err(input.error("expected `type`")),
            }
        };

        // parentheses must follow.
        let content = parenthesized!(input);
        let size = parse_int(&content)?;

        // It might be an array, see if a bracket follows
        let mut array = None;
        if input.peek(token::Bracket) {
            let content = bracketed!(input);
            array = Some(parse_int(&content)?);
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
            let d: Value = input.parse()?;
            default = Some(d);
        }

        Ok(VarDecl{
            optional,
            unsigned,
            typ,
            size,
            array,
            name,
            default,
        })
    }
}

// A list of VarDecl's.
#[derive(Debug)]
struct VarDecls(Vec<VarDecl>);

impl Parse for VarDecls {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut decls = Vec::new();
        while !input.is_empty() {
            let decl: VarDecl = input.parse()?;
            input.parse::<Token![;]>()?;
            decls.push(decl);
            // The next token must be something that looks like another
            // variable declaration. If not, bail.
            if !input.peek(Ident) {
                break;
            }
        }
        Ok(VarDecls(decls))
    }
}

#[derive(Debug)]
struct ClassHeader {
    name:       Ident,
    aligned:    Option<u32>,
    args:      Vec<VarDecl>,
}

// the stuff before { ... }
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

        Ok(ClassHeader{
            name,
            aligned,
            args: args.into_iter().collect(),
        })
    }
}

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

// left == right, or left != right.
#[derive(Debug)]
struct IfExpr {
    left:   Value,
    right:  Value,
    op:     Op,
}

impl Parse for IfExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let input = parenthesized!(input);
        let left: Value = input.parse()?;
        let op: Op = input.parse()?;
        let right: Value = input.parse()?;
        Ok(IfExpr {
            left,
            op,
            right,
        })
    }
}

// if (variable == some_value) { decls } else { decls }
#[derive(Debug)]
struct If {
    ifexpr: IfExpr,
    left: Stmts,
    right: Box<Stmts>,
}

impl Parse for If {
    fn parse(input: ParseStream) -> Result<Self> {
        // if (someting == whatever)
        let _: token::If = input.parse()?;
        let ifexpr: IfExpr = input.parse()?;

        // if true block.
        let left = braced!(input);
        let left: Stmts = left.parse()?;

        // else block is initiall empty.
        let mut right = Box::new(Stmts(Vec::new()));

        // see if there's an "else"
        if input.peek(Token![else]) {
            let _: token::Else = input.parse()?;
            // parse whatever is after else as another Stmts block.
            let stmts_right: Stmts = input.parse()?;
            right = Box::new(stmts_right);
        }

        Ok(If{
            ifexpr,
            left,
            right,
        })
    }
}

// member declaration, or an "if" expresion.
#[derive(Debug)]
enum Stmt {
    VarDecl(VarDecl),
    If(If),
}

impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![if]) {
            let res: If = input.parse()?;
            Ok(Stmt::If(res))
        } else if lookahead.peek(Ident) {
            let res: VarDecl = input.parse()?;
            input.parse::<Token![;]>()?;
            Ok(Stmt::VarDecl(res))
        } else {
            Err(lookahead.error())
        }
    }
}

// Stmts is just a run of Stmt.
#[derive(Debug)]
struct Stmts(Vec<Stmt>);

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

#[derive(Debug)]
struct ClassBody {
    statements: Stmts,
}

impl Parse for ClassBody {
    fn parse(input: ParseStream) -> Result<Self> {
        let input = braced!(input);
        let statements: Stmts = input.parse()?;
        Ok(ClassBody {
            statements
        })
    }
}

#[derive(Debug)]
struct Class {
    head:   ClassHeader,
    body:   ClassBody,
}

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

