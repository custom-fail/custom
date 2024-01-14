pub mod macros;
pub mod modals;
pub mod embeds;
pub mod errors;
pub mod avatars;
pub mod uppercase;
pub mod constants;
pub mod message;
pub mod config;

#[macro_export]
macro_rules! all_macro {
  ($attr: meta; $($v:vis mod $__: ident;)*) => {
    $(
        #[$attr]
        $v mod $__;
    )*
  };
  ($attr: meta; $(use $__: path;)*) => {
    $(
        #[$attr]
        use $__;
    )*
  }
}

#[macro_export]
macro_rules! env_unwrap {
    ($name: expr) => {
        std::env::var($name).unwrap_or_else(|_|
            panic!("There is no {} environment viable", $name)
        )
    };
}
