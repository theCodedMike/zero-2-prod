/// Returns true if all chars of this string are White_Space.
///
/// White_Space is specified in the Unicode Character Database:
/// [White_Space](https://www.unicode.org/Public/UCD/latest/ucd/PropList.txt)
pub fn is_blank(str: &str) -> bool {
    str.chars().all(|item| item.is_whitespace())
}

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn test_white_space() {
        let empty_str1 = "   ";
        assert!(super::is_blank(empty_str1));
        let empty_str2 = "\n\n";
        assert!(super::is_blank(empty_str2));
        let empty_str3 = "\t\t";
        assert!(super::is_blank(empty_str3));
    }
}
