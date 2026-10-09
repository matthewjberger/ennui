use super::crates::{Owners, home_key};
use super::{Crate, Findings, clipped, layer_of, spanned};
use crate::audit::{read, rust_files};
use regex::Regex;
use std::collections::BTreeSet;
use syn::visit::Visit;
use syn::{
    Expr, ExprCall, ExprForLoop, ExprMethodCall, FnArg, ImplItem, ImplItemFn, Item, ItemFn,
    ItemTrait, Lit, Local, Member, Pat, Path, PathArguments, ReturnType, Signature, Stmt,
    TraitItem, Type, UnOp, Visibility,
};

const SYSTEM_PARAMS: [&str; 8] = [
    "Res<", "ResMut<", "View<", "Peek<", "Mut<", "Glance", "Later", "Edits",
];
const OUTSIDE_TRAIT_FUNCTIONS: [&str; 23] = [
    "drop",
    "deref_mut",
    "deref",
    "index_mut",
    "fmt",
    "hash",
    "eq",
    "cmp",
    "partial_cmp",
    "clone",
    "into_iter",
    "next",
    "write",
    "write_u64",
    "finish",
    "resumed",
    "window_event",
    "device_event",
    "about_to_wait",
    "exiting",
    "default",
    "from",
    "try_from",
];
const PLUMBING: [&str; 3] = [
    "crates/engine/ennui-ecs",
    "crates/engine/ennui-reflect",
    "crates/engine/ennui-reflect-derive",
];
const COMPUTING_CALLS: [&str; 11] = [
    "map", "filter", "fold", "iter", "sum", "min", "max", "sqrt", "powf", "sin", "cos",
];
const SINKS: [&str; 14] = [
    "spawn",
    "spawn_with",
    "attach",
    "set",
    "set_if_new",
    "despawn",
    "despawn_tree",
    "drop_slab",
    "crew_ask",
    "insert_clip",
    "remove_mesh",
    "remove_material",
    "remove_image",
    "send",
];
const SORTS: [&str; 6] = [
    "sort",
    "sort_unstable",
    "sort_by",
    "sort_by_key",
    "sort_unstable_by",
    "sort_unstable_by_key",
];
const WALK_CALLS: [&str; 9] = [
    "iter",
    "iter_mut",
    "keys",
    "values",
    "values_mut",
    "drain",
    "into_iter",
    "into_keys",
    "into_values",
];
const LOOKUPS: [&str; 9] = [
    "get",
    "get_mut",
    "contains",
    "contains_key",
    "len",
    "is_empty",
    "entry",
    "insert",
    "remove",
];
const REPEATED_CALLS: usize = 4;

pub(super) struct Visible<'ast>(pub(super) Vec<(&'ast Visibility, &'ast Signature)>);

impl<'ast> Visit<'ast> for Visible<'ast> {
    fn visit_item_fn(&mut self, item: &'ast ItemFn) {
        self.0.push((&item.vis, &item.sig));
        syn::visit::visit_item_fn(self, item);
    }

    fn visit_impl_item_fn(&mut self, item: &'ast ImplItemFn) {
        self.0.push((&item.vis, &item.sig));
        syn::visit::visit_impl_item_fn(self, item);
    }
}

struct Signatures<'ast>(Vec<&'ast Signature>);

impl<'ast> Visit<'ast> for Signatures<'ast> {
    fn visit_signature(&mut self, signature: &'ast Signature) {
        self.0.push(signature);
    }
}

struct Traits<'ast>(Vec<&'ast ItemTrait>);

impl<'ast> Visit<'ast> for Traits<'ast> {
    fn visit_item_trait(&mut self, item: &'ast ItemTrait) {
        self.0.push(item);
        syn::visit::visit_item_trait(self, item);
    }
}

struct Computing(bool);

