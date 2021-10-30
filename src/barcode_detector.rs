use std::cmp::max;
use std::cmp::min;
use crate::BarcodeBarArray;


/**Implement PixelValue for the image data source.
    x - pixel position on vertical axis
    y - pixel position on horizontal axis
    channel - color channel. red, green on blue.
    w - width of the image
**/
pub trait PixelValue {
    fn get_pixel_value(&self, x: u32, y:u32, channel: usize, w:usize) -> u8;
}

/**
Detects and parses barcode(s) from images.
Parameters:
img - object with type that has implemented PixelValue trait.
dim - width and height of the image
color_channel - color channel number that is provided to get_pixel_value()
**/
pub fn process_image_by_rows(img: &dyn PixelValue, dim: (u32,u32), color_channel: usize) -> Vec<BarcodeBarArray> {
    let step = calculate_row_step(dim.1);
    println!("Dimensions {} X {} STEP: {}", dim.0, dim.1, step);
    let mut y = 0;

    let row_slice_size = max(30, max(dim.0, dim.1) / 30) as usize;
    let big_image = row_slice_size > 40;

    let mut found_bar_codes : Vec<BarcodeBarArray> = Vec::new();
    let mut partial_bar_codes : Vec<BarcodeBarArray> = Vec::new();

    while y < dim.1 {
        let len = dim.0;
        let mut line_sum = 0;
        let mut line = ColorLine {
                avg: 0,
                min: 255,
                max: 0,
                deg: 0,
                pos: y,
                len,
                values: vec![0 as u8; len as usize],
                avg_loc : Vec::with_capacity((len as usize /row_slice_size) + 1),
                min_loc : Vec::with_capacity((len as usize/row_slice_size) + 1),
                max_loc : Vec::with_capacity((len as usize/row_slice_size) + 1),
                slice_size: row_slice_size
            };
        let mut slc = 0;
        let mut slice_vals = (255,0,0);
        for x in 0..len {
            let px_code = img.get_pixel_value(x, y, color_channel,len as usize);
            line.values[x as usize] = px_code;
            slice_vals.2 += px_code as usize;
            line_sum += px_code as usize;
            if slice_vals.0 > px_code {
                slice_vals.0 = px_code;
                if line.min > px_code {
                    line.min = px_code;
                }
            }
            if slice_vals.1 < px_code {
                slice_vals.1 = px_code;
                if line.max < px_code {
                    line.max = px_code;
                }
            }

            slc += 1;
            if slc >= row_slice_size {
                line.min_loc.push(slice_vals.0);
                line.max_loc.push(slice_vals.1);
                line.avg_loc.push((slice_vals.2 / slc as usize) as u8);
                slice_vals = (255,0,0);
                slc = 0;
            }

        }
        if slc > 0 {
            line.min_loc.push(slice_vals.0);
            line.max_loc.push(slice_vals.1);
            line.avg_loc.push((slice_vals.2 / slc as usize) as u8);
        }

        line.avg = (line_sum / (len) as usize) as u8;
        let a = find_crossings_from_average(&line, big_image);
        let bar_code = find_bar_code(&line,&a);
        if bar_code.0[3] > 0 {
            //if full bar code
            if bar_code.0[4] == 2 {
                let mut add = true;
                if found_bar_codes.len() > 0 {
                    add = !are_barcodes_same(&found_bar_codes.last().unwrap(), &bar_code);
                }
                if add {
                    found_bar_codes.push(bar_code);
                }
            } else {
                if partial_bar_codes.len() > 0{
                    let find_full = check_partial_bar_code(&mut partial_bar_codes, &found_bar_codes, &bar_code);
                    match find_full {
                        Some(code) =>  {
                            found_bar_codes.push(code);
                        },
                        None => {}
                    };
                }
                partial_bar_codes.push(bar_code);
            }
        }
        y += step;
    }

    return found_bar_codes;
}


