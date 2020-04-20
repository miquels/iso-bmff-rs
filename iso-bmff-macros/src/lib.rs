extern crate proc_macro;

use proc_macro::TokenStream;
use proc_macro2::{Span, TokenTree};
use syn::{Ident, Token, spanned::Spanned, Result, Error};
use syn::{parse_macro_input, parenthesized, bracketed, braced, token};
use syn::parse::{Parse, ParseStream};
use syn::punctuated::Punctuated;

// Keywords that we reckognize and use in this module.
// Note, these are not all the keywords; some of them are
// interpreted as Idents an parsed on-the-fly.
mod kw {
    syn::custom_keyword!(rust_type);
    syn::custom_keyword!(optional);
    syn::custom_keyword!(extends);
    syn::custom_keyword!(template);
    syn::custom_keyword!(aligned);
    syn::custom_keyword!(unsigned);
    syn::custom_keyword!(signed);
    syn::custom_keyword!(int);
    syn::custom_keyword!(uint);
    syn::custom_keyword!(bit);
    syn::custom_keyword!(class);
}

mod expr;

use expr::{BinOp, Expr};

// Eat the rest of the input.
fn eat(input: ParseStream, end: Option<char>) -> Result<()> {
    input.step(|cursor| {
        let mut rest = *cursor;
        while let Some((tt, next)) = rest.token_tree() {
            match &tt {
                TokenTree::Punct(punct) if Some(punct.as_char()) == end => {
                    return Ok(((), rest));
                }
                _ => rest = next,
            }
        }
        return Ok(((), rest));
    })
}

// aligned(8)
#[derive(Debug)]
struct Aligned {
    span:   Span,
    value:  u32,
}

// align attribute: aligned(8)
impl Parse for Aligned {
    fn parse(input: ParseStream) -> Result<Self> {
        input.parse::<kw::aligned>()?;
        let content;
        let parens = parenthesized!(content in input);
        let val: Expr = content.parse()?;
        let value = val.get_lit_int()?;
        Ok(Aligned{ span: parens.span, value })
    }
}

// [template] unsigned int(8)[16] [= default_value]
#[derive(Debug)]
struct VarDecl {
    optional:   bool,
    template:   bool,
    iso_type:   String,
    rust_type:  String,
    size:       u32,
    array:      Option<Expr>,
    name:       String,
    default:    Option<Expr>,
}

