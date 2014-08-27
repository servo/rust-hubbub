// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// High-level bindings to the Hubbub HTML5 parser.

use libc;
use libc::{c_void, size_t};
use std::mem;
use std::ptr;
use ll;

pub enum QuirksMode {
    NoQuirks,
    LimitedQuirks,
    FullQuirks
}

pub enum Ns {
    NullNs,
    HtmlNs,
    MathMlNs,
    SvgNs,
    XLinkNs,
    XmlNs,
    XmlNsNs
}

pub struct Doctype {
    pub name: String,
    pub public_id: Option<String>,
    pub system_id: Option<String>,
    pub force_quirks: bool
}

pub struct Attribute {
    pub ns: Ns,
    pub name: String,
    pub value: String,
}

pub struct Tag {
    pub ns: Ns,
    pub name: String,
    pub attributes: Vec<Attribute>,
    pub self_closing: bool
}

// FIXME: This is terribly type-unsafe. But we don't have working generic extern functions yet...
pub type NodeDataPtr = uint;

pub struct TreeHandler<'a> {
    pub create_comment: |data: String|: 'a -> NodeDataPtr,
    pub create_doctype: |doctype: Box<Doctype>|: 'a -> NodeDataPtr,
    pub create_element: |tag: Box<Tag>|: 'a -> NodeDataPtr,
    pub create_text: |data: String|: 'a -> NodeDataPtr,
    pub ref_node: |node: NodeDataPtr|: 'a,
    pub unref_node: |node: NodeDataPtr|: 'a,
    pub append_child: |parent: NodeDataPtr, child: NodeDataPtr|: 'a -> NodeDataPtr,
    pub insert_before: |parent: NodeDataPtr, child: NodeDataPtr|: 'a -> NodeDataPtr,
    pub remove_child: |parent: NodeDataPtr, child: NodeDataPtr|: 'a -> NodeDataPtr,
    pub clone_node: |node: NodeDataPtr, deep: bool|: 'a -> NodeDataPtr,
    pub reparent_children: |node: NodeDataPtr, new_parent: NodeDataPtr|: 'a -> NodeDataPtr,
    pub get_parent: |node: NodeDataPtr, element_only: bool|: 'a -> NodeDataPtr,
    pub has_children: |node: NodeDataPtr|: 'a -> bool,
    pub form_associate: |form: NodeDataPtr, node: NodeDataPtr|: 'a,
    pub add_attributes: |node: NodeDataPtr, attributes: Vec<Attribute>|: 'a,
    pub set_quirks_mode: |mode: QuirksMode|: 'a,
    pub encoding_change: |encname: String|: 'a,
    pub complete_script: |script: NodeDataPtr|: 'a,
    pub complete_style: |style: NodeDataPtr|: 'a,
}

pub struct TreeHandlerPair<'a> {
    pub tree_handler: &'a mut TreeHandler<'a>,
    pub ll_tree_handler: ll::TreeHandler
}

pub struct Parser<'a> {
    pub hubbub_parser: *mut ll::Parser,
    pub tree_handler: Option<TreeHandlerPair<'a>>,
}

#[unsafe_destructor]
impl<'a> Drop for Parser<'a> {
    fn drop(&mut self) {
        unsafe { ll::parser::hubbub_parser_destroy(self.hubbub_parser) };
    }
}

impl<'a> Parser<'a> {
    pub fn new(encoding: &str, fix_encoding: bool) -> Parser {
        let mut hubbub_parser = ptr::mut_null();
        let encoding_c = encoding.to_c_str();
        let hubbub_error = unsafe {
            ll::parser::hubbub_parser_create(encoding_c.as_ptr() as *const u8, fix_encoding, allocator,
                                             ptr::mut_null(), &mut hubbub_parser)
        };
        assert!(hubbub_error == ll::OK);

        Parser {
            hubbub_parser: hubbub_parser,
            tree_handler: None
        }
    }

