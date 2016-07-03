use std::collections::HashSet;

lazy_static! {
    pub static ref STOPWORDS: HashSet<&'static str> = include_str!("words/english")
                                                        .lines().map(|l| l.trim()).collect();
}
