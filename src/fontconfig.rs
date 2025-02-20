// The MIT License (MIT)
// Copyright (c) 2016 font-loader Developers
//
// Permission is hereby granted, free of charge, to any person obtaining a copy of this software and
// associated documentation files (the "Software"), to deal in the Software without restriction,
// including without limitation the rights to use, copy, modify, merge, publish, distribute,
// sublicense, and/or sell copies of the Software, and to permit persons to whom the Software is
// furnished to do so, subject to the following conditions:
//
// The above copyright notice and this permission notice shall be included in all copies or
// substantial portions of the Software.
//
// THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT
// NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR PURPOSE AND
// NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM,
// DAMAGES OR OTHER LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
// OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.

/// Font loading utilities for installed system fonts
pub mod system_fonts {
    use servo_fontconfig::fontconfig::FcPatternAddString;
    use servo_fontconfig::fontconfig::{
        FcChar8, FcDefaultSubstitute, FcFontList, FcObjectSetBuild,
    };
    use servo_fontconfig::fontconfig::{FcConfig, FcInitLoadConfigAndFonts, FcNameParse};
    use servo_fontconfig::fontconfig::{FcConfigGetFontDirs, FcStrList};
    use servo_fontconfig::fontconfig::{
        FcConfigSubstitute, FcMatchPattern, FcResultMatch, FcResultNoMatch,
    };
    use servo_fontconfig::fontconfig::{
        FcFontMatch,
        FcPattern,
        FcPatternCreate, // , FcPatternDestroy
    };
    use servo_fontconfig::fontconfig::{
        FcPatternAddInteger,
        // FcPatternGetInteger,
        FcPatternGetString,
        FcStrListDone,
        FcStrListNext,
        FcStrSet,
    };

    use libc::{c_char, c_int};

    use std::ffi::{CStr, CString};
    use std::ptr;
    use std::slice;

    use std::sync::Once;

    static FC_FAMILY: &'static [u8] = b"family\0";
    // static FC_FILE: &'static [u8] = b"file\0";
    static FC_WEIGHT: &'static [u8] = b"weight\0";
    // static FC_INDEX: &'static [u8] = b"index\0";
    static FC_SLANT: &'static [u8] = b"slant\0";
    static FC_SPACING: &'static [u8] = b"spacing\0";
    //  static FC_FONTFORMAT: &'static [u8] = b"fontformat\0";
    // 	static FC_STYLE: &'static [u8] = b"style\0";
    // 	static FC_FAMILYLANG: &'static[u8] = b"familylang\0";
    // 	static FC_CHARSET: &'static [u8] = b"charset\0";
    // 	static FC_LANG: &'static [u8] = b"lang\0";
    // 	static FC_STYLELANG: &'static [u8] = b"stylelang\0";

    // 	static FC_WEIGHT_THIN: c_int 		= 0;
    // 	static FC_WEIGHT_EXTRALIGHT: c_int 	= 40;
    // 	static FC_WEIGHT_LIGHT: c_int 		= 50;
    // 	static FC_WEIGHT_DEMILIGHT: c_int 	= 55;
    // 	static FC_WEIGHT_BOOK: c_int 		= 75;
    static FC_WEIGHT_REGULAR: c_int = 80;
    // 	static FC_WEIGHT_MEDIUM: c_int 		= 100;
    // 	static FC_WEIGHT_DEMIBOLD: c_int 	= 180;
    static FC_WEIGHT_BOLD: c_int = 200;
    // 	static FC_WEIGHT_EXTRABOLD: c_int 	= 205;
    // 	static FC_WEIGHT_BLACK: c_int 		= 210;
    // 	static FC_WEIGHT_EXTRA_BLACK: c_int = 215;

    static FC_SLANT_ROMAN: c_int = 0;
    static FC_SLANT_ITALIC: c_int = 100;
    static FC_SLANT_OBLIQUE: c_int = 110;

    //    static FC_PROPORTIONAL: c_int = 0;
    // 	static FC_DUAL: c_int = 90;
    static FC_MONO: c_int = 100;
    // 	static FC_CHARCELL: c_int = 110;

    static INIT_FONTCONFIG: Once = Once::new();
    static mut CONFIG: *mut FcConfig = 0 as *mut FcConfig;

    fn init() -> *mut FcConfig {
        unsafe {
            INIT_FONTCONFIG.call_once(|| {
                CONFIG = FcInitLoadConfigAndFonts();
            });
            CONFIG
        }
    }

    /// The platform specific font properties
    pub struct FontProperty {
        slant: c_int,
        weight: c_int,
        family: String,
        spacing: Option<c_int>,
    }

    /// Builder for FontProperty
    pub struct FontPropertyBuilder {
        property: FontProperty,
    }

    impl FontPropertyBuilder {
        pub fn new() -> FontPropertyBuilder {
            let property = FontProperty {
                slant: FC_SLANT_ROMAN,
                weight: FC_WEIGHT_REGULAR,
                family: String::new(),
                spacing: None,
            };
            FontPropertyBuilder { property: property }
        }

        pub fn italic(mut self) -> FontPropertyBuilder {
            self.property.slant = FC_SLANT_ITALIC;
            self
        }

        pub fn oblique(mut self) -> FontPropertyBuilder {
            self.property.slant = FC_SLANT_OBLIQUE;
            self
        }

        pub fn bold(mut self) -> FontPropertyBuilder {
            self.property.weight = FC_WEIGHT_BOLD;
            self
        }