// [template] unsigned int(8)[16] [= default_value]
impl Parse for VarDecl {
    fn parse(input: ParseStream) -> Result<Self> {

        let mut template = false;
        let mut unsigned = false;
        let mut signed = false;
        let mut is_const = false;

        if input.peek(kw::template) {
            input.parse::<kw::template>()?;
            template = true;
        }

        if input.peek(token::Const) {
            input.parse::<token::Const>()?;
            is_const = true;
        }

        let typ = if input.peek(kw::signed) || input.peek(kw::unsigned) {
            if input.peek(kw::signed) {
                input.parse::<kw::signed>()?;
                signed = true;
            } else if input.peek(kw::unsigned) {
                input.parse::<kw::unsigned>()?;
                unsigned = true;
            }
            input.parse::<kw::int>()?;
            "int".to_string()
        } else {
            if input.peek(kw::int) {
                input.parse::<kw::int>()?;
                "int".to_string()
            } else if input.peek(kw::uint) {
                input.parse::<kw::uint>()?;
                "uint".to_string()
            } else if input.peek(kw::bit) {
                input.parse::<kw::bit>()?;
                "bit".to_string()
            } else if input.peek(kw::class) {
                input.parse::<kw::class>()?;
                "class".to_string()
            } else if input.peek(Ident) {
                println!("XXX 1");
                let name = input.parse::<Ident>()?;
                println!("XXX 2");
                name.to_string()
            } else {
                return Err(input.error("expected `type`"));
            }
        };

        // Translate typ to iso_type.
        let mut iso_type = String::new();
        if is_const {
            iso_type.push_str("const ");
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
        iso_type.push_str(&typ);
        let rust_type;
        let mut size = 0;

        if typ == "int" || typ == "uint" || typ == "bit" {
            // parentheses must follow.
            let content;
            parenthesized!(content in input);
            let size_expr: Expr = content.parse()?;
            size = size_expr.get_lit_int()?;
            let span = size_expr.span();

            // Translate to a rust type.
            rust_type = match (typ.to_string().as_str(), signed, size) {
                ("int", true, 1..=8) => "i8",
                ("int", true, 9..=16) => "i16",
                ("int", true, 17..=32) => "i32",
                ("int", true, 33..=64) => "i64",
                ("int", false, 1..=8) => "u8",
                ("int", false, 9..=16) => "u16",
                ("int", false, 17..=32) => "u32",
                ("int", false, 33..=64) => "u64",
                ("int", _, size) => return Err(Error::new(span, format!("unsupported int({})", size))),
                ("bit", _, 1) => "bool",
                ("bit", _, 2..=8) => "u8",
                ("bit", _, 9..=16) => "u16",
                ("bit", _, 17..=23) => "u32",
                ("bit", _, 24) => "Flags",
                ("bit", _, 25..=32) => "u32",
                ("bit", _, 33..=64) => "u64",
                ("bit", _, size) => return Err(Error::new(span, format!("unsupported bit({})", size))),
                _ => return Err(Error::new(typ.span(), "expected `type`")),
            }.to_string();
        } else {
            // Other types here. XXX FIXME check against symbol table.
            rust_type = typ.to_string();

            // If it's a valid class (check!) then empty () may follow.
            if input.peek(token::Paren) {
                let content;
                parenthesized!(content in input);
                eat(&content, None)?;
            }
        }

        // It might be an array, see if a bracket follows
        // empty [] is valid, and means "to end of box".
        let mut array = None;
        if input.peek(token::Bracket) {
            let content;
            let span = bracketed!(content in input).span;
            if content.is_empty() {
                array = Some(Expr::new_lit_int(0, span));
            } else {
                let e: Expr = content.parse()?;
                array = Some(e);
            }
        }
        println!("XXX 2.1");

        // This allows simple "ChannelLayout();" or "DownMixInstructions() []" with no name.
        // XXX not actually correct, we allow stuff like template ChannelLayout(); etc.
        if input.peek(Token![;]) {
            return Ok(VarDecl{
                optional: false,
                template: false,
                iso_type: iso_type.clone(),
                rust_type: "".to_string(),
                size,
                array,
                name: iso_type,
                default: None,
            });
        }

        // Then the name of the variable.
        // This might be a rust keyword ...
        let name = if input.peek(token::Type) {
            let _ = input.parse::<Token![type]>()?;
            "type".to_string()
        } else {
            input.parse::<Ident>()?.to_string()
        };
        println!("XXX 2.2");

        // *another* place for the array indicaton, sheesh
        if input.peek(token::Bracket) {
            let content;
            let span = bracketed!(content in input).span;
            if content.is_empty() {
                array = Some(Expr::new_lit_int(0, span));
            } else {
                let e: Expr = content.parse()?;
                array = Some(e);
            }
        }

        // An equals sign might be ahead.
        let mut default = None;
        if input.peek(Token![=]) {
            let _eq: Token![=] = input.parse()?;
            // for now, we just ignore everything in a block.
            if input.peek(token::Brace) {
                let content;
                let span = braced!(content in input).span;
                eprintln!("warning: ignoring default {{ ... }} block");
                eprintln!(" --> [source file]:{}:{}", span.start().line, span.start().column);
                let w = span.start().line.to_string().len();
                eprintln!("{:.*} |", w, "           ");
                eprintln!("{:<.*} |", w, span.start().line);
                eprintln!("{:.*} |", w, "           ");
                if span.start().line == 0 {
                    eprintln!("(imprecise message because of current compiler / proc-macro restrictions)");
                }
                eat(&content, None)?;
                // ... and the rest up to ';' as well.
                eat(&input, Some(';'))?;
            } else {
                let e: Expr = input.parse()?;
                default = Some(e);
            }
        }

        println!("XXX 3");
        Ok(VarDecl{
            optional: false,
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
        let content;
        parenthesized!(content in input);
        let args: Punctuated<ExtendsArg, Token![,]> = content.parse_terminated(ExtendsArg::parse)?;
        Ok(Extends {
            class:  class.to_string(),
            args:   args.into_iter().collect(),
        })
    }
}

// arguments for extends Box(arg, arg ..)
#[derive(Debug)]
struct ExtendsArg {
    varname:    Option<Ident>,
    value:      Option<Expr>,
}

// arguments for extends Box(arg, arg ..)
impl Parse for ExtendsArg {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut varname = None;
        let value;
        let left: Expr = input.parse()?;
        match left {
            e@Expr::LitInt{..} => value = Some(e),
            e@Expr::LitStr{..} => value = Some(e),
            Expr::Variable(name) => {
                varname = Some(name);
                match input.parse::<token::Eq>() {
                    Ok(_) => {
                        let right: Expr = input.parse()?;
                        match right {
                            e@Expr::LitInt{..} => value = Some(e),
                            e@Expr::LitStr{..} => value = Some(e),
                            e@Expr::Variable(_) => value = Some(e),
                            other => return Err(Error::new(other.span(), "unexpected")),
                        }
                    },
                    Err(_) => value = None,
                }
            }
            other => return Err(Error::new(other.span(), "expected expression")),
        }
        if !input.peek(Token![,]) && !input.is_empty() {
            return Err(input.error("unexpected"));
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
    aligned:    Option<Aligned>,
    is_abstract:    bool,
    name:       Ident,
    args:       Vec<VarDecl>,
    extends:    Option<Extends>,
}

// Header of a class, i.e. the definitions before the body.
impl Parse for ClassHeader {
    fn parse(input: ParseStream) -> Result<Self> {

        // aligned(n)
        let mut aligned = None;
        if input.peek(kw::aligned) {
            aligned = Some(Aligned::parse(input)?);
        }

        // abstract
        let mut is_abstract = false;
        if input.peek(token::Abstract) {
            input.parse::<token::Abstract>()?;
            is_abstract = true;
        }

        // "class"
        input.parse::<kw::class>()?;

        // "name"
        let name: Ident = input.parse()?;

        // args
        let mut args = Vec::new();
        if input.peek(token::Paren) {
            let content;
            parenthesized!(content in input);
            let a: Punctuated<VarDecl, Token![,]> = content.parse_terminated(VarDecl::parse)?;
            args = a.into_iter().collect();
        }

        let mut extends: Option<Extends> = None;
        if input.peek(kw::extends) {
            extends = Some(input.parse()?);
        }

        Ok(ClassHeader{
            aligned,
            is_abstract,
            name,
            args,
            extends,
        })
    }
}
/*
// Operators.
#[derive(Debug)]
enum Op {
    EqEq,
    Ne,
    Lt,
    Le,
    And,
}

// Parse an operator. "==" or "!=", etc
impl Parse for Op {
    fn parse(input: ParseStream) -> Result<Self> {
        let lookahead = input.lookahead1();
        if lookahead.peek(Token![==]) {
            let _: token::EqEq = input.parse()?;
            Ok(Op::EqEq)
        } else if lookahead.peek(Token![!=]) {
            let _: token::Ne = input.parse()?;
            Ok(Op::Ne)
        } else if lookahead.peek(Token![<]) {
            let _: token::Lt = input.parse()?;
            Ok(Op::Lt)
        } else if lookahead.peek(Token![<=]) {
            let _: token::Le = input.parse()?;
            Ok(Op::Le)
        } else if lookahead.peek(Token![&]) {
            let _: token::And = input.parse()?;
            Ok(Op::And)
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
    op:     BinOp,
}

// `if` expression. left == right, or left != right.
impl Parse for IfExpr {
    fn parse(input: ParseStream) -> Result<Self> {
        let content;
        parenthesized!(content in input);
        let left: ValueToken = content.parse()?;
        let op = BinOp::parse(&content)?;
        let right: ValueToken = content.parse()?;
        if !content.is_empty() {
            return Err(input.error("expected `)`"));
        }
        Ok(IfExpr {
            left: left.value,
            op,
            right: right.value,
        })
    }
}
*/

// if (variable == some_value) { decls }
#[derive(Debug)]
struct If {
    ifexpr: Expr,
    if_true: Stmts,
}

// if (variable == some_value) { decls } else { decls }
impl Parse for If {
    fn parse(input: ParseStream) -> Result<Self> {
        // if (someting == whatever)
        let _: token::If = input.parse()?;
        let content;
        parenthesized!(content in input);
        let ifexpr: Expr = content.parse()?;

        // if true block.
        let if_true;
        braced!(if_true in input);
        let if_true: Stmts = if_true.parse()?;

        Ok(If {
            ifexpr,
            if_true,
        })
    }
}

// if (variable == some_value) { decls } else { decls }
#[derive(Debug)]
struct IfElse {
    ifexpr: Expr,
    if_true: Stmts,
    if_else: Vec<If>,
    if_false: Stmts,
}

// if (variable == some_value) { decls } else { decls }
impl Parse for IfElse {
    fn parse(input: ParseStream) -> Result<Self> {
        // if (something == whatever)
        let if_stmt: If = input.parse()?;

        // if_else block is initiall empty.
        let mut if_else = Vec::new();

        // else block is initiall empty.
        let mut if_false = Stmts(Vec::new());

        // see if there's an "else"
        while input.peek(Token![else]) {
            let _: token::Else = input.parse()?;
            let lookahead = input.lookahead1();

            // followed by another if?
            if lookahead.peek(token::If) {
                let next_if: If = input.parse()?;
                if_else.push(next_if);
                continue;
            }

            // nope, must be a block.
            if lookahead.peek(token::Brace) {
                let content;
                braced!(content in input);
                if_false = content.parse::<Stmts>()?;
            } else {
                return Err(lookahead.error());
            }
            break;
        }

        Ok(IfElse{
            ifexpr: if_stmt.ifexpr,
            if_true: if_stmt.if_true,
            if_else,
            if_false,
        })
    }
}

// for(i = 0; i < whatever; i++)
#[derive(Debug)]
struct For {
    start:  u32,
    op:     Option<BinOp>,
    end:    Option<Expr>,
    statements: Stmts,
}

// for(i = 0; i < whatever; i++)
impl Parse for For {
    fn parse(input: ParseStream) -> Result<Self> {
        // Parse "for"
        input.parse::<Token![for]>()?;

        // Args between parens
        let inner;
        parenthesized!(inner in input);

        // [int] i = 0;
        if inner.peek(kw::int) {
            inner.parse::<kw::int>()?;
        }
        let varname: Ident = inner.parse()?;
        inner.parse::<Token![=]>()?;
        let expr: Expr = inner.parse()?;
        let start = expr.get_lit_int()?;
        inner.parse::<Token![;]>()?;

        // i < something. may be empty!
        let mut end = None;
        let mut op = None;
        if !inner.peek(Token![;]) {
            // variable
            let v: Ident = inner.parse()?;
            if v != varname {
                return Err(Error::new(v.span(), format!("expected `{}`", varname)));
            }
            op = match BinOp::parse(&inner)? {
                op@BinOp::Le(_) => Some(op),
                op@BinOp::Lt(_) => Some(op),
                other => return Err(Error::new(other.span(), "unsupported operator")),
            };
            // end value
            let e: Expr = inner.parse()?;
            end = Some(e);
        }
        inner.parse::<Token![;]>()?;

        // i++
        let v: Ident = inner.parse()?;
        if v != varname {
            return Err(Error::new(v.span(), format!("expected `{}`", varname)));
        }
        if !inner.peek(Token![+]) || !inner.peek2(Token![+]) {
            return Err(inner.error("expected `++`"));
        }
        inner.parse::<Token![+]>()?;
        inner.parse::<Token![+]>()?;
        if !inner.is_empty() {
            return Err(inner.error("expected `)`"));
        }

        let inner;
        braced!(inner in input);
        let statements: Stmts = inner.parse()?;

        Ok(For {
            start,
            op,
            end,
            statements,
        })
    }
}

// member declaration, or an "if" expresion.
#[derive(Debug)]
enum Stmt {
    VarDecl(VarDecl),
    If(IfElse),
    For(For),
}

// member declaration, or an "if" expresion.
impl Parse for Stmt {
    fn parse(input: ParseStream) -> Result<Self> {

        // silently drop "int i, j;" declarations.
        while input.peek(kw::int) && input.peek2(Ident) {
            input.parse::<kw::int>()?;
            loop {
                input.parse::<Ident>()?;
                if !input.peek(Token![,]) {
                    break;
                } 
                input.parse::<Token![,]>()?;
            }
            input.parse::<Token![;]>()?;
        }

        let lookahead = input.lookahead1();
        if lookahead.peek(Token![if]) {
            let res: IfElse = input.parse()?;
            Ok(Stmt::If(res))
        } else if lookahead.peek(Token![for]) {
            let res: For = input.parse()?;
            Ok(Stmt::For(res))
        } else if lookahead.peek(Ident) || lookahead.peek(token::Const) {
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
        let inner;
        braced!(inner in input);
        let statements: Stmts = inner.parse()?;
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

