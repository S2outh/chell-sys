#[macro_export]
macro_rules! match_def {

    ($prim_def:expr, bytes: $bytes:expr, error: $error:expr, { $($rest:tt)* }) => {
        match_def!(@impl $prim_def, $bytes, $error, { $($rest)* })
    };

    ($prim_def:expr, bytes: $bytes:expr, { $($rest:tt)* }) => {
        match_def!(@impl $prim_def, $bytes, {}, { $($rest)* })
    };

    ($prim_def:expr, { $($rest:tt)* }) => {
        match_def!(@impl $prim_def, [], {}, { $($rest)* })
    };

    (@impl $prim_def:expr, $bytes:expr, $error:expr, {
        $($t:path $([$value:ident])? => $body:expr,)+
        $(:default $def:expr)?
    }) => {{
        let any = $prim_def.as_any();
        $(if any.is::<$t>() {
            match_def!(@impl body $bytes, $error, $t $([$value])? => $body)
        })else*
        $(else {
            $def
        })?
    }};
    (@impl body $bytes:expr, $error:expr, $t:path => $body:expr) => {
        $body
    };
    (@impl body $bytes:expr, $error:expr, $t:path [$value:ident] => $body:expr) => {
        if let Ok($value) = $t.deserialize(&$bytes) {
            $body
        } else {
            $error
        }
    };
}
