use crate::BarcodeBarArray;

const BARCODE_DICT_POS : [[usize;5];4] = [[0,4,7,9,10],[10,13,15,16,16],[16,18,19,19,19],[19,20,20,20,20]];

const BARCODE_DICT : [(u8,[u8;4],bool);20] = [
        (6, [1, 1, 1, 4], true),
        (0, [1, 1, 2, 3], false),
        (4, [1, 1, 3, 2], true),
        (3, [1, 1, 4, 1], false),
        (8, [1, 2, 1, 3], true),
        (1, [1, 2, 2, 2], false),
        (5, [1, 2, 3, 1], true),
        (7, [1, 3, 1, 2], true),
        (5, [1, 3, 2, 1], false),
        (3, [1, 4, 1, 1], true),
        (9, [2, 1, 1, 3], false),
        (2, [2, 1, 2, 2], true),
        (7, [2, 1, 3, 1], false),
        (2, [2, 2, 1, 2], false),
        (1, [2, 2, 2, 1], true),
        (4, [2, 3, 1, 1], false),
        (9, [3, 1, 1, 2], true),
        (8, [3, 1, 2, 1], false),
        (0, [3, 2, 1, 1], true),
        (6, [4, 1, 1, 1], false),
];

const EAN_PARITY : [(usize,usize);29] = [(1,10),(2,3),(0,0),(4,5),(0,1),(6,7),(0,2),(7,8),(0,3),
    (0,10),(11,18),(12,13),(0,4),(14,15),(0,7),(16,17),(0,8),(0,10),(19,24),(20,21),(0,5),(22,23),(0,9),(0,10),(25,28),(26,27),(0,6),(0,10),(0,10)];


/**
Translates EAN-13 (and UPC-A) barcode from BarcodeBarArray format to array of u8.
Does checksum validation, returns None if barcode is invalid;
**/
pub fn translate_bar_code(bcode: &BarcodeBarArray) -> Option<[u8;13]>{

    let mut barcode = [0 as u8;13];

    let mut even_odd : [bool;12] = [false;12];
    let mut n = 1;
    let mut valid = true;

    for c in bcode.1.iter(){
        let num = find_number_from_bars(&c);
        barcode[n] = num.0;
        even_odd[n] = num.1;
        if barcode[n] > 9 {
            valid = false;
            break;
        }
        n += 1;
    }
    if !valid {
        return None;
    }
    for c in bcode.2.iter(){
        let num = find_number_from_bars(&c);
        barcode[n] = num.0;
        even_odd[n] = num.1;
        if barcode[n] > 9 {
            valid = false;
            break;
        }
        if even_odd[n] == false {
            valid = false;
            break;
        }
        n += 1;
    }
    if valid {
        let first = find_first_number(&even_odd[2..7]);
        let check = calc_checksum(first,&barcode[1..12]);
        if check as u8 == barcode[11]{
            println!("{:?} {:?}, {:?}",first,barcode,check);
            barcode[0] = first as u8;
            Some(barcode)
        }

    }
    None
}


fn find_first_number(parity : &[bool]) -> usize{
    let mut d = 0;
    let mut cur_ix = 0;
    while d < 6 {
        let elem = EAN_PARITY[cur_ix];
        if elem.0 == 0{
            return elem.1;
        } else {
            let b = parity[d];
            if b {
                cur_ix = elem.0;
            } else {
                cur_ix = elem.1;
            }
        }
        d += 1;
    }
    return 10
}

fn calc_checksum(first:usize,nums: &[u8]) -> u8{
    let mut sum = first;
    let mut b = true;
    for n in nums.iter(){
        if b {
            sum += *n as usize * 3;
        } else {
            sum += *n as usize;
        }
        b = !b;
    }
    let remv = sum % 10;
    if remv == 0{
        return remv as u8;
    }
    return (10 - remv) as u8;
}


fn find_number_from_bars(c : &[u8;4]) -> (u8,bool){
    if c[0] == 0 || c[1] == 0 || c[2] == 0 || c[3] == 0{
        return (10,false);
    }
    let numbers = &BARCODE_DICT[BARCODE_DICT_POS[c[0] as usize-1][c[1] as usize-1] .. BARCODE_DICT_POS[c[0] as usize-1][c[1] as usize]];
    for dc in numbers{
        if dc.1[2] == c[2] && dc.1[3] == c[3] {
            return (dc.0,dc.2);
        }
    }
    return (10,false);
}