    pub fn set_tree_handler(&mut self, tree_handler: &'a mut TreeHandler<'a>) {
        self.tree_handler = Some(TreeHandlerPair {
            tree_handler: tree_handler,
            ll_tree_handler: ll::TreeHandler {
                create_comment: tree_callbacks::create_comment,
                create_doctype: tree_callbacks::create_doctype,
                create_element: tree_callbacks::create_element,
                create_text: tree_callbacks::create_text,
                ref_node: tree_callbacks::ref_node,
                unref_node: tree_callbacks::unref_node,
                append_child: tree_callbacks::append_child,
                insert_before: tree_callbacks::insert_before,
                remove_child: tree_callbacks::remove_child,
                clone_node: tree_callbacks::clone_node,
                reparent_children: tree_callbacks::reparent_children,
                get_parent: tree_callbacks::get_parent,
                has_children: tree_callbacks::has_children,
                form_associate: tree_callbacks::form_associate,
                add_attributes: tree_callbacks::add_attributes,
                set_quirks_mode: tree_callbacks::set_quirks_mode,
                encoding_change: tree_callbacks::encoding_change,
                complete_script: tree_callbacks::complete_script,
                complete_style: tree_callbacks::complete_style,
                ctx: unsafe { mem::transmute(&self.tree_handler) },
            }
        });

        let ptr: *mut ll::TreeHandler =
            &mut self.tree_handler.get_mut_ref().ll_tree_handler;

        unsafe {
            let hubbub_error = ll::parser::hubbub_parser_setopt(self.hubbub_parser,
                                                                ll::PARSER_TREE_HANDLER,
                                                                mem::transmute(&ptr));
            assert!(hubbub_error == ll::OK);
        }
    }

    pub fn set_document_node(&mut self, node: NodeDataPtr) {
        unsafe {
            debug!("setting document node");
            let hubbub_error = ll::parser::hubbub_parser_setopt(self.hubbub_parser,
                                                                ll::PARSER_DOCUMENT_NODE,
                                                                mem::transmute(&node));
            assert!(hubbub_error == ll::OK);
        }
    }

    pub fn enable_scripting(&mut self, enable: bool) {
        unsafe {
            debug!("enabling scripting");
            let hubbub_error = ll::parser::hubbub_parser_setopt(self.hubbub_parser,
                                                                ll::PARSER_ENABLE_SCRIPTING,
                                                                mem::transmute(&enable));
            assert!(hubbub_error == ll::OK);
        }
    }

    pub fn enable_styling(&mut self, enable: bool) {
        unsafe {
            debug!("enabling styling");
            let hubbub_error = ll::parser::hubbub_parser_setopt(self.hubbub_parser,
                                                                ll::PARSER_ENABLE_STYLING,
                                                                mem::transmute(&enable));
            assert!(hubbub_error == ll::OK);
        }
    }

    pub fn parse_chunk(&mut self, data: &[u8]) {
        unsafe {
            debug!("parsing chunk");
            let ptr = data.as_ptr();
            let hubbub_error = ll::parser::hubbub_parser_parse_chunk(self.hubbub_parser, ptr,
                                                                     data.len() as size_t);
            assert!(hubbub_error == ll::OK);
        }
    }

    pub fn insert_chunk(&mut self, data: &[u8]) {
        unsafe {
            debug!("inserting chunk");
            let ptr = data.as_ptr();
            let hubbub_error = ll::parser::hubbub_parser_insert_chunk(self.hubbub_parser, ptr,
                                                                      data.len() as size_t);
            assert!(hubbub_error == ll::OK);
        }
    }

    pub fn completed(&self) {
        unsafe {
            debug!("completing");
            let hubbub_error = ll::parser::hubbub_parser_completed(self.hubbub_parser);
            assert!(hubbub_error == ll::OK);
        }
    }
}

pub mod tree_callbacks {

    use libc::{c_void, c_char};
    use std::mem;
    use std::ptr::RawPtr;
    use std::string;
    use super::{NodeDataPtr, Ns, NullNs, HtmlNs, MathMlNs, SvgNs, XLinkNs, XmlNs, XmlNsNs};
    use super::{QuirksMode, NoQuirks, LimitedQuirks, FullQuirks};
    use super::{Attribute, Tag, Doctype, TreeHandlerPair};
    use ll;

    // Data conversions

    pub fn from_hubbub_node(node: *mut c_void) -> NodeDataPtr {
        unsafe { mem::transmute(node) }
    }

    pub fn from_hubbub_string(string: &ll::String) -> String {
        unsafe {
            debug!("from_hubbub_string: {:u}", (*string).len as uint);
            let s = string::raw::from_buf_len(&*(*string).ptr, (*string).len as uint);
            debug!("from_hubbub_string: {:s}", s);
            s
        }
    }

