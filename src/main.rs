use image::{GenericImageView};
use std::time::Instant;
use std::cmp::max;
use std::cmp::min;
use std::env;

//Holds info about one pixel line in image
#[derive(Clone, Debug)]
struct ColorLine {
    avg: u8,
    min: u8,
    max: u8,
    deg: u16,
    len: u32,
    pos: u32,
    values: Vec<u8>,
    avg_loc: Vec<u8>,
    min_loc: Vec<u8>,
    max_loc: Vec<u8>,
    slice_size: usize
}

fn main() {
    let args: Vec<String> = env::args().collect();
    load_image(&args[1]);
}

fn load_image(filename: &String) {
    println!("Start: {:?}",Instant::now());
    let img = image::open(&filename).unwrap();
    println!("Image opened: {:?}",Instant::now());
    let dim = img.dimensions();
    let lnst = (dim.1 as f64).log10();
    let step = (lnst * 8.0) as u32;
    let mut by_color: Vec<Vec<ColorLine>> = vec![Vec::with_capacity((1 + dim.1 / step) as usize); 3];
    println!("Dimensions {} X {}", dim.0, dim.1);
    let mut y = 0;
    let row_slice_size = 30 as usize;
    while y < dim.1 {
        let len = dim.0;
        let mut extra_values = [((0, 0), (0, 0), 0 as usize); 3];
        let mut lines = vec![
            ColorLine {
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
            }; 3];
        let mut slc = 0;
        let mut slice_vals = [(255,0,0); 3];
        for x in 0..len {
            let px = img.get_pixel(x, y);
            for i in 0..3 {
                lines[i].values[x as usize] = px.0[i];
                slice_vals[i].2 += px.0[i] as usize;
                extra_values[i].2 += px.0[i] as usize;
                if slice_vals[i].0 > px.0[i] {
                    slice_vals[i].0 = px.0[i];
                    if lines[i].min > px.0[i] {
                        lines[i].min = px.0[i];
                    }
                }
                if slice_vals[i].1 < px.0[i] {
                    slice_vals[i].1 = px.0[i];
                    if lines[i].max < px.0[i] {
                        lines[i].max = px.0[i];
                    }
                }
            }
            slc += 1;
            if slc >= row_slice_size {
                for i in 0..3 {
                    lines[i].min_loc.push(slice_vals[i].0);
                    lines[i].max_loc.push(slice_vals[i].1);
                    lines[i].avg_loc.push((slice_vals[i].2 / slc as usize) as u8);
                }
                slice_vals = [(255,0,0); 3];
                slc = 0;
            }

        }
        if slc > 0 {
            for i in 0..3 {
                lines[i].min_loc.push(slice_vals[i].0);
                lines[i].max_loc.push(slice_vals[i].1);
                lines[i].avg_loc.push((slice_vals[i].2 / slc as usize) as u8);
            }
        }
        for i in (0..3).rev() {
            lines[i].avg = (extra_values[i].2 / (len) as usize) as u8;
            by_color[i].push(lines.remove(i));
        }
        y += step;
    }
    for color_line in &by_color[0] {
        let a = find_crossings_from_average(&color_line);
        let bar_code = find_bar_code(&color_line,&a);
        if bar_code.0[1] > 0 {
            println!("{:?} {:?}",color_line.pos, color_line.deg);
            println!("{:?}",bar_code);
        }
    }

    println!("Image processed: {:?}",Instant::now());
}

fn find_crossings_from_average(v: &ColorLine) -> (bool,Vec<usize>){

    let mut c_arr: (bool,Vec<usize>) = (true,Vec::new());
    let mut cur= true;
    let mut cur_loc = 0;
    let cur_stat = (v.min_loc[cur_loc],v.max_loc[cur_loc],v.avg_loc[cur_loc]);
    if v.values[0] < cur_stat.2 {
        c_arr.0 = false;
        cur = true;
    }
    let buf = max(3,((cur_stat.1 -cur_stat.0) as f32 * 0.04) as u8);
    let mut range = [cur_stat.2-buf,cur_stat.2+buf];

    let mut num = 0;
    let mut slc = v.slice_size/2;
    for col in &v.values{
        if cur == true {
            if *col < range[0] {
                cur = false;
                c_arr.1.push(num)
            }
        } else {
            if *col > range[1] {
                cur = true;
                c_arr.1.push(num)
            }
        }
        num += 1;
        slc += 1;
        if slc >= v.slice_size {
            range = find_range_buffer(cur_loc,v);
            cur_loc += 1;
            slc = 0;
        }
    }
    return c_arr;
}

