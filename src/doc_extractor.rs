#![crate_id = "doc_extractor#0.1"]
#![crate_type = "dylib"]


extern crate rustdoc;
extern crate syntax;
extern crate collections;


use syntax::ast;
use rustdoc::clean;
use collections::HashMap;

use rustdoc::plugins::{PluginCallback, PluginResult, PluginJson};

// rustc src/doc_extractor.rs -o doc_extractor.dylib
// rustdoc -L. --plugin-path . --plugins dummy rust-sdl2/src/sdl2/lib.rs
// rustdoc --plugin-path . --plugins doc_extractor ~/Repos/rust/src/libcollections/lib.rs

// #[deriving(Eq, Hash, Show)]
// pub enum CompletionPart {
//     PathSegment(~str),
//     Struct
// }



pub trait Extractable {
    fn extract(&self, prefix: &str, vis: Option<clean::Visibility>);
}


impl Extractable for clean::Crate {
    fn extract(&self, prefix: &str, vis: Option<clean::Visibility>) {
        println!("crate: {}", self.name);
        match self.module {
            Some(ref i) => {
                i.extract(prefix + "::" + self.name, vis);
            }
            _ => ()
        }
    }
}

impl Extractable for clean::Item {
    fn extract(&self, prefix: &str, vis: Option<clean::Visibility>) {
        if self.visibility.is_none() { // unwrap_or(ast::Inherited) != ast::Public {
            //println!("ignore");
            //println!("debug {:?}", self);
            //println!("{} vis => {:?}", prefix, self.visibility)
            return
        }
        match self.name {
            Some(ref n) => {
                println!("Item name => {}::{}", prefix,  n);
                //println!("{}::{} vis => {:?}", prefix, n, self.visibility)
                if n.len() > 0 {
                    self.inner.extract(prefix + "::" + *n, self.visibility)
                } else {
                    self.inner.extract(prefix, self.visibility)
                }
            }
            _ => {
                self.inner.extract(prefix, self.visibility)
                //println!("Item name => {}::**", prefix);
                //println!("debug => {:?}", self.inner)
            }
        }

    }
}

