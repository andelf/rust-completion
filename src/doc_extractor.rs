#![crate_id = "doc_extractor#0.1"]
#![crate_type = "dylib"]


extern crate rustdoc;
extern crate syntax;


use syntax::ast;
use rustdoc::clean;

use rustdoc::plugins::{PluginCallback, PluginResult, PluginJson};

// rustc src/doc_extractor.rs -o doc_extractor.dylib
// rustdoc -L. --plugin-path . --plugins dummy rust-sdl2/src/sdl2/lib.rs
// rustdoc --plugin-path . --plugins doc_extractor ~/Repos/rust/src/libcollections/lib.rs

pub trait Extractable {
    fn extract(&self, prefix: &str);
}


impl Extractable for clean::Crate {
    fn extract(&self, prefix: &str) {
        println!("crate: {}", self.name);
        match self.module {
            Some(ref i) => {
                i.extract(prefix + "::" + self.name);
            }
            _ => ()
        }
    }
}

impl Extractable for clean::Item {
    fn extract(&self, prefix: &str) {
        if self.visibility.unwrap_or(ast::Inherited) != ast::Public {
            //println!("ignore");
            //println!("debug {:?}", self);
            return
        }
        match self.name {
            Some(ref n) => {
                println!("Item name => {}::{}", prefix,  n);
                if n.len() > 0 {
                    self.inner.extract(prefix + "::" + *n)
                } else {
                    self.inner.extract(prefix)
                }
            }
            _ => {
                self.inner.extract(prefix)
                //println!("Item name => {}::**", prefix);
                //println!("debug => {:?}", self.inner)
            }
        }

    }
}

impl Extractable for clean::ItemEnum {
    fn extract(&self, prefix: &str) {
        match *self {
            clean::StructItem(ref s) => {
                println!("Struct => {}", s.struct_type);
                for item in s.fields.iter() {
                    //println!("vis => {:?}", item.visibility);
                    if item.visibility.is_none() {
                        continue
                    }
                    match item.inner {
                        clean::StructFieldItem(ref f) => {
                            match *f {
                                clean::TypedStructField(ref tp) => {
                                    println!("| {}: {:?}", item.name.as_ref().unwrap_or(&"".to_owned()), tp)
                                }
                                _ => () // HiddenStructField
                            }
                        }
                        _ => {
                            unreachable!()
                        }
                    }

                }
            }
            clean::FunctionItem(ref f) => {
                //println!("{}() => {} -> {:?}", prefix, f.decl.inputs, f.decl.output);
                println!("{}() f", prefix);
            }
            clean::ModuleItem(ref m) => {
                // is top level crate
                //println!("is_crate: {}", m.is_crate);
                for item in m.items.iter() {
                    item.extract(prefix);
                }
            }
            clean::TypedefItem(ref t) => {
                //println!("| {} {:?}", t.generics, t.type_);
                println!("typedef!");
            }
            clean::TraitItem(ref t) => {
                for m in t.methods.iter() {
                    match *m {
                        clean::Required(ref i) => {
                            println!("| {}()", i.name.as_ref().expect("a method name"));
                        }
                        clean::Provided(ref i) => {
                            println!("| {}()", i.name.as_ref().expect("a method name"));
                        }
                    }
                }
            }

            clean::ViewItemItem(ref v) => {
                v.extract(prefix);
            }
            ref i => {
                println!("unkown => {:?}", i);
            }
        }
    }
}

impl Extractable for clean::ViewItem {
    fn extract(&self, prefix: &str) {
        //println!("{}::{}", prefix)
        match self.inner {
            clean::ExternCrate(ref name, ref cname_opt, _) => {
                println!("fuck");
                println!("{}::{} = {}", prefix, cname_opt, name);
            }
            clean::Import(ref vp) => {
                //println!("import => {:?}", vp)
                vp.extract(prefix);
            }
        }
    }
}

impl Extractable for clean::ViewPath {
    fn extract(&self, prefix: &str) {
        match *self {
            clean::SimpleImport(ref name, ref src) => {
                //println!("{}::{} = {}", prefix, name, src.path)
                println!("use {} = {}", name, src.path)
            }
            clean::GlobImport(ref src) => {
                println!("{}::* = {}", prefix, src.path)
                //println!("{}::* = {}", prefix, src.path)
            }
            clean::ImportList(ref src, ref lst) => {
                //print!("{}:: use {}::\\{", prefix, src);
                print!("use {}::\\{", src);
                for ident in lst.iter() {
                    print!("{},", ident.name)
                }
                println!("\\}")
            }
        }
    }
}


fn dump_item_enum(item: &clean::ItemEnum, ident_level: uint) {
    match *item {
        clean::ModuleItem(ref m) => {
            println!("{}!A Module", " ".repeat(ident_level));
            println!("{}| module items => {:?}", " ".repeat(ident_level), m.items);
            for item in m.items.iter() {
                dump_item(item, ident_level + 2)
            }
        },
        clean::ViewItemItem(ref v) => {
            println!("{}!A ViewItem", " ".repeat(ident_level));
            match v.inner {
                clean::Import(ref vpath) => {
                    print!("{}| ", " ".repeat(ident_level));
                    println!("{:?}", vpath);
                }
                _ => {
                    print!("{}| ", " ".repeat(ident_level));
                    println!("{}", "extern create");
                }
            }
        }
        ref i => {
            println!("{}!{:?}", " ".repeat(ident_level), i)
        }
    }
}

fn dump_item(item: &clean::Item, ident_level: uint) {
    println!("{}| item name => {:?}", " ".repeat(ident_level), item.name);
    println!("{}| vis => {:?}", " ".repeat(ident_level), item.visibility);

    // for attr in item.attrs.iter() {
    //     println!("{}| attr => {:?}", " ".repeat(ident_level), attr);
    // }
    //println!("{}| inner => {:?}", " ".repeat(ident_level), item.inner);
    if item.visibility.expect("visibility here") == ast::Public {
        dump_item_enum(&item.inner, ident_level);
    }
}

#[no_mangle]
pub fn rustdoc_plugin_entrypoint(c: clean::Crate) -> PluginResult {
    println!("loading extension ok!");
    println!("crate => {}", c.name);

    c.module.as_ref().map(|m| dump_item(m, 0));
    // externs is useless for our app
    // for ext in c.externs.iter() {
    //     println!("externs => {:?}", *ext);
    //     for attr in ext.ref1().attrs.iter() {
    //         println!("attrs => {:?}", attr);
    //     }
    // }
    println!("{}", "=".repeat(78));
    c.extract("");
    (c, None)

}