    pub fn from_hubbub_ns(ns: ll::NS) -> Ns {
        match ns {
            0 => NullNs,
            1 => HtmlNs,
            2 => MathMlNs,
            3 => SvgNs,
            4 => XLinkNs,
            5 => XmlNs,
            6 => XmlNsNs,
            _ => fail!("unknown namespace")
        }
    }

    pub fn from_hubbub_quirks_mode(mode: ll::QuirksMode) -> QuirksMode {
        match mode {
            0 => NoQuirks,
            1 => LimitedQuirks,
            2 => FullQuirks,
            _ => fail!("unknown quirks mode")
        }
    }

    pub fn from_hubbub_attributes(attributes: *mut ll::Attribute, n_attributes: u32) -> Vec<Attribute> {
        debug!("from_hubbub_attributes n={:u}", n_attributes as uint);
        unsafe {
            Vec::from_fn(n_attributes as uint, |i| {
                let attribute = attributes.offset(i as int);
                Attribute {
                    ns: from_hubbub_ns((*attribute).ns),
                    name: from_hubbub_string(&(*attribute).name),
                    value: from_hubbub_string(&(*attribute).value)
                }
            })
        }
    }

    pub fn from_hubbub_tag(tag: &ll::Tag) -> Box<Tag> {
        box Tag {
            ns: from_hubbub_ns((*tag).ns),
            name: from_hubbub_string(&(*tag).name),
            attributes: from_hubbub_attributes((*tag).attributes, (*tag).n_attributes),
            self_closing: (*tag).self_closing
        }
    }

    pub fn from_hubbub_doctype(doctype: &ll::Doctype) -> Box<Doctype> {
        box Doctype {
            name: from_hubbub_string(&doctype.name),
            public_id:
                if doctype.public_missing {
                    None
                } else {
                    Some(from_hubbub_string(&doctype.public_id))
                },
            system_id:
                if doctype.system_missing {
                    None
                } else {
                    Some(from_hubbub_string(&doctype.system_id))
                },
            force_quirks: doctype.force_quirks
        }
    }

    pub fn to_hubbub_node(node: NodeDataPtr) -> *mut c_void {
        unsafe { mem::transmute(node) }
    }

    // Callbacks

    pub extern fn create_comment(ctx: *mut c_void, data: *mut ll::String, result: *mut *mut c_void)
                          -> ll::Error {
        debug!("ll create comment");
        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        unsafe {
            let data: &ll::String = mem::transmute(data);
            *result = to_hubbub_node((this.tree_handler.create_comment)(from_hubbub_string(data)));
        }
        return ll::OK;
    }

    pub extern fn create_doctype(ctx: *mut c_void, doctype: *mut ll::Doctype, result: *mut *mut c_void)
                          -> ll::Error {
        debug!("ll create doctype");
        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        unsafe {
            let doctype: &ll::Doctype = mem::transmute(doctype);
            *result = to_hubbub_node((this.tree_handler.create_doctype)(from_hubbub_doctype(doctype)));
        }
        return ll::OK;
    }

    pub extern fn create_element(ctx: *mut c_void, tag: *mut ll::Tag, result: *mut *mut c_void)
                          -> ll::Error {
        debug!("ll create element");
        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        unsafe {
            let tag: &ll::Tag = mem::transmute(tag);
            *result = to_hubbub_node((this.tree_handler.create_element)(from_hubbub_tag(tag)));
        }
        return ll::OK;
    }

    pub extern fn create_text(ctx: *mut c_void, data: *mut ll::String, result: *mut *mut c_void)
                       -> ll::Error {
        debug!("ll create text");
        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        unsafe {
            let data: &ll::String = mem::transmute(data);
            *result = to_hubbub_node((this.tree_handler.create_text)(from_hubbub_string(data)));
        }
        return ll::OK;
    }

    pub extern fn ref_node(ctx: *mut c_void, node: *mut c_void) -> ll::Error {
        debug!("ll ref node");
        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        (this.tree_handler.ref_node)(from_hubbub_node(node));
        return ll::OK;
    }

    pub extern fn unref_node(ctx: *mut c_void, node: *mut c_void) -> ll::Error {
        debug!("ll unref node");
        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        (this.tree_handler.unref_node)(from_hubbub_node(node));
        return ll::OK;
    }

    pub extern fn append_child(ctx: *mut c_void, parent: *mut c_void, child: *mut c_void, result: *mut *mut c_void)
                        -> ll::Error {
        debug!("ll append child");
        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        unsafe {
            *result = to_hubbub_node((this.tree_handler.append_child)(from_hubbub_node(parent),
                                                                      from_hubbub_node(child)));
        }
        return ll::OK;
    }

