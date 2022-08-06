
fn screen_int_to_float(x: &i32, y: &i32, screen_width: &i32, screen_height: &i32, view_width: &f64, view_height: &f64) -> (f64, f64) {
    //let swf = screen_width as f32;
    //let shf, xf, yf = (screen_height as f32, x as f32, y as f32);
    return ((*x as f64)*view_width/(*screen_width as f64), (*y as f64)*view_height/(*screen_height as f64));
}



fn sample_mandelbrot(c: (f64, f64), iter_limit: i32, escape_radius: f64) -> i32 {
    let mut z = (0.0_f64, 0.0_f64);
    for i in 0..iter_limit {
        z = (z.0*z.0-(z.1*z.1), 2f64*z.0*z.1);
        z = (z.0+c.0, z.1+c.1);
        if (z.0*z.0+z.1*z.1) > escape_radius*escape_radius {
            return i;
        }
    }
    return iter_limit;
} 


const ITER_LIMIT: i32 = 128;
const ESCAPE_RADIUS: f64 = 4.0_f64;
const _PALETTE_STR: &str = " .-+%#@";
const PALETTE_SIZE: i32 = _PALETTE_STR.len() as i32;
const SCREEN_SIZE: (i32, i32) = (128, 64);

fn main() {
    let greeting = String::from("hi");
    let hello_world_info: (&String, &String) = (&greeting, &String::from("world"));
    println!("{}, {}!", hello_world_info.0, hello_world_info.1);

 
    let PALETTE: String = String::from(_PALETTE_STR);
    let view_pos = (0.0_f64, 0.0_f64);
    let view_size = (4.0_f64, 4.0_f64);
    
    for y in 0..SCREEN_SIZE.1 {
        print!("{}: ", y);
        for x in 0..SCREEN_SIZE.0 {
            // let intensity = (y + 2*x) % 6;
            // let currChar: String = String::from(PALETTE.as_bytes()[]);
            let centerC = screen_int_to_float(&x, &y, &SCREEN_SIZE.0, &SCREEN_SIZE.1, &view_size.0, &view_size.1);
            let c = (centerC.0 + view_pos.0 - 0.5*view_size.0, centerC.1 + view_pos.1 - 0.5*view_size.1);
            let iterCount = sample_mandelbrot(c, ITER_LIMIT, ESCAPE_RADIUS);
            let intensity: i32 =  iterCount * PALETTE_SIZE / (ITER_LIMIT + 1);
            let currChar = PALETTE.chars().nth(intensity as usize).unwrap();
            print!("{}", currChar);
        }
        println!("");
    }

}