fn check_partial_bar_code(partial_bar_codes : &mut Vec<BarcodeBarArray>, full_codes : &Vec<BarcodeBarArray>,bar_code : &BarcodeBarArray) -> Option<BarcodeBarArray>{
    let prev = partial_bar_codes.last().unwrap();
    let len = (bar_code.0[3] - bar_code.0[2]) / 2;
    let st =  bar_code.0[0] - prev.0[0];
    if len >= st{
        if prev.0[2] > bar_code.0[2]{
            if prev.0[2] > bar_code.0[2] + len && prev.0[2] < bar_code.0[3] + len {
                let new = (bar_code.0.clone(),bar_code.1.clone(),prev.1.clone());
                return Some(new);
            }
        } else {
            if bar_code.0[2] > prev.0[2] + len && bar_code.0[2] < prev.0[3] + len {
                let new = (bar_code.0.clone(),prev.1.clone(),bar_code.1.clone());
                return Some(new);
            }
        }
    } else {
        partial_bar_codes.truncate(0);
    }
    if  full_codes.len() > 0 {
        let prev = full_codes.last().unwrap();
        if prev.0[0] >= st{
            // Partial barcode starts before the full code.
            if prev.0[2] > bar_code.0[2]{
                if prev.0[2] < bar_code.0[2] + len && prev.0[2] < bar_code.0[3] + len {
                    let new = (bar_code.0.clone(),bar_code.1.clone(),prev.1.clone());
                    return Some(new);
                }
            } else {
                let middle = prev.0[2]/2 + prev.0[3]/2;
                // Partial barcode starts after the full code and before the middle part of full code.
                if middle > bar_code.0[2]{
                    return if middle < bar_code.0[2] + len {
                        let new = (bar_code.0.clone(), prev.1.clone(), bar_code.1.clone());
                        Some(new)
                    } else {
                        let new = (bar_code.0.clone(), bar_code.1.clone(), prev.2.clone());
                        Some(new)
                    }
                } else {
                    if middle + len > bar_code.0[2] {
                        let new = (bar_code.0.clone(), prev.1.clone(), bar_code.1.clone());
                        return Some(new);
                    }
                }
            }
        }
    }
    None
}


fn calculate_row_step(y: u32) -> u32{
    let lnst = (y as f64).log10();
    let step = (lnst * 6.0) as u32;
    return step;
}

fn are_barcodes_same(a : &BarcodeBarArray, b : &BarcodeBarArray) -> bool{
    let mut i = 0;
    while i < a.2.len() {
        let mut j = 0;
        while j < 4 {
            if a.2[i][j] != b.2[i][j] {
                return false;
            }
            j += 1;
        }
        i += 1
    }
    true
}

fn find_bar_code(color_line: &ColorLine, avg_cross : &(bool,Vec<usize>)) -> BarcodeBarArray{

    let mut bar_code_widths : BarcodeBarArray = ([0;5],[[0;4];6],[[0;4];6]);
    let mut partial_barcodes : Vec<BarcodeBarArray> = Vec::new();
    let c_len = avg_cross.1.len();
    if c_len >= 32{
        let mut diffs : Vec<usize> = Vec::with_capacity(c_len-1);
        let mut f = 0;
        let mut t = 1;
        while t < c_len{
            let diff = avg_cross.1[t] - avg_cross.1[f];
            diffs.push(diff);
            t += 1;
            f += 1;
        }
        let mut light = avg_cross.0;
        for t in 2..(c_len-31) {
            light = !light;
            let f = t-2;
            let rangechange = (diffs[f] as f32 * 0.12) as usize + 2;
            let range = (max(rangechange+1,diffs[f])-rangechange,diffs[f]+rangechange);
            let mut rangem = range.clone();
            if rangechange > 2 {
                rangem.1 += 2;
            }
            if diffs[t] >= rangem.0 && diffs[t] <=  rangem.1{
                if diffs[f+1] >= range.0 && diffs[f+1] <=  range.1{
                    let m_e = has_bar_code_middle_and_end(&diffs,t,&range);
                    if m_e.0 {
                        if m_e.1 {
                            let ulen = find_unit_len(avg_cross.1[f], avg_cross.1[t + 1], &color_line.values, light);
                            bar_code_widths.1 = parse_barcode_section(t + 1, &diffs, ulen, color_line, avg_cross);
                            if bar_code_widths.1[5][0] == 0 {
                                continue;
                            }
                            bar_code_widths.2 = parse_barcode_section(t + 30, &diffs, ulen, color_line, avg_cross);
                            if bar_code_widths.2[5][0] == 0 {
                                continue;
                            }
                            bar_code_widths.0 = [color_line.pos as usize, color_line.deg as usize, avg_cross.1[f], avg_cross.1[t + 55],2];
                            return bar_code_widths;
                        } else {
                            bar_code_widths = ([0;5],[[0;4];6],[[0;4];6]);
                            let mut pos = t;
                            if m_e.2 {
                                pos = t+1;
                            }
                            let ulen = find_unit_len(avg_cross.1[f], avg_cross.1[t + 1], &color_line.values, light);
                            let part = parse_barcode_section(pos + 1, &diffs, ulen, color_line, avg_cross);
                            if part[5][0] == 0 {
                                continue;
                            }
                            bar_code_widths.1 = part;
                            bar_code_widths.0 = [color_line.pos as usize, color_line.deg as usize, avg_cross.1[pos], avg_cross.1[t + 29],1];
                            partial_barcodes.push(bar_code_widths);
                        }
                    }
                }
            }
        }
    }
    if partial_barcodes.len() > 1 {
        match combine_row_barcode_parts(partial_barcodes) {
            Some(code) => {
                bar_code_widths = code;
            }
            _ => {}
        }
    }
    return bar_code_widths;
}

