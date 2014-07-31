// Copyright 2013 The Servo Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use libc::{c_int, c_void, size_t, c_char};

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
#[link(name = "hubbub", kind = "static")]
#[link(name = "parserutils", kind = "static")]
#[link(name = "iconv")]
extern { }

#[cfg(target_os = "linux")]
#[cfg(target_os = "android")]
#[link(name = "hubbub", kind = "static")]
#[link(name = "parserutils", kind = "static")]
extern { }

extern {
    pub fn hubbub_error_to_string(error: Error) -> *mut u8;
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
pub static PARSER_ENABLE_STYLING: c_int = 7;

pub struct ParserOptParamsTokenHandler {
    pub handler: *mut u8,
    pub pw: *mut c_void
}

pub struct ParserOptParamsErrorHandler {
    pub handler: *mut u8,
    pub pw: *mut c_void
}

pub struct ParserOptParamsContentModel {
    pub content_model: ContentModel
}

pub mod parser {
    use libc::{c_void, size_t, c_char};
    use super::{Parser, Error, ParserOptType, CharsetSource};

    extern {
        pub fn hubbub_parser_create(enc: *const u8,
                                    fix_enc: bool,
                                    alloc: extern "C" fn(*mut c_void, size_t, *mut c_void) -> *mut c_void,
                                    pw: *mut c_void,
                                    parser: *mut *mut Parser)
            -> Error;
        pub fn hubbub_parser_destroy(parser: *mut Parser) -> Error;
        pub fn hubbub_parser_setopt(parser: *mut Parser, opt_type: ParserOptType, params: *mut c_void)
            -> Error;
        pub fn hubbub_parser_parse_chunk(parser: *mut Parser, data: *const u8, len: size_t) -> Error;
        pub fn hubbub_parser_insert_chunk(parser: *mut Parser, data: *const u8, len: size_t) -> Error;
        pub fn hubbub_parser_completed(parser: *mut Parser) -> Error;
        pub fn hubbub_parser_read_charset(parser: *mut Parser, source: *mut CharsetSource) -> *const c_char;
    }
}

// tree.h

pub struct TreeHandler {
    pub create_comment: extern "C" fn(*mut c_void, *mut String, *mut *mut c_void) -> Error,
    pub create_doctype: extern "C" fn(*mut c_void, *mut Doctype, *mut *mut c_void) -> Error,
    pub create_element: extern "C" fn(*mut c_void, *mut Tag, *mut *mut c_void) -> Error,
    pub create_text: extern "C" fn(*mut c_void, *mut String, *mut *mut c_void) -> Error,
    pub ref_node: extern "C" fn(*mut c_void, *mut c_void) -> Error,
    pub unref_node: extern "C" fn(*mut c_void, *mut c_void) -> Error,
    pub append_child: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut *mut c_void) -> Error,
    pub insert_before: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut *mut c_void) -> Error,
    pub remove_child: extern "C" fn(*mut c_void, *mut c_void, *mut c_void, *mut *mut c_void) -> Error,
    pub clone_node: extern "C" fn(*mut c_void, *mut c_void, bool, *mut *mut c_void) -> Error,
    pub reparent_children: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> Error,
    pub get_parent: extern "C" fn(*mut c_void, *mut c_void, bool, *mut *mut c_void) -> Error,
    pub has_children: extern "C" fn(*mut c_void, *mut c_void, *mut bool) -> Error,
    pub form_associate: extern "C" fn(*mut c_void, *mut c_void, *mut c_void) -> Error,
    pub add_attributes: extern "C" fn(*mut c_void, *mut c_void, *mut Attribute, u32) -> Error,
    pub set_quirks_mode: extern "C" fn(*mut c_void, QuirksMode) -> Error,
    pub encoding_change: extern "C" fn(*mut c_void, *mut c_char) -> Error,
    pub complete_script: extern "C" fn(*mut c_void, *mut c_void) -> Error,
    pub complete_style: extern "C" fn(*mut c_void, *mut c_void) -> Error,
    pub ctx: *mut c_void,
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
    pub ptr: *mut u8,
    pub len: size_t
}

pub struct Attribute {
    pub ns: NS,
    pub name: String,
    pub value: String,
}

pub struct Doctype {
    pub name: String,
    pub public_missing: bool,
    pub public_id: String,
    pub system_missing: bool,
    pub system_id: String,
    pub force_quirks: bool,
}

pub struct Tag {
    pub ns: NS,
    pub name: String,
    pub n_attributes: u32,
    pub attributes: *mut Attribute,
    pub self_closing: bool,
}

// Token data
pub struct Token {
    pub token_type: TokenType,
    pub data: u8,   // union: one of Doctype, Tag, Comment (string), or Character (string)
}

