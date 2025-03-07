pub trait Slug {
    fn to_slug(&self) -> String;
    fn slugify(text: &str) -> String {
        let mut result = String::new();

        // Convert to lowercase and process each character
        for c in text.to_lowercase().chars() {
            match c {
                // Keep alphanumeric characters
                'a'..='z' | '0'..='9' => result.push(c),

                // Replace spaces and special characters with hyphens
                ' ' | '_' | '-' => {
                    // Only add hyphen if last char wasn't a hyphen
                    if !result.is_empty() && result.chars().last() != Some('-') {
                        result.push('-');
                    }
                }

                // Convert accented characters to base letters
                'á' | 'à' | 'ã' | 'â' | 'ä' => result.push('a'),
                'é' | 'è' | 'ê' | 'ë' => result.push('e'),
                'í' | 'ì' | 'î' | 'ï' => result.push('i'),
                'ó' | 'ò' | 'õ' | 'ô' | 'ö' => result.push('o'),
                'ú' | 'ù' | 'û' | 'ü' => result.push('u'),
                'ñ' => result.push('n'),

                _ => {}
            }
        }

        // Remove trailing hyphens
        while result.ends_with('-') {
            result.pop();
        }

        result
    }
}
impl Slug for &'_ str {
    fn to_slug(&self) -> String {
        Self::slugify(self)
    }
}
impl Slug for String {
    fn to_slug(&self) -> String {
        Self::slugify(self)
    }
}

#[cfg(test)]
mod tests {
    use core::str;

    use super::*;

    #[test]
    fn test_slug() {
        let str = "Hello World!";
        assert_eq!(Slug::to_slug(&str), "hello-world");
    }
}