impl<'ast> Visit<'ast> for Computing {
    fn visit_expr(&mut self, expr: &'ast Expr) {
        match expr {
            Expr::ForLoop(_) | Expr::While(_) | Expr::Loop(_) | Expr::If(_) | Expr::Match(_) => {
                self.0 = true;
            }
            Expr::MethodCall(call)
                if COMPUTING_CALLS.contains(&call.method.to_string().as_str()) =>
            {
                self.0 = true;
            }
            _ => {}
        }
        syn::visit::visit_expr(self, expr);
    }
}

struct Writes {
    sink: bool,
    sorted: bool,
}

impl<'ast> Visit<'ast> for Writes {
    fn visit_expr_call(&mut self, call: &'ast ExprCall) {
        if let Expr::Path(path) = &*call.func
            && path
                .path
                .segments
                .last()
                .is_some_and(|segment| SINKS.contains(&segment.ident.to_string().as_str()))
        {
            self.sink = true;
        }
        syn::visit::visit_expr_call(self, call);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        let method = call.method.to_string();
        if SINKS.contains(&method.as_str()) || (method == "remove" && call.turbofish.is_some()) {
            self.sink = true;
        }
        if SORTS.contains(&method.as_str()) {
            self.sorted = true;
        }
        syn::visit::visit_expr_method_call(self, call);
    }

    fn visit_path(&mut self, path: &'ast Path) {
        let count = path.segments.len();
        for (index, segment) in path.segments.iter().enumerate() {
            if (segment.ident == "later" && index + 1 < count)
                || (segment.ident == "remove"
                    && matches!(segment.arguments, PathArguments::AngleBracketed(_)))
            {
                self.sink = true;
            }
        }
        syn::visit::visit_path(self, path);
    }
}

fn hashed(ty: &Type) -> bool {
    match ty {
        Type::Reference(reference) => hashed(&reference.elem),
        Type::Path(path) => path
            .path
            .segments
            .last()
            .is_some_and(|segment| segment.ident == "HashMap" || segment.ident == "HashSet"),
        _ => false,
    }
}

struct Locals<'a>(&'a mut BTreeSet<String>);

impl<'ast> Visit<'ast> for Locals<'_> {
    fn visit_local(&mut self, local: &'ast Local) {
        let (name, typed) = match &local.pat {
            Pat::Ident(pat) => (Some(pat.ident.to_string()), false),
            Pat::Type(typed) => match &*typed.pat {
                Pat::Ident(pat) => (Some(pat.ident.to_string()), hashed(&typed.ty)),
                _ => (None, false),
            },
            _ => (None, false),
        };
        let built = local.init.as_ref().is_some_and(|init| {
            matches!(&*init.expr, Expr::Call(call) if matches!(&*call.func, Expr::Path(path)
                if path.path.segments.iter().rev().nth(1).is_some_and(|segment| segment.ident == "HashMap" || segment.ident == "HashSet")))
        });
        if let Some(name) = name
            && (typed || built)
        {
            self.0.insert(name);
        }
        syn::visit::visit_local(self, local);
    }
}

fn walked_in(
    mut expr: &Expr,
    fields: &BTreeSet<String>,
    locals: &BTreeSet<String>,
) -> Option<String> {
    let mut applied: Option<String> = None;
    loop {
        match expr {
            Expr::Reference(reference) => expr = &reference.expr,
            Expr::Paren(paren) => expr = &paren.expr,
            Expr::Unary(unary) if matches!(unary.op, UnOp::Deref(_)) => expr = &unary.expr,
            Expr::MethodCall(call) => {
                applied = Some(call.method.to_string());
                expr = &call.receiver;
            }
            Expr::Field(field) => {
                let Member::Named(ident) = &field.member else {
                    return None;
                };
                let name = ident.to_string();
                let looked_up = applied.is_some_and(|method| LOOKUPS.contains(&method.as_str()));
                return (fields.contains(&name) && !looked_up).then_some(name);
            }
            Expr::Path(path) => {
                let name = path.path.get_ident()?.to_string();
                let looked_up = applied.is_some_and(|method| LOOKUPS.contains(&method.as_str()));
                return (locals.contains(&name) && !looked_up).then_some(name);
            }
            _ => return None,
        }
    }
}