impl Extractable for clean::ItemEnum {
    fn extract(&self, prefix: &str, vis: Option<clean::Visibility>) {
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
                                    println!("  | {}: {}", item.name.as_ref().unwrap_or(&"".to_owned()), type_to_str(tp))
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
            clean::EnumItem(ref e) => {
                println!("Enum => {}", generics_to_str(&e.generics));
                for i in e.variants.iter() {
                    println!("  | {}", i.name.as_ref().expect("a enum variant here"));
                }
            }
            clean::FunctionItem(ref f) => {
                //println!("{}() => {} -> {:?}", prefix, f.decl.inputs, f.decl.output);
                println!("vis => {:?}", vis);
                println!("fn {}{}(...) -> {}", prefix, generics_to_str(&f.generics), type_to_str(&f.decl.output));
            }
            clean::ModuleItem(ref m) => {
                // is top level crate
                //println!("is_crate: {}", m.is_crate);
                //println!("vis => {:?}", vis);
                // only pub mod
                if vis.unwrap_or(ast::Inherited) == ast::Public {
                    println!("mod {}", prefix);
                    for item in m.items.iter() {
                        item.extract(prefix, vis);
                    }
                }
            }
            clean::TypedefItem(ref t) => {
                //println!("| {} {:?}", t.generics, t.type_);
                //println!("typedef!");
                println!("type {} = {}{}", prefix, generics_to_str(&t.generics), type_to_str(&t.type_));
            }
            clean::StaticItem(ref s) => {
                println!("static {}: {}", prefix, type_to_str(&s.type_));
            }
            clean::TraitItem(ref t) => {
                println!("trait {}{}", prefix, generics_to_str(&t.generics))
                for m in t.methods.iter() {
                    match *m {
                        clean::Required(ref i) => {
                            let method_name = i.name.as_ref().expect("a method name");
                            match i.inner {
                                clean::TyMethodItem(ref m) => {
                                    println!("  | {}{}", method_name, tymethod_to_str(m))
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                            //println!("  | {:?}", i);
                        }
                        clean::Provided(ref i) => {
                            let method_name = i.name.as_ref().expect("a method name");
                            match i.inner {
                                clean::MethodItem(ref m) => {
                                    println!("  | {}{}", method_name, method_to_str(m))
                                }
                                _ => {
                                    unreachable!()
                                }
                            }
                        }
                    }
                }
            }
            clean::ImplItem(ref i) => {
                if i.trait_.is_some() {
                    println!("impl {} for {}", i.trait_.as_ref().map(type_to_str).unwrap(), type_to_str(&i.for_))
                } else {
                    println!("impl {}", type_to_str(&i.for_));
                }
                for m in i.methods.iter() {
                    // methods
                    //println!("m.name => {}", m.name);
                    //m.extract("."); //*m.name.as_ref().expect("a method name"));
                    //m.extract(*m.name.as_ref().expect("a method name"));
                    m.extract("", vis);
                }
            }
            clean::ViewItemItem(ref v) => {
                v.extract(prefix, vis);
            }
            clean::MethodItem(ref m) => {
                println!("{}{}", prefix, method_to_str(m));
                //dump_method_type(m);
            }
            clean::ForeignFunctionItem(ref f) => {
                println!("extern fn {}()", prefix);
            }
            ref i => {
                println!("unkown => {:?}", i);
            }
        }
    }
}

impl Extractable for clean::ViewItem {
    fn extract(&self, prefix: &str, vis: Option<clean::Visibility>) {
        //println!("{}::{}", prefix)
        match self.inner {
            clean::ExternCrate(ref name, ref cname_opt, _) => {
                println!("fuck");
                println!("{}::{} = {}", prefix, cname_opt, name);
            }
            clean::Import(ref vp) => {
                //println!("import => {:?}", vp)
                vp.extract(prefix, vis);
            }
        }
    }
}

impl Extractable for clean::ViewPath {
    fn extract(&self, prefix: &str, vis: Option<clean::Visibility>) {
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

fn typaram_to_str(t: &clean::TyParam) -> ~str {
    let mut ret = StrBuf::new();
    //ret.push_str(t.name);
    ret.push_str(format!("T_{}", t.id));
    if !t.bounds.is_empty() {

        ret.push_str(":(");
        for p in t.bounds.iter() {
            match *p {
                clean::TraitBound(ref tp) => {
                    ret.push_str(type_to_str(tp));
                    ret.push_str("+")
                }
                _ => ()
            }
        }
        ret.push_str(")");
    }
    ret.to_owned()
}

fn generics_to_str(g: &clean::Generics) -> ~str {
    let mut segs = Vec::new();
    g.lifetimes.iter().map(|ref l| segs.push(format!("'{}", l.get_ref()))).collect::<Vec<()>>();
    g.type_params.iter().map(|ref t| segs.push(typaram_to_str(*t))).collect::<Vec<()>>();
    let content = segs.connect(", ");
    if content.len() > 0 {
        format!("<{}>", content)
    } else {
        "".to_owned()
    }
}


fn tymethod_to_str(m: &clean::TyMethod) -> ~str {
    let mut ret = StrBuf::new();
    ret.push_str(generics_to_str(&m.generics));
    ret.push_str("(");
    let mut args : Vec<~str> = Vec::new();
    match m.self_ {
        clean::SelfStatic => (),
        clean::SelfValue  => {
            args.push("self".to_owned())
        }
        clean::SelfBorrowed(ref lftm, ref mutable) => {
            match *mutable {
                clean::Mutable => {
                    args.push(format!("&{}mut self", lftm.as_ref().map(|l| format!("'{} ", l.get_ref())).unwrap_or("".to_owned())))
                }
                _ => {
                    args.push(format!("&{}self", lftm.as_ref().map(|l| format!("'{} ", l.get_ref())).unwrap_or("".to_owned())))
                }
            }
        }
        clean::SelfOwned => {
            args.push("~self".to_owned())
        }
    }
    for arg in m.decl.inputs.values.iter() {
        args.push(format!("{}: {}", arg.name, type_to_str(&arg.type_)));
    }

    ret.push_str(args.connect(", "));
    ret.push_str(") -> ");
    ret.push_str(format!("{}", type_to_str(&m.decl.output)));
    ret.to_owned()

}
fn method_to_str(m: &clean::Method) -> ~str {
    let mut ret = StrBuf::new();
    ret.push_str(generics_to_str(&m.generics));
    ret.push_str("(");
    // for l in m.generics.lifetimes.iter() {
    //     gen.push_str(format!("'{},", l.get_ref()))
    // }
    // for t in m.generics.type_params.iter() {
    //     gen.push_str(format!("T_{}:{},", t.id, t.name))
    // }
    //print!("type is: ");
    // if gen.len() > 0 {
    //     //print!("<{}>(", gen);
    //     ret.push_str(format!("<{}>(", gen));
    // } else {
    //     ret.push_str("(")
    // }

    let mut args : Vec<~str> = Vec::new();
    match m.self_ {
        clean::SelfStatic => (),
        clean::SelfValue  => {
            args.push("self".to_owned())
        }
        clean::SelfBorrowed(ref lftm, ref mutable) => {
            match *mutable {
                clean::Mutable => {
                    args.push(format!("&{}mut self", lftm.as_ref().map(|l| format!("'{} ", l.get_ref())).unwrap_or("".to_owned())))
                }
                _ => {
                    args.push(format!("&{}self", lftm.as_ref().map(|l| format!("'{} ", l.get_ref())).unwrap_or("".to_owned())))
                }
            }
        }
        clean::SelfOwned => {
            args.push("~self".to_owned())
        }
    }
    for arg in m.decl.inputs.values.iter() {
        //print!("{}: {},", arg.name, "type");
        //print!("{}: {}, ", arg.name, type_to_str(&arg.type_));
        args.push(format!("{}: {}", arg.name, type_to_str(&arg.type_)));
    }

    ret.push_str(args.connect(", "));
    //print!("{}", m.decl.inputs);
    //print!(") -> ");
    ret.push_str(") -> ");
    ret.push_str(format!("{}", type_to_str(&m.decl.output)));
    ret.to_owned()
}



fn type_to_str(t: &clean::Type) -> ~str {
    let mut ret = StrBuf::new();
    match *t {
        clean::ResolvedPath { path: ref p, .. } => {
            //format!("{}", p)  // empty
            if p.global {
                ret.push_str("::")
            }
            ret.push_str(path_to_str(p));
            ret.to_owned()
        }
        clean::ExternalPath { fqn: ref f, typarams: ref params, .. } => {
            // println!("path {:?}", p); // only has last item
            // println!("fqn {}", f);
            // println!("pos2: {}", path_to_str(p));
            ret.push_str("::");
            ret.push_str(f.connect("::"));
            if params.is_some() {
                ret.push_str("<");
                // ret.push_str(params.unwrap().iter().map(typaram_to_str).collect::<Vec<&str>>().connect(", "));
                ret.push_str(">");
            }
            ret.to_owned()
        }
        clean::Tuple(ref ts) => {
            ret.push_str("(");
            for t in ts.iter() {
                ret.push_str(type_to_str(t));
                ret.push_str(", ");
            }
            ret.push_str(")");
            ret.to_owned()
        }
        clean::Primitive(ref p) => {
            match *p {
                ast::TyInt(_) => "int",
                ast::TyUint(_) => "uint",
                ast::TyFloat(_) => "float",
                ast::TyStr => "str",
                ast::TyBool => "bool",
                ast::TyChar => "char",
            }.to_owned()
        }
        clean::FixedVector(ref tp, ref num) => {
            format!("[{}, ..{}]", type_to_str(*tp), num)
        }
        clean::Vector(ref tp) => {
            format!("Vec<{}>", type_to_str(*tp))
        }
        clean::String => "str".to_owned(),
        clean::Bool => "bool".to_owned(),
        clean::Unit => "()".to_owned(),
        clean::Bottom => "!".to_owned(),
        clean::Unique(ref tp) => {
            format!("~{}", type_to_str(*tp))
        }
        clean::RawPointer(ref mutable, ref tp) => {
            ret.push_str("*");
            match *mutable {
                clean::Mutable => {
                    ret.push_str("mut ")
                }
                _ => ()
            }
            ret.push_str(type_to_str(*tp));
            ret.to_owned()
        }
        clean::BorrowedRef{ mutability: ref mutable, type_: ref tp, .. } => {
            ret.push_str("&");
            match *mutable {
                clean::Mutable => {
                    ret.push_str("mut ")
                }
                _ => ()
            }
            ret.push_str(type_to_str(*tp));
            ret.to_owned()
        }
        clean::Generic(ref g) => format!("T_{}", *g),
        clean::Self(..) => "Self".to_owned(),
        clean::Closure(..) => "||".to_owned(),
        _ => {
            println!("pos1: {:?}", t);
            "".to_owned()
        }
    }
}

fn path_to_str(p: &clean::Path) -> ~str {
    let mut ret = StrBuf::new();
    if p.global {
        ret.push_str("::")
    }
    for seg in p.segments.iter() {
        ret.push_str(seg.name)
    }
    ret.to_owned()
}

// fn dump_item(item: &clean::Item, ident_level: uint) {
//     println!("{}| item name => {:?}", " ".repeat(ident_level), item.name);
//     println!("{}| vis => {:?}", " ".repeat(ident_level), item.visibility);

//     // for attr in item.attrs.iter() {
//     //     println!("{}| attr => {:?}", " ".repeat(ident_level), attr);
//     // }
//     //println!("{}| inner => {:?}", " ".repeat(ident_level), item.inner);
//     if item.visibility.expect("visibility here") == ast::Public {
//         dump_item_enum(&item.inner, ident_level);
//     }
// }

#[no_mangle]
pub fn rustdoc_plugin_entrypoint(c: clean::Crate) -> PluginResult {
    println!("loading extension ok!");
    println!("crate => {}", c.name);

    //c.module.as_ref().map(|m| dump_item(m, 0));
    // externs is useless for our app
    // for ext in c.externs.iter() {
    //     println!("externs => {:?}", *ext);
    //     for attr in ext.ref1().attrs.iter() {
    //         println!("attrs => {:?}", attr);
    //     }
    // }
    println!("{}", "=".repeat(78));
    c.extract("", Some(ast::Public));
    (c, None)

}
