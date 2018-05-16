macro_rules! ok_or {
    ($test: expr, $on_err: expr) => {
        ok_or!($test, _e => $on_err);
    };
    ($test: expr, $why: ident => $on_err: expr) => {
        match $test {
            Ok(ok) => ok,
            Err($why) => $on_err,
        };
    };
}

macro_rules! ok_or_continue {
    ($test: expr, $why: ident => $on_err: expr) => {
        ok_or!($test, $why => { $on_err; continue; })
    };
}

macro_rules! throw {
    ($err: expr) => {
        Err::<!, _>($err)?
    };
}
