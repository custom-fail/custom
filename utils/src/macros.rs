#[macro_export]
macro_rules! ok_or_return {
    ($value: expr, $type: path) => {
        if let $type(value) = $value {
            value
        } else { return }
    };
}

#[macro_export]
macro_rules! ok_or_skip {
    ($value: expr, $type: path) => {
        if let $type(value) = $value {
            value.clone()
        } else { continue }
    };
}

#[macro_export]
macro_rules! ok_or_break {
    ($value: expr, $type: path) => {
        if let $type(value) = $value {
            value.clone()
        } else { break }
    };
}

#[macro_export]
macro_rules! ok_or_break_without_clone {
    ($value: expr, $type: path) => {
        if let $type(value) = $value {
            value
        } else { break }
    };
}

#[macro_export]
macro_rules! ok_or_skip_without_clone {
    ($value: expr, $type: path) => {
        if let $type(value) = $value {
            value
        } else { continue }
    };
}

#[macro_export]
macro_rules! check_type {
    ($value: expr, $type: path) => {
        match $value {
            $type(v) => Some(v),
            _ => None
        }
    }
}