struct Walks<'a> {
    fields: &'a BTreeSet<String>,
    locals: &'a BTreeSet<String>,
    walked: BTreeSet<String>,
}

impl<'ast> Visit<'ast> for Walks<'_> {
    fn visit_expr_for_loop(&mut self, held: &'ast ExprForLoop) {
        if let Some(name) = walked_in(&held.expr, self.fields, self.locals) {
            self.walked.insert(name);
        }
        syn::visit::visit_expr_for_loop(self, held);
    }

    fn visit_expr_method_call(&mut self, call: &'ast ExprMethodCall) {
        if WALK_CALLS.contains(&call.method.to_string().as_str())
            && let Some(name) = walked_in(&call.receiver, self.fields, self.locals)
        {
            self.walked.insert(name);
        }
        syn::visit::visit_expr_method_call(self, call);
    }
}

fn plain(expr: &Expr) -> bool {
    match expr {
        Expr::Lit(literal) => !matches!(literal.lit, Lit::Str(_) | Lit::ByteStr(_) | Lit::Char(_)),
        Expr::Path(path) => path.path.segments.len() == 1,
        Expr::Field(field) => plain(&field.base),
        Expr::Reference(reference) => plain(&reference.expr),
        Expr::Unary(unary) => matches!(unary.op, UnOp::Deref(_)) && plain(&unary.expr),
        _ => false,
    }
}

pub(super) fn layers(
    crates: &[Crate],
    root: &std::path::Path,
    limit: usize,
    found: &mut Findings,
) -> Result<(), String> {
    let bundled = Regex::new(r"pub type (\w+)(?:<[^>]*>)?\s*=\s*\(\s*ResMut<").unwrap();
    let mut shared: Vec<String> = Vec::new();
    for path in rust_files(root) {
        let text = read(&path)?;
        shared.extend(bundled.captures_iter(&text).map(|held| held[1].to_string()));
    }
    for held in crates {
        let mut aliases = shared.clone();
        for source in &held.sources {
            let Some(file) = &source.parsed else {
                continue;
            };
            for item in &file.items {
                if let Item::Type(alias) = item
                    && SYSTEM_PARAMS
                        .iter()
                        .any(|param| spanned(&source.text, &alias.ty).starts_with(param))
                {
                    aliases.push(alias.ident.to_string());
                }
            }
        }
        for source in &held.sources {
            let parts: Vec<&str> = source.name.split('/').collect();
            let last = parts.last().copied().unwrap_or("");
            let systems = parts.contains(&"systems") || last == "systems.rs";
            let pass = last == "pass.rs" || parts[..parts.len() - 1].contains(&"pass");
            let Some(file) = &source.parsed else {
                continue;
            };
            if !systems || pass {
                continue;
            }
            for item in &file.items {
                let line = match item {
                    Item::Struct(held) => held.struct_token.span.start().line,
                    Item::Enum(held) => held.enum_token.span.start().line,
                    Item::Const(held) => held.const_token.span.start().line,
                    Item::Static(held) => held.static_token.span.start().line,
                    Item::Type(held) if !aliases.contains(&held.ident.to_string()) => {
                        held.type_token.span.start().line
                    }
                    Item::Trait(held) => held.trait_token.span.start().line,
                    Item::Impl(held) => held.impl_token.span.start().line,
                    Item::Fn(function) => {
                        let line = function.sig.fn_token.span.start().line;
                        let foreign: Vec<String> = function
                            .sig
                            .inputs
                            .iter()
                            .filter_map(|input| match input {
                                FnArg::Typed(typed) => Some(spanned(&source.text, &typed.ty)),
                                FnArg::Receiver(_) => None,
                            })
                            .filter(|ty| {
                                !SYSTEM_PARAMS.iter().any(|param| ty.starts_with(param))
                                    && !aliases.iter().any(|alias| ty.starts_with(alias.as_str()))
                            })
                            .collect();
                        if !foreign.is_empty() {
                            found.entry("layers").or_default().push(format!(
                                "{}:{line}: {} takes {} (helpers go in commands or queries)",
                                source.name,
                                function.sig.ident,
                                clipped(&foreign.join(", "), 60)
                            ));
                        }
                        let closed = function.block.brace_token.span.close().end().line;
                        let length = closed + 1 - line;
                        if length > limit {
                            found.entry("long").or_default().push(format!(
                                "{}:{line}: {} is {length} lines",
                                source.name, function.sig.ident
                            ));
                        }
                        continue;
                    }
                    _ => continue,
                };
                let text = source.text.split('\n').nth(line - 1).unwrap_or("").trim();
                found.entry("layers").or_default().push(format!(
                    "{}:{line}: {} (types go in data, components or resources)",
                    source.name,
                    clipped(text, 70)
                ));
            }
        }
    }
    Ok(())
}

