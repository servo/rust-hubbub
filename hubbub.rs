// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// High-level bindings to the Hubbub HTML5 parser.

use std::cast;
use std::libc;
use std::libc::{c_void, size_t};
use std::ptr::{null, to_unsafe_ptr};
use std::vec;
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
    name: ~str,
    public_id: Option<~str>,
    system_id: Option<~str>,
    force_quirks: bool
}

pub struct Attribute {
    ns: Ns,
    name: ~str,
    value: ~str,
}

pub struct Tag {
    ns: Ns,
    name: ~str,
    attributes: ~[Attribute],
    self_closing: bool
}

// FIXME: This is terribly type-unsafe. But we don't have working generic extern functions yet...
pub type NodeDataPtr = uint;

pub struct TreeHandler {
    create_comment: ~fn(data: ~str) -> NodeDataPtr,
    create_doctype: ~fn(doctype: ~Doctype) -> NodeDataPtr,
    create_element: ~fn(tag: ~Tag) -> NodeDataPtr,
    create_text: ~fn(data: ~str) -> NodeDataPtr,
    ref_node: ~fn(node: NodeDataPtr),
    unref_node: ~fn(node: NodeDataPtr),
    append_child: ~fn(parent: NodeDataPtr, child: NodeDataPtr) -> NodeDataPtr,
    insert_before: ~fn(parent: NodeDataPtr, child: NodeDataPtr) -> NodeDataPtr,
    remove_child: ~fn(parent: NodeDataPtr, child: NodeDataPtr) -> NodeDataPtr,
    clone_node: ~fn(node: NodeDataPtr, deep: bool) -> NodeDataPtr,
    reparent_children: ~fn(node: NodeDataPtr, new_parent: NodeDataPtr) -> NodeDataPtr,
    get_parent: ~fn(node: NodeDataPtr, element_only: bool) -> NodeDataPtr,
    has_children: ~fn(node: NodeDataPtr) -> bool,
    form_associate: ~fn(form: NodeDataPtr, node: NodeDataPtr),
    add_attributes: ~fn(node: NodeDataPtr, attributes: ~[Attribute]),
    set_quirks_mode: ~fn(mode: QuirksMode),
    encoding_change: ~fn(encname: ~str),
    complete_script: ~fn(script: NodeDataPtr),
    complete_style: ~fn(style: NodeDataPtr),
}

pub struct TreeHandlerPair {
    tree_handler: ~TreeHandler,
    ll_tree_handler: ll::TreeHandler
}

pub struct Parser {
    hubbub_parser: *ll::Parser,
    tree_handler: Option<TreeHandlerPair>,
}

impl Drop for Parser {
	#[fixed_stack_segment]
    fn drop(&self) {
        unsafe { ll::parser::hubbub_parser_destroy(self.hubbub_parser) };
    }
}

#[fixed_stack_segment]
pub fn Parser(encoding: &str, fix_encoding: bool) -> Parser {
    let hubbub_parser = null();
    let hubbub_error = do encoding.to_c_str().with_ref |encoding_c: *libc::c_char| {
        unsafe {
            ll::parser::hubbub_parser_create(cast::transmute(encoding_c), fix_encoding, allocator,
                                             null(), to_unsafe_ptr(&hubbub_parser))
        }
    };
    assert!(hubbub_error == ll::OK);
    return Parser {
        hubbub_parser: hubbub_parser,
        tree_handler: None
    };
}

impl Parser {
	#[fixed_stack_segment]
    pub fn set_tree_handler(&mut self, tree_handler: ~TreeHandler) {
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
                ctx: unsafe { cast::transmute(to_unsafe_ptr(&self.tree_handler)) },
            }
        });

        let ptr: *ll::TreeHandler =
            to_unsafe_ptr(&self.tree_handler.get_ref().ll_tree_handler);

        unsafe {
            let hubbub_error = ll::parser::hubbub_parser_setopt(self.hubbub_parser,
                                                                ll::PARSER_TREE_HANDLER,
                                                                cast::transmute(&ptr));
            assert!(hubbub_error == ll::OK);
        }
    }

	#[fixed_stack_segment]
    pub fn set_document_node(&self, node: NodeDataPtr) {
        unsafe {
            debug!("setting document node");
            let hubbub_error = ll::parser::hubbub_parser_setopt(self.hubbub_parser,
                                                                ll::PARSER_DOCUMENT_NODE,
                                                                cast::transmute(&node));
            assert!(hubbub_error == ll::OK);
        }
    }

	#[fixed_stack_segment]
    pub fn enable_scripting(&self, enable: bool) {
        unsafe {
            debug!("enabling scripting");
            let hubbub_error = ll::parser::hubbub_parser_setopt(self.hubbub_parser,
                                                                ll::PARSER_ENABLE_SCRIPTING,
                                                                cast::transmute(&enable));
            assert!(hubbub_error == ll::OK);
        }
    }

	#[fixed_stack_segment]
    pub fn enable_styling(&self, enable: bool) {
        unsafe {
            debug!("enabling styling");
            let hubbub_error = ll::parser::hubbub_parser_setopt(self.hubbub_parser,
                                                                ll::PARSER_ENABLE_STYLING,
                                                                cast::transmute(&enable));
            assert!(hubbub_error == ll::OK);
        }
    }

	#[fixed_stack_segment]
    pub fn parse_chunk(&self, data: &[u8]) {
        unsafe {
            debug!("parsing chunk");
            let ptr = vec::raw::to_ptr(data);
            let hubbub_error = ll::parser::hubbub_parser_parse_chunk(self.hubbub_parser, ptr,
                                                                     data.len() as size_t);
            assert!(hubbub_error == ll::OK);
        }
    }

	#[fixed_stack_segment]
    pub fn insert_chunk(&self, data: &[u8]) {
        unsafe {
            debug!("inserting chunk");
            let ptr = vec::raw::to_ptr(data);
            let hubbub_error = ll::parser::hubbub_parser_insert_chunk(self.hubbub_parser, ptr,
                                                                      data.len() as size_t);
            assert!(hubbub_error == ll::OK);
        }
    }

	#[fixed_stack_segment]
    pub fn completed(&self) {
        unsafe {
            debug!("completing");
            let hubbub_error = ll::parser::hubbub_parser_completed(self.hubbub_parser);
            assert!(hubbub_error == ll::OK);
        }
    }
}

