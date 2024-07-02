use lazy_static::lazy_static;
use regex::Regex;

const URL_RE_: &str = r"https?://(www\.)?[-a-zA-Z0-9@:%._\+~#=]{2,256}(\.[a-z]{2,4})?\b([-a-zA-Z0-9@:%_\+.~#?&//=]*)";

lazy_static! {
    static ref URL_RE: Regex = Regex::new(URL_RE_).unwrap();    
}

/** validate if a string is a valid url */
pub fn is_url(v: &String) -> bool {
    URL_RE.is_match(v.as_str())
}

pub fn is_path(path_: &String) -> bool {
    let path = std::path::Path::new(path_);

    path.is_file()
}