pub(super) fn logic(crates: &[Crate], found: &mut Findings) {
    for held in crates {
        for source in &held.sources {
            if !matches!(layer_of(&source.name), "resources" | "data") {
                continue;
            }
            let Some(file) = &source.parsed else {
                continue;
            };
            for item in &file.items {
                match item {
                    Item::Fn(function) if function.sig.constness.is_none() => {
                        let mut computing = Computing(false);
                        computing.visit_block(&function.block);
                        if computing.0 {
                            found.entry("logic").or_default().push(format!(
                                "{}:{}: fn {} (logic goes in commands or queries; a const fn that builds a table can stay)",
                                source.name,
                                function.sig.fn_token.span.start().line,
                                function.sig.ident
                            ));
                        }
                    }
                    Item::Impl(block) if block.trait_.is_none() => {
                        for method in &block.items {
                            let ImplItem::Fn(method) = method else {
                                continue;
                            };
                            if method.sig.constness.is_some() {
                                continue;
                            }
                            let mut computing = Computing(false);
                            computing.visit_block(&method.block);
                            let builds = match method.sig.receiver() {
                                Some(receiver) => !matches!(&*receiver.ty, Type::Reference(_)),
                                None => matches!(&method.sig.output, ReturnType::Type(_, ty)
                                    if matches!(&**ty, Type::Path(path) if path.path.segments.first().is_some_and(|segment| segment.ident == "Self"))),
                            };
                            if computing.0 && !builds {
                                found.entry("logic").or_default().push(format!(
                                    "{}:{}: method {} (a free function in commands or queries; constructors and by-value builders can stay)",
                                    source.name,
                                    method.sig.fn_token.span.start().line,
                                    method.sig.ident
                                ));
                            }
                        }
                    }
                    _ => {}
                }
            }
        }
    }
}

