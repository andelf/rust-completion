// THis is copied from rustdoc

// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Rust AST Visitor. Extracts useful information and massages it into a form
//! usable for clean

use syntax::abi;
use syntax::ast;
use syntax::ast_util;
use syntax::ast_map;
use syntax::attr::AttrMetaMethods;
use syntax::codemap::Span;
use syntax::crateid::CrateId;
use syntax::parse::token;

use rustdoc::core;
use rustdoc::doctree::*;

use utils::*;

pub struct RustdocVisitor<'a> {
    pub module: Module,
    pub attrs: Vec<ast::Attribute>,
    pub cx: &'a core::DocContext,
    pub analysis: Option<&'a core::CrateAnalysis>,
    pub names: Vec<String>,
    current_prefix: String,
}

impl<'a> RustdocVisitor<'a> {
    pub fn new<'b>(cx: &'b core::DocContext,
                   analysis: Option<&'b core::CrateAnalysis>) -> RustdocVisitor<'b> {
        RustdocVisitor {
            module: Module::new(None),
            attrs: Vec::new(),
            cx: cx,
            analysis: analysis,
            names: Vec::new(),
            current_prefix: String::new(),
        }
    }

    pub fn visit(&mut self, krate: &ast::Crate) {
        let mut crate_id = String::new();
        self.attrs = krate.attrs.iter().map(|x| (*x).clone()).collect();
        for attr in self.attrs.iter() {
            match attr.node.value.node {
                ast::MetaNameValue(ref name, ref val) => {
                    if name.get() == "crate_id" {
                        match val.node {
                            ast::LitStr(ref crateid, _) => {
                                let cid: CrateId = from_str(crateid.get()).unwrap();
                                crate_id.push_str(cid.name.as_slice());
                            }
                            _ => {}
                        }
                    }

                }
                _ => {}
            }
        }

        self.current_prefix = crate_id;
        self.module = self.visit_mod_contents(krate.span,
                                              krate.attrs
                                                   .iter()
                                                   .map(|x| *x)
                                                   .collect(),
                                              ast::Public,
                                              ast::CRATE_NODE_ID,
                                              &krate.module,
                                              None);
        self.module.is_crate = true;
    }

    pub fn visit_struct_def(&mut self, item: &ast::Item, sd: @ast::StructDef,
                            generics: &ast::Generics) -> Struct {
        let struct_type = struct_type_from_def(sd);
        println!("!s {}::{}", self.current_prefix, token::get_ident(item.ident));
        Struct {
            id: item.id,
            struct_type: struct_type,
            name: item.ident,
            vis: item.vis,
            attrs: item.attrs.iter().map(|x| *x).collect(),
            generics: generics.clone(),
            fields: sd.fields.iter().map(|x| (*x).clone()).collect(),
            where: item.span
        }
    }

    pub fn visit_enum_def(&mut self, it: &ast::Item, def: &ast::EnumDef,
                          params: &ast::Generics) -> Enum {
        let mut vars: Vec<Variant> = Vec::new();
        for x in def.variants.iter() {
            println!("!e|{}::{}", self.current_prefix, token::get_ident(x.node.name));
            vars.push(Variant {
                name: x.node.name,
                attrs: x.node.attrs.iter().map(|x| *x).collect(),
                vis: x.node.vis,
                id: x.node.id,
                kind: x.node.kind.clone(),
                where: x.span,
            });
        }
        Enum {
            name: it.ident,
            variants: vars,
            vis: it.vis,
            generics: params.clone(),
            attrs: it.attrs.iter().map(|x| *x).collect(),
            id: it.id,
            where: it.span,
        }
    }

    pub fn visit_fn(&mut self, item: &ast::Item, fd: &ast::FnDecl,
                    fn_style: &ast::FnStyle, _abi: &abi::Abi,
                    gen: &ast::Generics) -> Function {
        Function {
            id: item.id,
            vis: item.vis,
            attrs: item.attrs.iter().map(|x| *x).collect(),
            decl: fd.clone(),
            name: item.ident,
            where: item.span,
            generics: gen.clone(),
            fn_style: *fn_style,
        }
    }