fn combine_row_barcode_parts(partial_barcodes: Vec<BarcodeBarArray>) -> Option<BarcodeBarArray>{
    let mut sc_ix= 1;
    for (pos, partial1) in partial_barcodes.iter().enumerate() {
        if sc_ix <= pos {
            sc_ix = pos + 1;
        }
        for ix2 in sc_ix..partial_barcodes.len() {
            let partial2 = partial_barcodes[ix2];
            if partial1.0[3] == partial2.0[2] {
                let meta = [partial1.0[0], partial1.0[1], partial1.0[2], partial2.0[3], 2];
                return Some((meta, partial1.1.clone(),partial2.1.clone()));
            }
        }
    }
    return None;
}


fn has_bar_code_middle_and_end(diffs: &Vec<usize>,t: usize, range: &(usize,usize)) -> (bool,bool,bool) {
    let middle = &diffs[t+25..t+30];
    let mut has_middle = true;
    let first = middle.first().unwrap();
    for m in 1..middle.len()-1{
        let elem = middle[m];
        if elem < range.0 || elem > range.1{
            has_middle = false;
            break;
        }
    }
    let elem = middle.last().unwrap();
    let has_sides = *elem >= range.0 && *elem <= range.1 && *first >= range.0 && *first <= range.1;
    if has_middle && !has_sides {
        return (has_middle, false, true);
    }

    if !has_middle || diffs.len() < t+57{
        return (has_middle,false,false);
    }
    let end = &diffs[t+54..t+57];
    let mut has_end = true;
    for m in end{
        if *m < range.0 || *m > range.1{
            has_end = false;
        }
    }
    return (has_middle,has_end, false);
}

fn parse_barcode_section(mut r:usize, diffs: &Vec<usize>, ulen:f32, color_line: &ColorLine, avg_cross : &(bool,Vec<usize>)) -> [[u8; 4]; 6] {
    let section_end = r+24;
    let mut ix= 0;
    let mut ret_codes = [[0;4];6];
    while r < section_end {
        let n = r+4;
        let s_ix = avg_cross.1[r];
        let e_ix = avg_cross.1[n+1];
        let avg_ix = s_ix/color_line.slice_size;
        let codes = parse_ean_code(&diffs[r..n], ulen, &color_line.values[s_ix..e_ix], color_line.avg_loc[avg_ix]);
        if codes.len() == 0 {
            break;
        }
        let mut i= 0;
        while i < codes.len(){
            ret_codes[ix][i] = codes[i];
            i += 1;
        }
        r = n;
        ix +=1;
    }
    return ret_codes;
}

fn parse_ean_code(lens: &[usize], unit: f32, vals: &[u8], avg_col: u8) -> Vec<u8>{
    return parse_number_bars(&lens, unit, vals, avg_col, 7, 4);
}

fn parse_number_bars(lens: &[usize], unit: f32, vals: &[u8], avg_col: u8, units: u8, max_len: u8) -> Vec<u8>{
    let len = lens.len();
    let mut divs= Vec::with_capacity(len);
    let mut fracs = vec![(0,-1.0,&[] as &[u8]);len];
    let mut total = 0;
    let mut num = 0;
    let mut lix = 0;
    for n in lens{
        let cur_vals = &vals[lix..lix+*n];
        let mut parts = *n as f32 / unit;
        if parts < 1.0 {
            parts = 1.0;
        } else {
            if *n > 4 {
                let edges = check_bar_edge(&cur_vals, avg_col);
                if edges.0 < 0.05 {
                    parts = parts-0.1;
                }
                if edges.1 < 0.05 {
                    parts = parts-0.1;
                }
            }
        }
        let int = parts.trunc() as u8;
        let frac = parts.fract();
        if int > max_len{
            return Vec::with_capacity(0);
        }
        divs.push(int);
        total += int;
        let mut l = num;
        while l>0{
            l-=1;
            if frac > fracs[l].1{
                fracs[l+1] = fracs[l];
            } else {
                if frac == fracs[l].1{
                    let clen = cur_vals.len();
                    let plen = fracs[l].2.len();
                    if clen == plen {
                        if compare_bar_by_color(cur_vals,fracs[l].2) > 0{
                            fracs[l+1] = fracs[l];
                            continue;
                        }
                    } else {
                        if clen > plen {
                            fracs[l+1] = fracs[l];
                            continue;
                        }
                    }
                }
                l += 1;
                break;
            }
        }
        fracs[l] = (num,frac,cur_vals);
        num += 1;
        lix += *n;
    }
    num = 0;
    while total < units{
        divs[fracs[num].0] += 1;
        total += 1;
        num += 1;
        if num >= len {
            num = 0;
        }
    }
    if total == units + 1{
        let lastf = fracs.last().unwrap();
        if divs[lastf.0] > 1 {
            if lastf.1 < 0.2{
                divs[lastf.0] -= 1;
                total -= 1;
            }
        }
    }
    if total > units{
        return Vec::with_capacity(0);
    }
    return divs;
}