pub(super) fn self_methods(crates: &[Crate], found: &mut Findings) {
    for held in crates {
        if PLUMBING.contains(&held.place.as_str()) {
            continue;
        }
        for source in &held.sources {
            let Some(file) = &source.parsed else {
                continue;
            };
            let mut signatures = Signatures(Vec::new());
            signatures.visit_file(file);
            for signature in signatures.0 {
                if OUTSIDE_TRAIT_FUNCTIONS.contains(&signature.ident.to_string().as_str()) {
                    continue;
                }
                let line = signature.fn_token.span.start().line;
                let text = source.text.split('\n').nth(line - 1).unwrap_or("").trim();
                let place = format!("{}:{line}: {}", source.name, clipped(text, 80));
                let receiver = signature.receiver();
                let mutating = receiver.is_some_and(|receiver| {
                    matches!(&*receiver.ty, Type::Reference(reference) if reference.mutability.is_some())
                });
                let handed = signature.inputs.iter().any(|input| {
                    matches!(input, FnArg::Typed(typed) if matches!(&*typed.ty, Type::Reference(reference)
                        if matches!(&*reference.elem, Type::Path(path) if path.path.is_ident("Self"))))
                });
                let returns_self = matches!(&signature.output, ReturnType::Type(_, ty)
                    if matches!(&**ty, Type::Path(path) if path.path.segments.first().is_some_and(|segment| segment.ident == "Self")));
                let consumed = receiver
                    .is_some_and(|receiver| !matches!(&*receiver.ty, Type::Reference(_)))
                    && !returns_self;
                if mutating {
                    found
                        .entry("self_methods")
                        .or_default()
                        .push(format!("{place} (use a free function over the data)"));
                } else if handed {
                    found.entry("self_methods").or_default().push(format!(
                        "{place} (a Self parameter is a method in disguise; make it plain data processed by a free function)"
                    ));
                } else if consumed {
                    found.entry("self_methods").or_default().push(format!(
                        "{place} (self by value that does not build Self is a method; use a free function)"
                    ));
                }
            }
        }
    }
}

pub(super) fn traits(crates: &[Crate], found: &mut Findings) {
    let typed = Regex::new(r"\b(self|Self)\b").unwrap();
    for held in crates {
        if PLUMBING.contains(&held.place.as_str()) {
            continue;
        }
        for source in &held.sources {
            let Some(file) = &source.parsed else {
                continue;
            };
            let mut traits = Traits(Vec::new());
            traits.visit_file(file);
            for item in traits.0 {
                let on_self = item.items.iter().any(|held| {
                    matches!(held, TraitItem::Fn(function) if typed.is_match(&spanned(&source.text, &function.sig)))
                });
                if on_self {
                    found.entry("traits").or_default().push(format!(
                        "{}:{}: trait {} has functions on self or Self (behaviour chosen by type; store it as plain rows grouped by kind and process each kind with one known function, never a pointer per item)",
                        source.name,
                        item.trait_token.span.start().line,
                        item.ident
                    ));
                }
            }
        }
    }
}

pub(super) fn one_liners(crates: &[Crate], owners: &Owners, found: &mut Findings) {
    for held in crates {
        let home = home_key(&held.folder);
        for source in &held.sources {
            let last = source.name.rsplit('/').next().unwrap_or("");
            if matches!(last, "main.rs" | "lib.rs" | "plugin.rs") {
                continue;
            }
            let Some(file) = &source.parsed else {
                continue;
            };
            for item in &file.items {
                let Item::Fn(function) = item else {
                    continue;
                };
                let opened = function.block.brace_token.span.open().start().line;
                let closed = function.block.brace_token.span.close().end().line;
                let body = source
                    .text
                    .split('\n')
                    .enumerate()
                    .filter(|(index, line)| {
                        index + 1 > opened && index + 1 < closed && !line.trim().is_empty()
                    })
                    .count();
                if body > 1 {
                    continue;
                }
                let name = function.sig.ident.to_string();
                let shared = matches!(function.vis, Visibility::Public(_))
                    && owners.words.get(&name).is_some_and(|homes| {
                        homes.iter().any(|&index| owners.homes[index] != home)
                    });
                let forward = body == 0
                    || match function.block.stmts.as_slice() {
                        [Stmt::Expr(Expr::Call(call), _)] => {
                            matches!(&*call.func, Expr::Path(_)) && call.args.iter().all(plain)
                        }
                        [Stmt::Expr(Expr::MethodCall(call), _)] => {
                            plain(&call.receiver) && call.args.iter().all(plain)
                        }
                        _ => false,
                    };
                let call =
                    Regex::new(&format!(r"\b{}\s*(?:::<[^>]*>)?\(", regex::escape(&name))).unwrap();
                let calls = call
                    .find_iter(&owners.every)
                    .filter(|found| !owners.every[..found.start()].ends_with("fn "))
                    .count();
                if !shared && (forward || calls < REPEATED_CALLS) {
                    found.entry("one_liners").or_default().push(format!(
                        "{}:{}: {name} (write its body where it is called, then delete it)",
                        source.name,
                        function.sig.fn_token.span.start().line
                    ));
                }
            }
        }
    }
}