    pub fn visit_mod_contents(&mut self, span: Span, attrs: Vec<ast::Attribute> ,
                              vis: ast::Visibility, id: ast::NodeId,
                              m: &ast::Mod,
                              name: Option<ast::Ident>) -> Module {
        let mut om = Module::new(name);
        for item in m.view_items.iter() {
            self.visit_view_item(item, &mut om);
        }
        om.where_outer = span;
        om.where_inner = m.inner;
        om.attrs = attrs;
        om.vis = vis;
        om.id = id;
        for i in m.items.iter() {
            self.visit_item(*i, &mut om);
        }
        om
    }

    pub fn visit_view_item(&mut self, item: &ast::ViewItem, om: &mut Module) {
        if item.vis != ast::Public {
            return om.view_items.push(item.clone());
        }
        //println!("fuck to push. {:?}, item", item);
        println!("// visitor vi => {}", view_item_to_str(item));
        let please_inline = item.attrs.iter().any(|item| {
            match item.meta_item_list() {
                Some(list) => {
                    list.iter().any(|i| i.name().get() == "inline")
                }
                None => false,
            }
        });
        let item = match item.node {
            ast::ViewItemUse(ref vpath) => {
                match self.visit_view_path(*vpath, om, please_inline) {
                    None => return,
                    Some(path) => {
                        //println!("use path => {:?}", path);
                        ast::ViewItem {
                            node: ast::ViewItemUse(path),
                            .. item.clone()
                        }
                    }
                }
            }
            ast::ViewItemExternCrate(..) => item.clone()
        };
        om.view_items.push(item);
    }

    fn visit_view_path(&mut self, path: @ast::ViewPath,
                       om: &mut Module,
                       please_inline: bool) -> Option<@ast::ViewPath> {
        match path.node {
            ast::ViewPathSimple(ref ident, _, id) => {

                println!("!  {}::{}", self.current_prefix, token::get_ident(*ident));
                if self.resolve_id(id, false, om, please_inline) { return None }
            }
            ast::ViewPathList(ref p, ref paths, ref b) => {
                let mut mine = Vec::new();
                for path in paths.iter() {
                    let finnal_seg = path.node.name.clone();
                    println!("!* {}::{}", self.current_prefix, token::get_ident(finnal_seg));
                    if !self.resolve_id(path.node.id, false, om, please_inline) {
                        mine.push(path.clone());
                    }
                }

                if mine.len() == 0 { return None }
                return Some(@::syntax::codemap::Spanned {
                    node: ast::ViewPathList(p.clone(), mine, b.clone()),
                    span: path.span,
                })
            }

            // these are feature gated anyway
            ast::ViewPathGlob(_, id) => {
                println!("!! {}::*", self.current_prefix); // not support
                if self.resolve_id(id, true, om, please_inline) { return None }
            }
        }
        return Some(path);
    }

    fn resolve_id(&mut self, id: ast::NodeId, glob: bool,
                  om: &mut Module, please_inline: bool) -> bool {
        let tcx = match self.cx.maybe_typed {
            core::Typed(ref tcx) => tcx,
            core::NotTyped(_) => return false
        };
        let def = ast_util::def_id_of_def(*tcx.def_map.borrow().get(&id));
        if !ast_util::is_local(def) { return false }
        let analysis = match self.analysis {
            Some(analysis) => analysis, None => return false
        };
        if !please_inline && analysis.public_items.contains(&def.node) {
            return false
        }

        match tcx.map.get(def.node) {
            ast_map::NodeItem(it) => {
                if glob {
                    match it.node {
                        ast::ItemMod(ref m) => {
                            for vi in m.view_items.iter() {
                                self.visit_view_item(vi, om);
                            }
                            for i in m.items.iter() {
                                self.visit_item(*i, om);
                            }
                        }
                        _ => { fail!("glob not mapped to a module"); }
                    }
                } else {
                    self.visit_item(it, om);
                }
                true
            }
            _ => false,
        }
    }

