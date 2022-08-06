

#![allow(non_snake_case)]
#![allow(dead_code)]


use std::path::Path;
use std::fs::File;
use std::io::BufWriter;


fn path_to_buffer_writer(path: &Path) -> BufWriter<File> {
    let file: File = File::create(path).unwrap();
    let w: BufWriter<File> = BufWriter::new(file);
    return w;
}

fn make_png_encoder(path: &Path, size: (u32, u32)) -> png::Encoder<BufWriter<File>> {
    let encoder = png::Encoder::new(path_to_buffer_writer(path), size.0, size.1);
    return encoder;
}


fn screen_int_to_float(int_coords: &(i32, i32), screen_size: &(i32, i32), view_size: &(f64, f64), view_corner_pos: &(f64, f64)) -> (f64, f64) {
    return ((int_coords.0 as f64)*view_size.0/((screen_size.0-1) as f64)+view_corner_pos.0, (int_coords.1 as f64)*view_size.1/((screen_size.1-1) as f64)+view_corner_pos.1);
}

fn screen_float_to_int(z: &(f64, f64), screen_size: &(i32, i32), view_size: &(f64, f64), view_corner_pos: &(f64, f64)) -> (i32, i32) {
    return (((z.0-view_corner_pos.0) * (screen_size.0 as f64) / view_size.0) as i32,  ((z.1-view_corner_pos.1) * (screen_size.1 as f64) / view_size.1) as i32);
}

fn coords_to_wrapped_vec_index(int_coords: (i32, i32), wrap_width: i32) -> i32 {
    return int_coords.1 * wrap_width + int_coords.0;
}



fn step_mandelbrot_point(z: (f64, f64), c: (f64, f64)) -> (f64, f64) {
    let zSquared = (z.0*z.0-(z.1*z.1), 2f64*z.0*z.1);
    return (zSquared.0+c.0, zSquared.1+c.1);
}
fn abs_squared(z: (f64, f64)) -> f64 {
    return z.0*z.0+z.1*z.1;
}

fn sample_mandelbrot(c: (f64, f64), iter_limit: i32, escape_radius: f64) -> i32 {
    let mut z = (0.0_f64, 0.0_f64);
    for i in 0..iter_limit {
        z = step_mandelbrot_point(z, c);
        if abs_squared(z) > escape_radius*escape_radius {
            return i;
        }
    }
    return iter_limit;
}

fn i32_is_bounded(value: i32, min: i32, max: i32) -> bool {
    return value.clamp(min, max) == value;
}

fn do_buddhabrot_point(c: (f64, f64), iter_limit: i32, escape_radius: f64, include_escaping: bool, include_nonescaping: bool, screen_data: &mut Vec<u8>, screen_size: &(i32, i32), view_size: &(f64, f64), view_corner_pos: &(f64, f64)) {
    let iterCount = sample_mandelbrot(c, iter_limit, escape_radius);
    let escaped: bool = iterCount < iter_limit;
    let shouldBeDrawn = (escaped && include_escaping) || ((!escaped) && include_nonescaping);

    //let mandelIndexInScreenData = coords_to_wrapped_vec_index(screenIntCoord, screen_size.0);
    //screen_data[mandelIndexInScreenData as usize] += iterCount;

    if shouldBeDrawn {
        let mut z = (0.0_f64, 0.0_f64);
        for i in 0..iter_limit {
            z = step_mandelbrot_point(z, c);
            if abs_squared(z) > escape_radius*escape_radius {
                return;
            }
            assert!(z.0.abs()*0.45_f64 <= view_size.0);
            assert!(z.1.abs()*0.45_f64 <= view_size.1);
            let mut screenIntCoord = screen_float_to_int(&z, screen_size, view_size, view_corner_pos);
            if (!i32_is_bounded(screenIntCoord.0, 0, screen_size.0-1)) || (!i32_is_bounded(screenIntCoord.1, 0, screen_size.1-1)) {
                // screenIntCoord = (screenIntCoord.0 % screen_size.0, screenIntCoord.1 % screen_size.1);
                panic!("invalid int coord reached: {:?} (from {:?} at iteration {}).", screenIntCoord, z, i);
            } 
            let indexInScreenData = coords_to_wrapped_vec_index(screenIntCoord, screen_size.0);
            if screen_data[indexInScreenData as usize] < 255 {
                screen_data[indexInScreenData as usize] += 1;
            }
        }
    }
}

fn itercount_to_intensity_index(itercount: i32, iterlimit: i32, intensity_limit: i32) -> i32 {
    /* intensity_limit is exclusive */
    return if itercount < iterlimit { itercount * intensity_limit / iterlimit } else { 0 };
}


