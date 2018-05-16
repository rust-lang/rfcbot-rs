macro_rules! throw {
    ($err: expr) => {
        Err::<!, _>($err)?
    };
}
