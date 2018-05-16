use failure;
use magic::{flags, Cookie, CookieFlags};
use mime::Mime;

use std::path::Path;

thread_local! {
    static COOKIE: Cookie = {
        let mut cflags = CookieFlags::default();
        cflags.insert(flags::MIME_TYPE);
        cflags.insert(flags::SYMLINK);
        cflags.insert(flags::ERROR);
        cflags.insert(flags::NO_CHECK_COMPRESS);
        cflags.insert(flags::NO_CHECK_CDF);
        cflags.insert(flags::NO_CHECK_ENCODING);
        cflags.insert(flags::NO_CHECK_ELF);
        cflags.insert(flags::NO_CHECK_TAR);
        cflags.insert(flags::NO_CHECK_TEXT);
        cflags.insert(flags::NO_CHECK_TOKENS);
        let ret = Cookie::open(cflags).expect("Can't create libmagic cookie");
        // multiple paths not yet implemented
        if ret.load(&["/usr/share/file/misc/magic.mgc"]).is_err() {
            ret.load(&["/usr/share/misc/magic.mgc"]).expect("Can't load libmagic database");
        }
        ret
    }
}

pub fn mime_type_file<P: AsRef<Path>>(path: P) -> Result<Mime, failure::Error> {
    COOKIE.with(move |cookie| Ok(cookie.file(path)?.as_str().parse()?))
}

lazy_static! {
    pub static ref APPLICATION_ZIP: Mime = "application/zip".parse().unwrap();
}
