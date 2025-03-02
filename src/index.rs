use std::collections::HashMap;

static TITLE_STRING: &str = "TITLE and AUTHOR";
static END_STRING: &str = "<==End of ";

pub fn index(text: &str) -> HashMap<u32, String> {
    let mut return_value: HashMap<u32, String> = HashMap::new();
    let mut start = false;
    // this bool is to help when the line is longer than 1 line
    let mut in_title = false;

    // initialize some start values
    let mut index = 0;
    let mut title = String::new();
    for line in text.lines() {
        if line.contains(TITLE_STRING) {
            start = true;
            continue;
        }
        if line.starts_with(END_STRING) {
            break;
        }
        if start {
            // we already started parsing this
            if !in_title {
                let parsed = parse_line(line);
                index = parsed.0;
                title = parsed.1;
                in_title = true;
                continue;
            } else {
                // When we already found a title, we need to keep adding to it
                if line.is_empty() {
                    // This is the end of the title
                    in_title = false;
                    return_value.insert(index, title.clone());
                    // Reset title for next entry
                    title = String::new();
                    continue;
                }
                // Add the line to the title
                let line = line.trim();
                title.push('\n');
                title.push_str(line);
            }
        }
    }
    // check if we have a title that was not added
    if !title.is_empty() {
        return_value.insert(index, title);
    }
    return_value
}

// This should only handle one line at a time
fn parse_line(line: &str) -> (u32, String) {
    // Find the last sequence of digits in the line
    let mut number_start = line.len();
    for (i, c) in line.char_indices().rev() {
        if c.is_ascii_digit() {
            number_start = i;
        } else if number_start < line.len() {
            break;
        }
    }

    // Extract the number part
    let number_str = &line[number_start..];
    let number = number_str.trim().parse::<u32>().unwrap_or(0);

    // Extract the title part (trim trailing spaces)
    let title = line[..number_start].trim_end().to_string();

    (number, title)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn test_clean() {
        let input_string =
            "Memoirs of Arsène Lupin, by Maurice Le Blanc                             70719";
        let (index, title) = parse_line(input_string);
        assert_eq!(index, 70719);
        assert_eq!(title, "Memoirs of Arsène Lupin, by Maurice Le Blanc");
        let input_string =
            "The inter ocean curiosity shop for the year 1883, by Various             70718";
        let (index, title) = parse_line(input_string);
        assert_eq!(index, 70718);
        assert_eq!(
            title,
            "The inter ocean curiosity shop for the year 1883, by Various"
        );
    }
    #[test]
    fn test_get_index() {
        let input_string = r#"
TITLE and AUTHOR
Submerged forests, by Clement Reid                                       70654

Rattle of bones, by Robert E. Howard                                     70653
 [Illustrator: Doak]"#;
        println!("input_string: {}", input_string);
        let output = index(input_string);
        println!("output: {:?}", output);
        assert_eq!(output.len(), 2);
        assert_eq!(
            output.get(&70654).unwrap(),
            "Submerged forests, by Clement Reid"
        );
        assert_eq!(
            output.get(&70653).unwrap(),
            r#"Rattle of bones, by Robert E. Howard
[Illustrator: Doak]"#
        );
    }
}