fn check_bar_edge(cur_vals : &[u8],avg_col:u8) -> (f32,f32) {
    let df;
    let dt ;
    let px = cur_vals[0];
    if px < avg_col{
        let minval = cur_vals.iter().min().unwrap();
        let base = (avg_col-*minval) as f32;
        df = (avg_col- px) as f32 / base;
        dt = (avg_col as f32 - *cur_vals.last().unwrap() as f32)/ base;
    } else {
        let maxval = cur_vals.iter().max().unwrap();
        let base = (*maxval-avg_col) as f32;
        df = (px - avg_col) as f32 / base;
        dt = (*cur_vals.last().unwrap() as f32 - avg_col as f32) / base;
    }
    return (df,dt);
}


fn compare_bar_by_color(vals1: &[u8], vals2: &[u8]) -> i32{
    let len = vals1.len();
    let mut sums : (i32,i32) = (0,0);
    let mut i = 0;
    let mut extremes : (u8,u8) = (255,0);
    while i < len {
        if vals2[i] > vals1[i]{
            if vals2[i] > extremes.1 {
                extremes.1 = vals2[i];
            }
            if vals1[i] < extremes.0 {
                extremes.0 = vals1[i];
            }
        } else {
            if vals1[i] > extremes.1 {
                extremes.1 = vals1[i];
            }
            if vals2[i] < extremes.0 {
                extremes.0 = vals2[i];
            }
        }
        sums.0 += vals1[i] as i32;
        sums.1 += vals2[i] as i32;
        i += 1;
    }
    let ct : i32 = extremes.1 as i32 + extremes.0 as i32;
    if sums.1 > sums.0 {
        return ((sums.1 + sums.0) - len as i32*ct) * -1;
    } else {
        return (sums.1 + sums.0) - len as i32*ct;
    }
}


fn find_unit_len(start: usize, end: usize, row: &Vec<u8>,is_inverted:bool) -> f32{
    let nums = &row[start..end];
    let mut max = nums[0];
    let mut min = nums[0];
    for n in nums{
        if *n < min {
            min = *n;
        } else {
            if *n > max {
                max = *n;
            }
        }
    }
    let diff = (max - min) as f32;
    let mut sides: (f32,f32);
    if is_inverted {
        sides = ((max - nums[0]) as f32 / diff,(max - *nums.last().unwrap()) as f32 / diff);
        if start>0 && row[start-1] < max && end-start>6{
            sides.0 = sides.0 - (max.saturating_sub(row[start-1])) as f32 / diff;
        } else {
            sides.0 = sides.0/2.2;
            sides.1 = sides.1/2.2;
            if start>0 {
                if (row[start-1].saturating_sub(min)) as f32 / diff < 0.06 {
                    sides.0 = 0.0;
                }
            }
            if (row[end+1].saturating_sub(min)) as f32 / diff < 0.06 {
                sides.1 = 0.0;
            }
        }
    } else {
        sides = ((nums[0]-min) as f32 / diff,(nums.last().unwrap()-min) as f32 / diff);
        if start>0 && row[start-1] > min && end-start>6{
            sides.0 = sides.0 - (row[start-1].saturating_sub(min)) as f32 / diff;
        } else {
            sides.0 = sides.0/2.2;
            sides.1 = sides.1/2.2;
            if start>0 {
                if (row[start-1].saturating_sub(min)) as f32 / diff > 0.94 {
                    sides.0 = 0.0;
                }
            }
            if (row[end+1].saturating_sub(min)) as f32 / diff > 0.94 {
                sides.1 = 0.0;
            }
        }
    }
    let plen = (nums.len() as f32 - sides.0 - sides.1) / 3.0;
    return plen;
}


