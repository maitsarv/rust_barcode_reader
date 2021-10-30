use std::cmp::max;
use std::cmp::min;

///Holds info about one pixel line in image
#[derive(Clone, Debug)]
pub struct ColorLine {
    pub avg: u8,
    pub min: u8,
    pub max: u8,
    pub deg: u16,
    pub len: u32,
    pub pos: u32,
    pub values: Vec<u8>,
    pub avg_loc: Vec<u8>,
    pub min_loc: Vec<u8>,
    pub max_loc: Vec<u8>,
    pub slice_size: usize
}

pub fn find_crossings_from_average(v: &ColorLine, big_image: bool) -> (bool, Vec<usize>){

    let mut c_arr: (bool,Vec<usize>) = (true,Vec::new());
    let mut cur= true;
    let mut cur_loc = 0;
    let cur_stat = (v.min_loc[cur_loc],v.max_loc[cur_loc],v.avg_loc[cur_loc] / 2 + (v.min_loc[cur_loc] / 2 + v.max_loc[cur_loc] / 2)/2);
    if v.values[0] < cur_stat.2 {
        c_arr.0 = false;
        cur = true;
    }
    let buf = max(3,((cur_stat.1 -cur_stat.0) as f32 * 0.04) as u8);
    let mut range = (cur_stat.2.saturating_sub(buf),cur_stat.2.saturating_add(buf),0);

    let mut slc = v.slice_size/2;
    let mut num = 0;
    if v.max_loc[cur_loc] - v.min_loc[cur_loc] < 16 {
        range = find_range_buffer(cur_loc,v);
        cur_loc = range.2;
        num = slc + v.slice_size * (cur_loc-1);
        slc = 0;
    }


    let mut buf_buffer = (0,cur);
    let mut col;
    while num < v.values.len(){
        col = &v.values[num];
        if cur == true {
            if *col < range.0 {
                cur = false;
                if big_image {
                    c_arr.1.push(num - buf_buffer.0 / 2);
                } else {
                    c_arr.1.push(num);
                }
                buf_buffer = (0,false);
            } else {
                compare_row_value_with_buffer(*col,range.1, num, &mut buf_buffer, &mut c_arr);
            }
        } else {
            if *col > range.1 {
                cur = true;
                if big_image {
                    c_arr.1.push(num - buf_buffer.0 / 2);
                } else {
                    c_arr.1.push(num);
                }
                buf_buffer = (0,true);
            } else {
                compare_row_value_with_buffer(range.0,*col, num, &mut buf_buffer, &mut c_arr);
            }
        }
        num += 1;
        slc += 1;
        if slc >= v.slice_size {
            range = find_range_buffer(cur_loc,v);
            let diff = range.2 - cur_loc;
            if diff > 1 {
                num = num + v.slice_size * diff;
            }
            cur_loc = range.2;
            slc = 0;
        }
    }
    return c_arr;
}

fn compare_row_value_with_buffer(val1:u8,val2:u8, num : usize,buf_buffer : &mut (usize,bool),c_arr : &mut (bool,Vec<usize>)){
    if val1 <= val2 {
        buf_buffer.0 += 1;
    } else {
        if buf_buffer.1 == true && buf_buffer.0 > 3 {
            c_arr.1.push(num - buf_buffer.0);
            c_arr.1.push(num);
        }
        buf_buffer.0 = 0;
        buf_buffer.1 = true;
    }
}

fn find_range_buffer(mut cur: usize,v: &ColorLine) -> (u8,u8,usize){
    let mut next = cur+1;
    let len = v.max_loc.len();
    let mut avg : u8 = 0;
    let mut mx : u8 = 0;
    let mut mn : u8 = 0;
    while next < len{
        mx = max(v.max_loc[cur],v.max_loc[next]);
        mn = min(v.min_loc[cur],v.min_loc[next]);
        if mx - mn < 16 {
            cur += 1;
            next = cur + 1;
            continue;
        }
        if next < len-2 {
            avg = (v.avg_loc[cur] / 3) + (v.avg_loc[next] / 3) + (v.avg_loc[next+1] / 3);
        } else if next != len-1 {
            avg = (v.avg_loc[cur] / 2) + (v.avg_loc[next] / 2);
        } else {
            let rem = v.values.len() % v.slice_size;
            avg = ((v.avg_loc[cur] as usize * v.slice_size + v.avg_loc[next] as usize * rem) / (v.slice_size + rem)) as u8;
        }
        break;
    }
    if next == len {
        mn = v.min_loc[cur];
        mx = v.max_loc[cur];
        avg = v.avg_loc[cur];
    }
    avg = ((avg as u32 + (mx as u32 + mn as u32)/2) / 2) as u8;

    let buf = max(3,((mx -mn) as f32 * 0.04) as u8);
    let range = (avg.saturating_sub(buf),avg.saturating_add(buf),next);
    return range;
}