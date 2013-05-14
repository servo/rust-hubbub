// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use core::libc::{c_char, c_int, c_void, size_t};

// errors.h

pub type Error = c_int;
pub static OK: c_int = 0;
pub static REPROCESS: c_int = 1;
pub static ENCODINGCHANGE: c_int = 2;
pub static PAUSED: c_int = 3;
pub static NOMEM: c_int = 5;
pub static BADPARM: c_int = 6;
pub static INVALID: c_int = 7;
pub static FILENOTFOUND: c_int = 8;
pub static NEEDDATA: c_int = 9;
pub static BADENCODING: c_int = 10;
pub static UNKNOWN: c_int = 11;

#[cfg(target_os = "macos")]
#[nolink]
#[link_args="-L../../hubbub/libhubbub -lhubbub -L../../libparserutils/libparserutils -lparserutils -liconv"]
pub extern mod linking { }

#[cfg(target_os = "linux")]
#[nolink]
#[link_args="-L../../hubbub/libhubbub -lhubbub -L../../libparserutils/libparserutils -lparserutils"]
pub extern mod linking { }

#[nolink]
pub extern mod error {
    pub fn hubbub_error_to_string(error: Error) -> *u8;
}

// parser.h

pub type Parser = c_void;

pub type ParserOptType = c_int;
pub static PARSER_TOKEN_HANDLER: c_int = 0;
pub static PARSER_ERROR_HANDLER: c_int = 1;
pub static PARSER_CONTENT_MODEL: c_int = 2;
pub static PARSER_TREE_HANDLER: c_int = 3;
pub static PARSER_DOCUMENT_NODE: c_int = 4;
pub static PARSER_ENABLE_SCRIPTING: c_int = 5;
pub static PARSER_PAUSE: c_int = 6;

pub struct ParserOptParamsTokenHandler {
    handler: *u8,
    pw: *c_void
}

pub struct ParserOptParamsErrorHandler {
    handler: *u8,
    pw: *c_void
}

pub struct ParserOptParamsContentModel {
    content_model: ContentModel
}

#[nolink]
pub extern mod parser {
    pub fn hubbub_parser_create(enc: *u8,
                                fix_enc: bool,
                                alloc: *u8,
                                pw: *c_void,
                                parser: **Parser)
                             -> Error;
    pub fn hubbub_parser_destroy(parser: *Parser) -> Error;
    pub fn hubbub_parser_setopt(parser: *Parser, opt_type: ParserOptType, params: *c_void)
                             -> Error;
    pub fn hubbub_parser_parse_chunk(parser: *Parser, data: *u8, len: size_t) -> Error;
    pub fn hubbub_parser_insert_chunk(parser: *Parser, data: *u8, len: size_t) -> Error;
    pub fn hubbub_parser_completed(parser: *Parser) -> Error;
    pub fn hubbub_parser_read_charset(parser: *Parser, source: *CharsetSource) -> *c_char;
}

// tree.h

pub struct TreeHandler {
    create_comment: *u8,
    create_doctype: *u8,
    create_element: *u8,
    create_text: *u8,
    ref_node: *u8,
    unref_node: *u8,
    append_child: *u8,
    insert_before: *u8,
    remove_child: *u8,
    clone_node: *u8,
    reparent_children: *u8,
    get_parent: *u8,
    has_children: *u8,
    form_associate: *u8,
    add_attributes: *u8,
    set_quirks_mode: *u8,
    encoding_change: *u8,
    complete_script: *u8,
    ctx: *c_void
}

// types.h

// Source of charset information, in order of importance.
// A client-dictated charset will override all others.
// A document-specified charset will override autodetection or the default.
pub type CharsetSource = c_int;
pub static CHARSET_UNKNOWN: c_int = 0;
pub static CHARSET_TENTATIVE: c_int = 1;
pub static CHARSET_CONFIDENT: c_int = 2;

// Content model flag
pub type ContentModel = c_int;
pub static CONTENT_MODEL_PCDATA: c_int = 0;
pub static CONTENT_MODEL_RCDATA: c_int = 1;
pub static CONTENT_MODEL_CDATA: c_int = 2;
pub static CONTENT_MODEL_PLAINTEXT: c_int = 3;

// Quirks mode flag
pub type QuirksMode = c_int;
pub static QUIRKS_MODE_NONE: c_int = 0;
pub static QUIRKS_MODE_LIMITED: c_int = 1;
pub static QUIRKS_MODE_FULL: c_int = 2;

pub type TokenType = c_int;
pub static TOKEN_DOCTYPE: c_int = 0;
pub static TOKEN_START_TAG: c_int = 1;
pub static TOKEN_END_TAG: c_int = 2;
pub static TOKEN_COMMENT: c_int = 3;
pub static TOKEN_CHARACTER: c_int = 4;
pub static TOKEN_EOF: c_int = 5;

pub type NS = c_int;
pub static NS_NULL: c_int = 0;
pub static NS_HTML: c_int = 1;
pub static NS_MATHML: c_int = 2;
pub static NS_SVG: c_int = 3;
pub static NS_XLINK: c_int = 4;
pub static NS_XML: c_int = 5;
pub static NS_XMLNS: c_int = 6;

pub struct String {
    ptr: *u8,
    len: size_t
}

pub struct Attribute {
    ns: NS,
    name: String,
    value: String,
}

pub struct Doctype {
    name: String,
    public_missing: bool,
    public_id: String,
    system_missing: bool,
    system_id: String,
    force_quirks: bool,
}

pub struct Tag {
    ns: NS,
    name: String,
    n_attributes: u32,
    attributes: *Attribute,
    self_closing: bool,
}

// Token data
pub struct Token {
    token_type: TokenType,
    data: u8,   // union: one of Doctype, Tag, Comment (string), or Character (string)
}