pub mod tree_callbacks {

    use std::cast;
    use std::libc::{c_void, c_char};
    use std::ptr::offset;
    use std::str;
    use std::vec;
    use super::{NodeDataPtr, Ns, NullNs, HtmlNs, MathMlNs, SvgNs, XLinkNs, XmlNs, XmlNsNs};
    use super::{QuirksMode, NoQuirks, LimitedQuirks, FullQuirks};
    use super::{Attribute, Tag, Doctype, TreeHandlerPair};
    use ll;

    // Data conversions

    pub fn from_hubbub_node(node: *c_void) -> NodeDataPtr {
        unsafe { cast::transmute(node) }
    }

    pub fn from_hubbub_string(string: &ll::String) -> ~str {
        unsafe {
            debug!("from_hubbub_string: %u", (*string).len as uint);
            let s = str::raw::from_buf_len((*string).ptr, (*string).len as uint);
            debug!("from_hubbub_string: %s", s);
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
            _ => fail!(~"unknown namespace")
        }
    }

    pub fn from_hubbub_quirks_mode(mode: ll::QuirksMode) -> QuirksMode {
        match mode {
            0 => NoQuirks,
            1 => LimitedQuirks,
            2 => FullQuirks,
            _ => fail!(~"unknown quirks mode")
        }
    }

    pub fn from_hubbub_attributes(attributes: *ll::Attribute, n_attributes: u32) -> ~[Attribute] {
        debug!("from_hubbub_attributes n=%u", n_attributes as uint);
        unsafe {
            do vec::from_fn(n_attributes as uint) |i| {
                let attribute = offset(attributes, i as int);
                Attribute {
                    ns: from_hubbub_ns((*attribute).ns),
                    name: from_hubbub_string(&(*attribute).name),
                    value: from_hubbub_string(&(*attribute).value)
                }
            }
        }
    }

    pub fn from_hubbub_tag(tag: &ll::Tag) -> ~Tag {
        ~Tag {
            ns: from_hubbub_ns((*tag).ns),
            name: from_hubbub_string(&(*tag).name),
            attributes: from_hubbub_attributes((*tag).attributes, (*tag).n_attributes),
            self_closing: (*tag).self_closing
        }
    }