        pub fn monospace(mut self) -> FontPropertyBuilder {
            self.property.spacing = Some(FC_MONO);
            self
        }

        pub fn family(mut self, name: &str) -> FontPropertyBuilder {
            self.property.family.clear();
            self.property.family.push_str(name);
            self
        }

        pub fn build(self) -> FontProperty {
            self.property
        }
    }

    pub fn family_name(family: &str) -> Option<String> {
        let config = init();

        unsafe {
            let name = CString::new(family).unwrap();
            let pat = FcNameParse(name.as_ptr() as *const FcChar8);
            // add_int(pat, FC_SLANT, property.slant);
            // add_int(pat, FC_WEIGHT, property.weight);
            FcConfigSubstitute(config, pat, FcMatchPattern);
            FcDefaultSubstitute(pat);

            let mut result = FcResultNoMatch;
            let font_pat = FcFontMatch(config, pat, &mut result);

            if font_pat.is_null() {
                None
            } else {
                let family = get_string(font_pat, FC_FAMILY).unwrap();
                Some(family)
            }
        }
    }

    /// Query the names of all fonts installed in the system
    /// Note that only truetype fonts are supported
    pub fn query_all() -> Vec<String> {
        let mut property = FontPropertyBuilder::new().build();
        query_specific(&mut property)
    }

    /// Query the names of specifc fonts installed in the system
    /// Note that only truetype fonts are supported
    pub fn query_specific(property: &mut FontProperty) -> Vec<String> {
        let mut fonts: Vec<String> = Vec::new();
        unsafe {
            let config = init();

            let pattern = FcPatternCreate();
            if !property.family.is_empty() {
                add_string(pattern, FC_FAMILY, &property.family);
            }
            property
                .spacing
                .map(|spacing| add_int(pattern, FC_SPACING, spacing));
            add_int(pattern, FC_WEIGHT, property.weight);
            add_int(pattern, FC_SLANT, property.slant);

            let null_ptr: *const c_char = ptr::null();
            let o1 = FC_FAMILY.as_ptr() as *mut c_char;
            let os = FcObjectSetBuild(o1, null_ptr);
            let fs = FcFontList(config, pattern, os);

            let patterns = slice::from_raw_parts((*fs).fonts, (*fs).nfont as usize);
            for pat in patterns {
                let family_name = get_string(*pat, FC_FAMILY).unwrap();
                fonts.push(family_name);
            }
        }

        fonts.sort();
        fonts.dedup();
        fonts
    }

    fn add_int(pat: *mut FcPattern, object_name: &[u8], value: c_int) {
        let object = object_name.as_ptr() as *const c_char;
        unsafe {
            FcPatternAddInteger(pat, object, value);
        }
    }

    // fn get_int(pat: *mut FcPattern, object_name: &[u8]) -> Result<c_int, &str> {
    //     let object = object_name.as_ptr() as *const c_char;
    //     unsafe {
    //         let mut int: c_int = 0;
    //         if FcPatternGetInteger(pat, object, 0, &mut int) == FcResultMatch {
    //             Ok(int)
    //         } else {
    //             Err("Type didn't match")
    //         }
    //     }
    // }

    fn add_string(pat: *mut FcPattern, object_name: &[u8], value: &str) {
        let value = CString::new(value).unwrap();
        let value_ptr = value.as_ptr() as *const FcChar8;
        let object = object_name.as_ptr() as *const c_char;
        unsafe {
            FcPatternAddString(pat, object, value_ptr);
        }
    }

    fn get_string(pat: *mut FcPattern, object_name: &[u8]) -> Result<String, &str> {
        unsafe {
            let mut string: *mut FcChar8 = ptr::null_mut();
            let object = object_name.as_ptr() as *const c_char;
            if FcPatternGetString(pat, object, 0, &mut string) == FcResultMatch {
                let cstr = CStr::from_ptr(string as *mut c_char);
                let string = cstr.to_string_lossy().into_owned();
                Ok(string)
            } else {
                Err("Type didn't match")
            }
        }
    }

    pub fn get_font_dirs() -> Vec<String> {
        let config = init();
        let mut dirs = Vec::new();
        unsafe {
            let lang_strs: *mut FcStrList = FcConfigGetFontDirs(config);
            for dir in StrList::from_raw(lang_strs) {
                dirs.push(dir.to_string());
            }
        }
        dirs
    }

    // code borrowed here
    // https://github.com/yeslogic/fontconfig-rs/blob/v0.6.0/fontconfig/src/lib.rs#L408
    pub struct StrList<'a> {
        list: *mut FcStrList,
        _life: std::marker::PhantomData<&'a FcStrList>,
    }

    impl<'a> StrList<'a> {
        unsafe fn from_raw(raw_list: *mut FcStrSet) -> Self {
            Self {
                list: raw_list,
                _life: std::marker::PhantomData,
            }
        }
    }

    impl<'a> Drop for StrList<'a> {
        fn drop(&mut self) {
            unsafe { FcStrListDone(self.list) };
        }
    }

    impl<'a> Iterator for StrList<'a> {
        type Item = &'a str;

        fn next(&mut self) -> Option<&'a str> {
            let lang_str: *mut FcChar8 = unsafe { FcStrListNext(self.list) };
            if lang_str.is_null() {
                None
            } else {
                match unsafe { CStr::from_ptr(lang_str as *const c_char) }.to_str() {
                    Ok(s) => Some(s),
                    _ => self.next(),
                }
            }
        }
    }
}