    pub extern fn insert_before(ctx: *mut c_void, parent: *mut c_void, child: *mut c_void,
                                result: *mut *mut c_void) -> ll::Error {
        debug!("ll insert before");
        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        unsafe {
            *result = to_hubbub_node((this.tree_handler.insert_before)(from_hubbub_node(parent),
                                                                       from_hubbub_node(child)));
        }
        return ll::OK;
    }

    pub extern fn remove_child(ctx: *mut c_void, parent: *mut c_void, child: *mut c_void, result: *mut *mut c_void)
                        -> ll::Error {
        debug!("ll remove child");

        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        unsafe {
            *result = to_hubbub_node((this.tree_handler.remove_child)(from_hubbub_node(parent),
                                                                      from_hubbub_node(child)));
        }
        return ll::OK;
    }

    pub extern fn clone_node(ctx: *mut c_void, node: *mut c_void, deep: bool, result: *mut *mut c_void)
                      -> ll::Error {
        debug!("ll clone node");

        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        unsafe {
            *result = to_hubbub_node((this.tree_handler.clone_node)(from_hubbub_node(node), deep));
        }
        return ll::OK;
    }

    pub extern fn reparent_children(ctx: *mut c_void, node: *mut c_void, new_parent: *mut c_void)
                             -> ll::Error {
        debug!("ll reparent children");

        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        (this.tree_handler.reparent_children)(from_hubbub_node(node),
                                              from_hubbub_node(new_parent));
        return ll::OK;
    }

    pub extern fn get_parent(ctx: *mut c_void, node: *mut c_void, element_only: bool, result: *mut *mut c_void)
                      -> ll::Error {
        debug!("ll get parent");

        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        unsafe {
            *result = to_hubbub_node((this.tree_handler.get_parent)(from_hubbub_node(node),
                                                                    element_only));
        }
        return ll::OK;
    }

    pub extern fn has_children(ctx: *mut c_void, node: *mut c_void, result: *mut bool)
            -> ll::Error {
        debug!("ll has children");

        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        unsafe {
            *result = (this.tree_handler.has_children)(from_hubbub_node(node));
        }
        return ll::OK;
    }

    pub extern fn form_associate(ctx: *mut c_void, form: *mut c_void, node: *mut c_void) -> ll::Error {
        debug!("ll form associate");

        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        (this.tree_handler.form_associate)(from_hubbub_node(form), from_hubbub_node(node));
        return ll::OK;
    }

    pub extern fn add_attributes(ctx: *mut c_void,
                                 node: *mut c_void,
                                 attributes: *mut ll::Attribute,
                                 n_attributes: u32)
                              -> ll::Error {
        debug!("ll add attributes");

        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        (this.tree_handler.add_attributes)(from_hubbub_node(node),
                                           from_hubbub_attributes(attributes, n_attributes));
        return ll::OK;
    }

    pub extern fn set_quirks_mode(ctx: *mut c_void, mode: ll::QuirksMode) -> ll::Error {
        debug!("ll set quirks mode");

        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        (this.tree_handler.set_quirks_mode)(from_hubbub_quirks_mode(mode));
        return ll::OK;
    }

    pub extern fn encoding_change(ctx: *mut c_void, encname: *mut c_char) -> ll::Error {
        debug!("ll encoding change");

        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        (this.tree_handler.encoding_change)(unsafe { string::raw::from_buf((&*encname) as *const i8 as *const u8) });
        return ll::OK;
    }

    pub extern fn complete_script(ctx: *mut c_void, script: *mut c_void) -> ll::Error {
        debug!("ll complete script");

        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        (this.tree_handler.complete_script)(from_hubbub_node(script));
        return ll::OK;
    }

    pub extern fn complete_style(ctx: *mut c_void, style: *mut c_void) -> ll::Error {
        debug!("ll complete style");

        let self_opt: &mut Option<TreeHandlerPair> = unsafe { mem::transmute(ctx) };
        let this = self_opt.get_mut_ref();
        (this.tree_handler.complete_style)(from_hubbub_node(style));
        return ll::OK;
    }
}

pub extern fn allocator(ptr: *mut c_void, len: size_t, _pw: *mut c_void) -> *mut c_void {
    unsafe { libc::realloc(ptr, len) }
}

