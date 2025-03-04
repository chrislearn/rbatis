use std::collections::{HashSet, LinkedList};

//find like #{*,*},${*,*} value *
pub fn find_convert_string(arg: &str) -> LinkedList<(String, String)> {
    let mut list = LinkedList::new();
    let mut cache = HashSet::new();
    let chars: Vec<u8> = arg.bytes().collect();
    let mut item = String::new();
    let mut last_index: i32 = -1;
    let mut index: i32 = -1;
    for v in &chars {
        index = index + 1;
        if last_index == -1 && (*v == '#' as u8 || *v == '$' as u8) {
            let next = chars.get(index as usize + 1);
            let next_char = '{' as u8;
            if next.is_some() && next.unwrap().eq(&next_char) {
                last_index = index;
            }
            continue;
        }
        if *v == '}' as u8 && last_index != -1 {
            item = String::from_utf8(chars[(last_index + 2) as usize..index as usize].to_vec())
                .unwrap();
            if cache.get(&item).is_some() {
                item.clear();
                last_index = -1;
                continue;
            }
            let value =
                String::from_utf8(chars[last_index as usize..(index + 1) as usize].to_vec())
                    .unwrap();
            cache.insert(item.clone());
            list.push_back((item.clone(), value));
            item.clear();
            last_index = -1;
        }
    }
    return list;
}

pub fn count_string_num(s: &String, c: char) -> usize {
    let cs = s.chars();
    let mut num = 0;
    for x in cs {
        if x == c {
            num += 1;
        }
    }
    return num;
}

///input 'strings' => strings
pub fn un_packing_string(column: &str) -> &str {
    if column.len() >= 2 {
        if column.starts_with("'") && column.ends_with("'") {
            return &column[1..column.len() - 1];
        }
        if column.starts_with("`") && column.ends_with("`") {
            return &column[1..column.len() - 1];
        }
        if column.starts_with("\"") && column.ends_with("\"") {
            return &column[1..column.len() - 1];
        }
    }
    return column;
}