    pub fn from_hubbub_doctype(doctype: &ll::Doctype) -> ~Doctype {
        ~Doctype {
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

    pub fn to_hubbub_node(node: NodeDataPtr) -> *c_void {
        unsafe { cast::transmute(node) }
    }

    // Callbacks

    pub extern fn create_comment(ctx: *c_void, data: *ll::String, result: *mut *c_void)
                          -> ll::Error {
        debug!("ll create comment");
        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        unsafe {
            let data: &ll::String = cast::transmute(data);
            *result = to_hubbub_node((this.tree_handler.create_comment)(from_hubbub_string(data)));
        }
        return ll::OK;
    }

    pub extern fn create_doctype(ctx: *c_void, doctype: *ll::Doctype, result: *mut *c_void)
                          -> ll::Error {
        debug!("ll create doctype");
        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        unsafe {
            let doctype: &ll::Doctype = cast::transmute(doctype);
            *result = to_hubbub_node((this.tree_handler.create_doctype)(from_hubbub_doctype(doctype)));
        }
        return ll::OK;
    }

    pub extern fn create_element(ctx: *c_void, tag: *ll::Tag, result: *mut *c_void)
                          -> ll::Error {
        debug!("ll create element");
        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        unsafe {
            let tag: &ll::Tag = cast::transmute(tag);
            *result = to_hubbub_node((this.tree_handler.create_element)(from_hubbub_tag(tag)));
        }
        return ll::OK;
    }

    pub extern fn create_text(ctx: *c_void, data: *ll::String, result: *mut *c_void)
                       -> ll::Error {
        debug!("ll create text");
        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        unsafe {
            let data: &ll::String = cast::transmute(data);
            *result = to_hubbub_node((this.tree_handler.create_text)(from_hubbub_string(data)));
        }
        return ll::OK;
    }

    pub extern fn ref_node(ctx: *c_void, node: *c_void) -> ll::Error {
        debug!("ll ref node");
        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        (this.tree_handler.ref_node)(from_hubbub_node(node));
        return ll::OK;
    }

    pub extern fn unref_node(ctx: *c_void, node: *c_void) -> ll::Error {
        debug!("ll unref node");
        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        (this.tree_handler.unref_node)(from_hubbub_node(node));
        return ll::OK;
    }

    pub extern fn append_child(ctx: *c_void, parent: *c_void, child: *c_void, result: *mut *c_void)
                        -> ll::Error {
        debug!("ll append child");
        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        unsafe {
            *result = to_hubbub_node((this.tree_handler.append_child)(from_hubbub_node(parent),
                                                                      from_hubbub_node(child)));
        }
        return ll::OK;
    }

    pub extern fn insert_before(ctx: *c_void, parent: *c_void, child: *c_void,
                                result: *mut *c_void) -> ll::Error {
        debug!("ll insert before");
        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        unsafe {
            *result = to_hubbub_node((this.tree_handler.insert_before)(from_hubbub_node(parent),
                                                                       from_hubbub_node(child)));
        }
        return ll::OK;
    }

    pub extern fn remove_child(ctx: *c_void, parent: *c_void, child: *c_void, result: *mut *c_void)
                        -> ll::Error {
        debug!("ll remove child");

        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        unsafe {
            *result = to_hubbub_node((this.tree_handler.remove_child)(from_hubbub_node(parent),
                                                                      from_hubbub_node(child)));
        }
        return ll::OK;
    }

    pub extern fn clone_node(ctx: *c_void, node: *c_void, deep: bool, result: *mut *c_void)
                      -> ll::Error {
        debug!("ll clone node");

        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        unsafe {
            *result = to_hubbub_node((this.tree_handler.clone_node)(from_hubbub_node(node), deep));
        }
        return ll::OK;
    }

    pub extern fn reparent_children(ctx: *c_void, node: *c_void, new_parent: *c_void)
                             -> ll::Error {
        debug!("ll reparent children");

        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        (this.tree_handler.reparent_children)(from_hubbub_node(node),
                                              from_hubbub_node(new_parent));
        return ll::OK;
    }

    pub extern fn get_parent(ctx: *c_void, node: *c_void, element_only: bool, result: *mut *c_void)
                      -> ll::Error {
        debug!("ll get parent");

        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        unsafe {
            *result = to_hubbub_node((this.tree_handler.get_parent)(from_hubbub_node(node),
                                                                    element_only));
        }
        return ll::OK;
    }

    pub extern fn has_children(ctx: *c_void, node: *c_void, result: *mut bool)
            -> ll::Error {
        debug!("ll has children");

        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        unsafe {
            *result = (this.tree_handler.has_children)(from_hubbub_node(node));
        }
        return ll::OK;
    }

    pub extern fn form_associate(ctx: *c_void, form: *c_void, node: *c_void) -> ll::Error {
        debug!("ll form associate");

        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        (this.tree_handler.form_associate)(from_hubbub_node(form), from_hubbub_node(node));
        return ll::OK;
    }

    pub extern fn add_attributes(ctx: *c_void,
                                 node: *c_void,
                                 attributes: *ll::Attribute,
                                 n_attributes: u32)
                              -> ll::Error {
        debug!("ll add attributes");

        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        (this.tree_handler.add_attributes)(from_hubbub_node(node),
                                           from_hubbub_attributes(attributes, n_attributes));
        return ll::OK;
    }

    pub extern fn set_quirks_mode(ctx: *c_void, mode: ll::QuirksMode) -> ll::Error {
        debug!("ll set quirks mode");

        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        (this.tree_handler.set_quirks_mode)(from_hubbub_quirks_mode(mode));
        return ll::OK;
    }

    pub extern fn encoding_change(ctx: *c_void, encname: *c_char) -> ll::Error {
        debug!("ll encoding change");

        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        (this.tree_handler.encoding_change)(unsafe { str::raw::from_c_str(encname) });
        return ll::OK;
    }

    pub extern fn complete_script(ctx: *c_void, script: *c_void) -> ll::Error {
        debug!("ll complete script");

        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        (this.tree_handler.complete_script)(from_hubbub_node(script));
        return ll::OK;
    }

    pub extern fn complete_style(ctx: *c_void, style: *c_void) -> ll::Error {
        debug!("ll complete style");

        let self_opt: &Option<TreeHandlerPair> = unsafe { cast::transmute(ctx) };
        let this = self_opt.get_ref();
        (this.tree_handler.complete_style)(from_hubbub_node(style));
        return ll::OK;
    }
}

pub extern fn allocator(ptr: *mut c_void, len: size_t, _pw: *c_void) -> *mut c_void {
    unsafe { libc::realloc(ptr, len) }
}