fn find_range_buffer(cur: usize,v: &ColorLine) -> [u8;2]{
    let next = cur + 1;
    let len = v.max_loc.len();
    let avg : u8;
    let mx : u8;
    let mn : u8;
    if next < len{
        mx = max(v.max_loc[cur],v.max_loc[next]);
        mn = min(v.min_loc[cur],v.min_loc[next]);

        if next != len-1 {
            avg = v.avg_loc[cur] / 2 + v.avg_loc[next] / 2;
        } else {
            let rem = v.values.len() % v.slice_size;
            avg = ((v.avg_loc[cur] as usize * v.slice_size + v.avg_loc[next] as usize * rem) / (v.slice_size + rem)) as u8;
        }
    } else {
        mn = v.min_loc[cur];
        mx = v.max_loc[cur];
        avg = v.avg_loc[cur];
    }
    let buf = max(3,((mx -mn) as f32 * 0.04) as u8);
    let range = [avg-buf,avg+buf];
    return range;
}

fn find_bar_code(color_line: &ColorLine, avg_cross : &(bool,Vec<usize>)) -> ([usize;2],[[u8;4];6],[[u8;4];6]){

    let mut bar_code_widths : ([usize;2],[[u8;4];6],[[u8;4];6]) = ([0,0],[[0;4];6],[[0;4];6]);
    if avg_cross.1.len() >= 58{
        let mut diffs : Vec<usize> = Vec::with_capacity(avg_cross.1.len()-1);
        let mut f = 0;
        let mut t = 1;
        while t < avg_cross.1.len(){
            let diff = avg_cross.1[t] - avg_cross.1[f];
            diffs.push(diff);
            t += 1;
            f += 1;
        }
        let mut light = avg_cross.0;
        for t in 2..(avg_cross.1.len()-58) {
            light = !light;
            let f = t-2;
            let rangechange = (diffs[f] as f32 * 0.1) as usize + 2;
            let range = (max(rangechange+1,diffs[f])-rangechange,diffs[f]+rangechange);
            if diffs[t] >= range.0 && diffs[t] <=  range.1{
                if diffs[f+1] >= range.0 && diffs[f+1] <=  range.1{
                    if has_bar_code_middle_and_end(&diffs,t,&range) {
                        let ulen = find_unit_len(avg_cross.1[f], avg_cross.1[t+1], &color_line.values,light);
                        bar_code_widths.1 = parse_barcode_section(t+1, &diffs, ulen, color_line, avg_cross);
                        if bar_code_widths.1[5][0] == 0 {
                            continue;
                        }
                        bar_code_widths.2 = parse_barcode_section(t+30, &diffs, ulen, color_line, avg_cross);
                        if bar_code_widths.2[5][0] == 0 {
                            continue;
                        }
                        bar_code_widths.0 = [avg_cross.1[f],avg_cross.1[t+55]];
                        return bar_code_widths;
                    }
                }
            }
        }
    }
    return bar_code_widths;
}

fn has_bar_code_middle_and_end(diffs: &Vec<usize>,t: usize, range: &(usize,usize)) -> bool {
    let middle = &diffs[t+25..t+30];
    let mut has_middle = true;
    for m in middle{
        if *m < range.0 || *m > range.1{
            has_middle = false;
        }
    }
    let end = &diffs[t+54..t+57];
    let mut has_end = true;
    for m in end{
        if *m < range.0 || *m > range.1{
            has_end = false;
        }
    }
    return has_middle && has_end;
}

//Parse left and right sides
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

// Parse one number
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

//Find length of one one unit bar (one number in EAN format contains 7 units).
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
            sides.0 = sides.0 - (max - row[start-1]) as f32 / diff;
        } else {
            sides.0 = sides.0/2.2;
            sides.1 = sides.1/2.2;
            if start>0 {
                if (row[start-1]-min) as f32 / diff < 0.06 {
                    sides.0 = 0.0;
                }
            }
            if (row[end+1]-min) as f32 / diff < 0.06 {
                sides.1 = 0.0;
            }
        }
    } else {
        sides = ((nums[0]-min) as f32 / diff,(nums.last().unwrap()-min) as f32 / diff);
        if start>0 && row[start-1] > min && end-start>6{
            sides.0 = sides.0 - (row[start-1]-min) as f32 / diff;
        } else {
            sides.0 = sides.0/2.2;
            sides.1 = sides.1/2.2;
            if start>0 {
                if (row[start-1]-min) as f32 / diff > 0.94 {
                    sides.0 = 0.0;
                }
            }
            if (row[end+1]-min) as f32 / diff > 0.94 {
                sides.1 = 0.0;
            }
        }
    }
    let plen = (nums.len() as f32 - sides.0 - sides.1) / 3.0;
    return plen;
}