const ITER_LIMIT: i32 = 1024;
const ESCAPE_RADIUS: f64 = 2.0_f64;
const _PALETTE_STR: &str = " .-+%#@";
const PALETTE_SIZE: i32 = _PALETTE_STR.len() as i32;

const BIDIRECTIONAL_SUPERSAMPLING: i32 = 4;
const SCREEN_SIZE: (i32, i32) = (1024, 1024);
const SEED_SCREEN_SIZE: (i32, i32) = (SCREEN_SIZE.0*BIDIRECTIONAL_SUPERSAMPLING, SCREEN_SIZE.1*BIDIRECTIONAL_SUPERSAMPLING);
const SCREEN_PIXEL_COUNT: usize = (SCREEN_SIZE.0*SCREEN_SIZE.1) as usize;



#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn screen_int_to_float_test() {
        let result = screen_int_to_float(&(0,0), &(512,512), &(4.0,4.0), &(-2.0, -2.0));
        if (result.0 + 2.0).abs() > 0.05 || (result.1 + 2.0).abs() > 0.5 {
            panic!("low output float coord is not correct: {:?}", result);
        }

        let result = screen_int_to_float(&(511,511), &(512,512), &(4.0,4.0), &(-2.0, -2.0));
        if (result.0 - 2.0).abs() > 0.05 || (result.1 - 2.0).abs() > 0.5 {
            panic!("high output float coord is not correct: {:?}", result);
        }
    }

    #[test]
    fn screen_float_to_int_test() {
        let result = screen_float_to_int(&(-1.99, -1.99), &(400,400), &(4.0,4.0), &(-2.0,-2.0));
        if result.0 > 2 || result.1 > 2 {
            panic!("low output int coord is not correct: {:?}", result);
        }
        let result = screen_float_to_int(&(1.99, 1.99), &(400,400), &(4.0,4.0), &(-2.0,-2.0));
        if result.0 < 397 || result.1 < 397 {
            panic!("high output int coord is not correct: {:?}", result);
        }
    }

}


fn main() {
    println!("started.");



    // let mut encoder = png::Encoder::new(path_to_buffer_writer(Path::new(r"./output/test.png")), 16, 16);
    let mut encoder = make_png_encoder(Path::new(r"./output/test8.png"), (SCREEN_SIZE.0 as u32, SCREEN_SIZE.1 as u32));
    encoder.set_color(png::ColorType::Grayscale);
    encoder.set_depth(png::BitDepth::Eight);
 
    let PALETTE: String = String::from(_PALETTE_STR);
    let view_pos = (0.0_f64, 0.0_f64);
    let view_size = (4.0_f64, 4.0_f64);
    let view_corner_pos = (view_pos.0 - 0.5*view_size.0, view_pos.1 - 0.5*view_size.1);

    let mut screen_data = vec![0_u8; SCREEN_PIXEL_COUNT]; //: Vec<[u8; SCREEN_PIXEL_COUNT]>





    
    for y in 0..SEED_SCREEN_SIZE.1 {
        for x in 0..SEED_SCREEN_SIZE.0 {
            // let intensity = (y + 2*x) % 6;
            // let currChar: String = String::from(PALETTE.as_bytes()[]);
            //let centerC = screen_int_to_float(&x, &y, &SCREEN_SIZE.0, &SCREEN_SIZE.1, &view_size.0, &view_size.1);
            //let c = (centerC.0 - view_corner_pos.0, centerC.1 - view_corner_pos.1);
            let c = screen_int_to_float(&(x,y), &SEED_SCREEN_SIZE, &view_size, &view_corner_pos);
            /*
            let iterCount = sample_mandelbrot(c, ITER_LIMIT, ESCAPE_RADIUS);
            let indexInScreenData = coords_to_wrapped_vec_index((x,y), SCREEN_SIZE.0);
            screen_data[indexInScreenData as usize] = itercount_to_intensity_index(iterCount, ITER_LIMIT, 256) as u8;
            */
            do_buddhabrot_point(c, ITER_LIMIT, ESCAPE_RADIUS, true, false, &mut screen_data, &SCREEN_SIZE, &view_size, &view_corner_pos);

            // screen_data = write_to_wrapped_vec(screen_data, SCREEN_SIZE.0, (x,y), itercount_to_intensity_index(iterCount, ITER_LIMIT, 256) as u8);

        }
    }

    let mut main_writer = encoder.write_header().unwrap();
    main_writer.write_image_data(&screen_data).unwrap();


}