pub(super) fn storage(crates: &[Crate], found: &mut Findings) {
    let rows = Regex::new(r"\bRows\b").unwrap();
    for held in crates {
        if matches!(
            held.place.as_str(),
            "crates/engine/ennui-ecs" | "crates/engine/ennui"
        ) {
            continue;
        }
        for source in &held.sources {
            for (index, line) in source.text.split('\n').enumerate() {
                if rows.is_match(line) {
                    found.entry("storage").or_default().push(format!(
                        "{}:{}: Rows runs the system alone; take View, Peek, Mut, Glance, Later or Edits",
                        source.name,
                        index + 1
                    ));
                }
            }
            let Some(file) = &source.parsed else {
                continue;
            };
            let mut visible = Visible(Vec::new());
            visible.visit_file(file);
            for (vis, signature) in visible.0 {
                let handed = signature.inputs.iter().any(|input| {
                    matches!(input, FnArg::Typed(typed) if matches!(&*typed.ty, Type::Reference(reference)
                        if reference.mutability.is_some()
                            && matches!(&*reference.elem, Type::Path(path) if path.path.segments.last().is_some_and(|segment| segment.ident == "Storage"))))
                });
                if !matches!(vis, Visibility::Inherited) && handed {
                    found.entry("storage").or_default().push(format!(
                        "{}:{}: a public helper takes &mut Storage; take Later or Edits and typed reads",
                        source.name,
                        signature.fn_token.span.start().line
                    ));
                }
            }
        }
    }
}

pub(super) fn hash_walks(crates: &[Crate], found: &mut Findings) {
    for held in crates {
        let mut fields: BTreeSet<String> = BTreeSet::new();
        for source in &held.sources {
            let Some(file) = &source.parsed else {
                continue;
            };
            for item in &file.items {
                if let Item::Struct(held) = item
                    && let syn::Fields::Named(named) = &held.fields
                {
                    for field in &named.named {
                        if hashed(&field.ty)
                            && let Some(ident) = &field.ident
                        {
                            fields.insert(ident.to_string());
                        }
                    }
                }
            }
        }
        for source in &held.sources {
            let Some(file) = &source.parsed else {
                continue;
            };
            for item in &file.items {
                let Item::Fn(function) = item else {
                    continue;
                };
                let mut writes = Writes {
                    sink: false,
                    sorted: false,
                };
                writes.visit_block(&function.block);
                if !writes.sink || writes.sorted {
                    continue;
                }
                let mut locals = BTreeSet::new();
                for input in &function.sig.inputs {
                    if let FnArg::Typed(typed) = input
                        && hashed(&typed.ty)
                        && let Pat::Ident(pat) = &*typed.pat
                    {
                        locals.insert(pat.ident.to_string());
                    }
                }
                Locals(&mut locals).visit_block(&function.block);
                let mut walks = Walks {
                    fields: &fields,
                    locals: &locals,
                    walked: BTreeSet::new(),
                };
                walks.visit_block(&function.block);
                if !walks.walked.is_empty() {
                    found.entry("hash_walks").or_default().push(format!(
                        "{}:{}: fn {} walks {} and writes (hash order changes each run; walk a BTreeMap or sort first)",
                        source.name,
                        function.sig.fn_token.span.start().line,
                        function.sig.ident,
                        walks.walked.iter().cloned().collect::<Vec<_>>().join(", ")
                    ));
                }
            }
        }
    }
}
