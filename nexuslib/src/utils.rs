/// ### Description
/// Turns a vector of bytes `Vec<u8>` to a 'String'
///
/// There is a (comma) `,` between each element in the `String`
///
/// ### Example
/// The following `Vec<u8>` will be turned to 'String':
///
/// - `Vec<u8>`: `[1, 2, 3, 4, 5]`
/// - `String`: `"1,2,3,4,5"`
pub fn vec_to_string(arr: Vec<u8>) -> String {
    arr.clone()
        .into_iter()
        .map(|b| {
            let mut bn = b.to_string();
            bn.push(',');
            bn
        })
        .collect::<String>()
}

/// ### Description
/// Turns a `String` where each element is separeted by (comma) `,` to a bytes `Vec<u8>`
///
/// ### Example
/// The following `String` will be turned to `Vec<u8>`:
///
/// - `String`: `"1,2,3,4,5"`
/// - `Vec<u8>`: `[1, 2, 3, 4, 5]`
pub fn string_to_vec(string: String) -> Vec<u8> {
    let arr = string.rsplit(',').collect::<Vec<&str>>();
    let arr = arr
        .iter()
        .rev()
        .filter(|s| !s.is_empty())
        .map(|s| s.parse::<u8>().unwrap())
        .collect::<Vec<_>>();

    arr
}
