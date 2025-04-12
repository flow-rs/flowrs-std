pub mod nodes;

use wasm_bindgen::prelude::wasm_bindgen;

pub use self::nodes::debug;
// pub use self::nodes::timer;
pub use self::nodes::value;

pub use self::nodes::binops::add;
// pub use self::nodes::binops::div;
// pub use self::nodes::binops::mul;
// pub use self::nodes::binops::sub;
// pub use self::nodes::binops::and;
// pub use self::nodes::binops::or;

// pub use self::nodes::control::switch;
// pub use self::nodes::io::file;
// pub use self::nodes::transform::binary;
// pub use self::nodes::transform::json;
// pub use self::nodes::transform::vec;

// Required for debug node
#[wasm_bindgen]
extern "C" {
    fn alert(s: &str);
    #[wasm_bindgen(js_namespace = console)]
    fn log(s: &str);
}