    pub fn visit_item(&mut self, item: &ast::Item, om: &mut Module) {
        let name = token::get_ident(item.ident);
        match item.node {
            ast::ItemMod(ref m) => {
                let old_prefix = self.current_prefix.clone();
                println!("!m {}::{}", self.current_prefix, name);
                self.current_prefix = String::from_owned_str(format!("{}::{}", self.current_prefix, name));
                om.mods.push(self.visit_mod_contents(item.span,
                                                     item.attrs
                                                         .iter()
                                                         .map(|x| *x)
                                                         .collect(),
                                                     item.vis,
                                                     item.id,
                                                     m,
                                                     Some(item.ident)));
                self.current_prefix = old_prefix;
            },
            ast::ItemEnum(ref ed, ref gen) => {
                //let old_prefix = self.current_prefix.clone();
                //println!("!e {}::{}", self.current_prefix, name);
                //self.current_prefix = String::from_owned_str(format!("{}::{}", self.current_prefix, name));
                // enum variant share same prefix as enum
                om.enums.push(self.visit_enum_def(item, ed, gen));

                //self.current_prefix = old_prefix;
            },
            ast::ItemStruct(sd, ref gen) =>
                om.structs.push(self.visit_struct_def(item, sd, gen)),
            ast::ItemFn(fd, ref pur, ref abi, ref gen, _) => {
                println!("!f {}::{}", self.current_prefix, name);
                om.fns.push(self.visit_fn(item, fd, pur, abi, gen));
            },
            ast::ItemTy(ty, ref gen) => {
                println!("!t {}::{}", self.current_prefix, name);
                let t = Typedef {
                    ty: ty,
                    gen: gen.clone(),
                    name: item.ident,
                    id: item.id,
                    attrs: item.attrs.iter().map(|x| *x).collect(),
                    where: item.span,
                    vis: item.vis,
                };
                om.typedefs.push(t);
            },
            ast::ItemStatic(ty, ref mut_, ref exp) => {
                println!("!_ {}::{}", self.current_prefix, name);
                let s = Static {
                    type_: ty,
                    mutability: mut_.clone(),
                    expr: exp.clone(),
                    id: item.id,
                    name: item.ident,
                    attrs: item.attrs.iter().map(|x| *x).collect(),
                    where: item.span,
                    vis: item.vis,
                };
                om.statics.push(s);
            },
            ast::ItemTrait(ref gen, _, ref tr, ref met) => {
                println!("!T {}::{}", self.current_prefix, name);
                let t = Trait {
                    name: item.ident,
                    methods: met.iter().map(|x| (*x).clone()).collect(),
                    generics: gen.clone(),
                    parents: tr.iter().map(|x| (*x).clone()).collect(),
                    id: item.id,
                    attrs: item.attrs.iter().map(|x| *x).collect(),
                    where: item.span,
                    vis: item.vis,
                };
                om.traits.push(t);
            },
            ast::ItemImpl(ref gen, ref tr, ty, ref meths) => {
                // TODO
                println!("impl for => {}", ty.id);

                match ty.node {
                    ast::TyPath(ref p, _, _) => {
                        println!("!!!!!!!!!! {}", path_to_str(p));
                    },
                    _ => {}
                }
                // println!("type = {:?}", ty_to_str(ty));
                println!("imp name = {}", name);

                let i = Impl {
                    generics: gen.clone(),
                    trait_: tr.clone(),
                    for_: ty,
                    methods: meths.iter().map(|x| *x).collect(),
                    attrs: item.attrs.iter().map(|x| *x).collect(),
                    id: item.id,
                    where: item.span,
                    vis: item.vis,
                };
                om.impls.push(i);
            },
            ast::ItemForeignMod(ref fm) => {
                om.foreigns.push(fm.clone());
            }
            ast::ItemMac(ref _m) => {
                om.macros.push(Macro {
                    id: item.id,
                    attrs: item.attrs.iter().map(|x| *x).collect(),
                    name: item.ident,
                    where: item.span,
                })
            }
        }
    }
}
