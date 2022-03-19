use nu_table::{draw_table, StyledString, Table, TextStyle, Theme};
use std::collections::HashMap;

fn f((x, theme): (&str, Theme)) {
    println!("{x}");
    let args: Vec<_> = std::env::args().collect();
    let mut width = 0;

    if args.len() > 1 {
        // Width in terminal characters
        width = args[1].parse::<usize>().expect("Need a width in columns");
    }

    if width < 4 {
        println!("Width must be greater than or equal to 4, setting width to 80");
        width = 80;
    }

    // The mocked up table data
    let (table_headers, row_data) = make_table_data();
    // The table headers
    let headers = vec_of_str_to_vec_of_styledstr(&table_headers, true);
    // The table rows
    let rows = vec_of_str_to_vec_of_styledstr(&row_data, false);
    // The table itself
    let table = Table::new(headers, vec![rows; 3], theme);
    // FIXME: Config isn't available from here so just put these here to compile
    let color_hm: HashMap<String, nu_ansi_term::Style> = HashMap::new();
    // Capture the table as a string
    let output_table = draw_table(&table, width, &color_hm, true);
    // Draw the table
    println!("{}", output_table)
}

fn main() {
    let vec = vec![
        ("basic", Theme::basic()),
        ("thin", Theme::thin()),
        ("light", Theme::light()),
        ("compact", Theme::compact()),
        ("with_love", Theme::with_love()),
        ("compact_double", Theme::compact_double()),
        ("rounded", Theme::rounded()),
        ("reinforced", Theme::reinforced()),
        ("heavy", Theme::heavy()),
        ("none", Theme::none()),
    ];

    for x in vec {
        f(x);
    }
}

fn vec_of_str_to_vec_of_styledstr(data: &[impl ToString], is_header: bool) -> Vec<StyledString> {
    let f = if is_header {
        TextStyle::default_header()
    } else {
        TextStyle::basic_left()
    };

    data.iter()
        .map(|x| StyledString::new(x.to_string(), f))
        .collect()
}

fn make_table_data() -> (Vec<StyledString>, Vec<StyledString>) {
    let table_headers = vec![
        "category",
        "description",
        "emoji",
        "ios_version",
        "unicode_version",
        "aliases",
        "tags",
        "category2",
        "description2",
        "emoji2",
        "ios_version2",
        "unicode_version2",
        "aliases2",
        "tags2",
    ]
    .into_iter()
    .map(|x| StyledString::new(String::from(x), Default::default()))
    .collect();

    let row_data = vec![
        "Smileys & Emotion",
        "grinning face",
        "😀",
        "6",
        "6.1",
        "grinning",
        "smile",
        "Smileys & Emotion",
        "grinning face",
        "😀",
        "6",
        "6.1",
        "grinning",
        "smile",
    ]
    .into_iter()
    .map(|x| StyledString::new(String::from(x), Default::default()))
    .collect();

    (table_headers, row_data)
}
