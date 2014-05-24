use syntax::parse::token;
use syntax::ast;
use syntax::ext::quote::rt::ToSource;

pub fn view_item_to_str(i: &ast::ViewItem) -> StrBuf {
    let mut ret = StrBuf::new();
    match i.node {
        // ignore
        ast::ViewItemExternCrate(ref ident, ref location, _) => {
            ret.push_str("extern crate ");
            ret.push_str(token::get_ident(*ident).get());
            if location.is_some() {
                ret.push_str(format!(" = {}", location.as_ref().unwrap().ref0()).as_slice());
            }
        }
        // pub use
        ast::ViewItemUse(ref vp) => {
            if i.vis == ast::Public {
                ret.push_str("pub ")
            }
            ret.push_str("use ");
            match vp.node {
                ast::ViewPathSimple(ref ident, ref path, _) => {
                    ret.push_str(format!("{} = {}",
                                         token::get_ident(*ident).get(),
                                         path_to_str(path).as_slice(),
                                         ).as_slice());
                }
                ast::ViewPathGlob(ref path, _) => {
                    ret.push_str(format!("{}::*", path_to_str(path).as_slice()).as_slice());
                }
                ast::ViewPathList(ref path, ref idents, _) => {
                    ret.push_str(format!("{}::", path_to_str(path).as_slice()).as_slice());
                    if !idents.is_empty() {
                        ret.push_str("{");
                        ret.push_str(idents.iter().map(|i| token::get_ident(i.node.name).get().to_owned()).collect::<Vec<StrBuf>>().connect(",").as_slice());
                        ret.push_str("}");
                    }
                }
            }
        }
    }
    ret
}

pub fn path_to_str(i: &ast::Path) -> StrBuf {
    let mut ret = StrBuf::new();
    if i.global { ret.push_str("::") }
    ret.push_str(i.segments.iter().map(|seg| token::get_ident(seg.identifier).get().to_owned()).collect::<Vec<StrBuf>>().connect("::").as_slice());
    ret
}

pub fn item_to_str(i: &ast::Item) -> StrBuf {
    let mut ret = StrBuf::new();
    if i.vis == ast::Public {
        ret.push_str(format!("pub ").as_slice());
    }
    match i.node {
        ast::ItemStatic(..) => {
            ret.push_str(format!("ItemStatic").as_slice());
        }
        ast::ItemFn(..) => {
            ret.push_str(format!("ItemPub Fn").as_slice());
        }
        ast::ItemMod(..) => {
            ret.push_str(format!("ItemMod").as_slice());
        }
        ast::ItemForeignMod(..) => {
            ret.push_str(format!("ItemForeignMod").as_slice());
        }
        ast::ItemTy(..) => {
            ret.push_str(format!("ItemTy").as_slice());
        }
        ast::ItemEnum(..) => {
            ret.push_str(format!("ItemEnum").as_slice());
        }
        ast::ItemStruct(..) => {
            ret.push_str(format!("ItemStruct").as_slice());
        }
        ast::ItemTrait(..) => {
            ret.push_str(format!("ItemTrait").as_slice());
        }
        ast::ItemImpl(..) => {
            ret.push_str(format!("ItemImpl").as_slice());
        }
        ast::ItemMac(..) => {
            ret.push_str(format!("ItemMac").as_slice());
        }
    }
    ret.push_str(format!(" ITEM: {} ", i.ident.to_source()).as_slice());
    ret
}


// pub fn path_to_str(p: &ast::Path) -> ~str {
//     let mut ret = StrBuf::new();
//     if p.global {
//         ret.push_str("::")
//     }
//     ret.push_str(
//         p.segments.iter().map(|seg| {
//             let mut tmp = StrBuf::new();
//             tmp.push_str(get_ident(seg.identifier).get().to_owned());
//             // if !seg.types.is_empty() {
//             //     tmp.push_str("<");
//             //     tmp.push_str(seg.types.iter().map(|t| type_to_str(t)).collect::<Vec<~str>>().connect(","));
//             //     tmp.push_str(">");
//             // }
//             tmp.to_owned()
//         }).collect::<Vec<~str>>().connect("::"));
//     ret.to_owned()
// }


pub fn ty_to_str(t: &ast::Ty) -> StrBuf {
    match t.node {
        ast::TyNil => {}
        ast::TyBot => {}
        ast::TyBox(..) => {}
        ast::TyUniq(..) => {}
        ast::TyVec(..) => {}
        ast::TyFixedLengthVec(..) => {}
        ast::TyPtr(..) => {}
        ast::TyRptr(..) => {}
        ast::TyClosure(..) => {}
        ast::TyProc(..) => {}
        ast::TyBareFn(..) => {}
        ast::TyTup(..) => {}
        ast::TyPath(ref p, _, id) => {
            println!("path ~! {}", path_to_str(p))
        }
        ast::TyTypeof(..) => {}
        ast::TyInfer => {}
    }
    StrBuf::new()
}
