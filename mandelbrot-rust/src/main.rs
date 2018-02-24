extern crate num_complex;
extern crate sdl2;
extern crate threadpool;

use num_complex::Complex;
use sdl2::event::Event;
use sdl2::keyboard::Keycode;
use sdl2::pixels;
use std::ops::Index;
use std::sync::mpsc;
use threadpool::ThreadPool;

type Float = f64;

const THREADS: usize = 4;


fn main() {
    let mut view = View::new();
    let palette = Palette::new();
    let win_width = 1200u32;
    let win_height = (win_width as Float * view.aspect()) as u32;
    let max_iterations = 256;

    let sdl_context = sdl2::init().unwrap();
    let video_subsys = sdl_context.video().unwrap();
    let window = video_subsys.window("mandelbrot", win_width, win_height)
        .position_centered()
        .build()
        .unwrap();

    let event_sys = sdl_context.event().unwrap();
    event_sys.register_custom_event::<TextureUpdatedEvent>().unwrap();

    let mut canvas = window.into_canvas().build().unwrap();
    canvas.set_draw_color(pixels::Color::RGB(0, 0, 0));
    canvas.clear();
    canvas.present();

    let texture_creator = canvas.texture_creator();
    let mut texture = texture_creator.create_texture_static(
        pixels::PixelFormatEnum::RGB24, win_width, win_height).unwrap();

    let mut need_render = true;
    let mut need_draw = true;

    let mut events = sdl_context.event_pump().unwrap();
    loop {
        if need_render {
            render_mandelbrot(
                &event_sys, &mut texture,
                win_width, win_height, max_iterations,
                &view, &palette);
            need_draw = true;
            need_render = false;
        }
        if need_draw {
            canvas.clear();
            canvas.copy(&texture, None, None).unwrap();
            canvas.present();
            need_draw = false;
        }

        for event in events.poll_iter() {
            match event {
                Event::Quit {..}
                | Event::KeyDown {keycode: Some(Keycode::Escape), ..} => {
                    return;
                }

                Event::MouseButtonDown {x, y, ..} => {
                    let step_x = view.width() / (win_width as Float);
                    let step_y = view.height() / (win_height as Float);
                    let cx = view.left + (x as Float) * step_x;
                    let cy = view.top - (y as Float) * step_y;
                    view.zoom(cx, cy);
                    need_render = true;
                }

                Event::Window {..} => {
                    need_draw = true;
                }

                _ => {}
            }
        }
    }
}

struct TextureUpdatedEvent;

fn render_mandelbrot(
    event_sys: &sdl2::EventSubsystem, texture: &mut sdl2::render::Texture,
    win_width: u32, win_height: u32, max_iterations: u16,
    view: &View, palette: &Palette)
{
    println!("generating Mandelbrot escape-matrix");
    let escape = view.generate(
        win_width as usize, win_height as usize, max_iterations);

    println!("converting to texture");
    let mut pixels = Vec::new();
    for line in escape {
        for i in line {
            let color = palette[i as Float / max_iterations as Float];
            pixels.push(color.r);
            pixels.push(color.g);
            pixels.push(color.b);
        }
    }
    texture.update(None, &pixels, 3 * win_width as usize).unwrap();

    event_sys.push_custom_event(TextureUpdatedEvent).unwrap();
}


#[derive(Debug)]
struct View {
    left: Float,
    right: Float,
    top: Float,
    bottom: Float,
}

impl View {
    fn new() -> View {
        View {
            left: -2.5,
            right: 1.0,
            top: 1.5,
            bottom: -1.5,
        }
    }

    fn zoom(&mut self, x: Float, y: Float) {
        let width = self.width() / 4.0;
        let height = self.height() / 4.0;
        self.left = x - width;
        self.right = x + width;
        self.top = y + height;
        self.bottom = y - height;
    }

    fn width(&self) -> Float {
        self.right - self.left
    }

    fn height(&self) -> Float {
        self.top - self.bottom
    }

    fn aspect(&self) -> Float {
        self.height() / self.width()
    }

    fn generate(&self, img_width: usize, img_height: usize, max_iterations: u16) -> Vec<Vec<u16>>
    {
        let scalex = self.width() / img_width as Float;
        let scaley = self.height() / img_height as Float;

        let pool = ThreadPool::new(THREADS);
        let (tx, rx) = mpsc::channel();
        for y in 0..img_height {
            let tx = tx.clone();
            let cx = self.left;
            let cy = self.top - y as Float * scaley;
            pool.execute(move || {
                let line = View::generate_line(
                    img_width, max_iterations, cx, cy, scalex);
                tx.send((y, line)).unwrap();
            });
        }

        let mut result = rx.iter().take(img_height).collect::<Vec<_>>();
        result.sort();
        result.into_iter().map(|(_, escape)| escape).collect()
    }

    fn generate_line(img_width: usize, max_iterations: u16, mut cx: Float, cy: Float, step: Float) -> Vec<u16> {
        let mut escape = vec![0; img_width];

        for x in 0..img_width {
            let c = Complex::new(cx, cy);
            let mut z = Complex::new(cx, cy);

            for i in 0..max_iterations {
                if z.norm() >= 2.0 {
                    escape[x] = i;
                    break;
                }
                z = z * z + c;
            }

            cx = cx + step;
        }

        escape
    }
}


type Color = pixels::Color;

struct Palette {
    colors: Vec<Color>,
}

impl Palette {
    fn new() -> Palette {
        let mut colors = Vec::new();
        for i in 0..255 {
            let f = i as f64;
            let r = ((f / 255.0).sqrt() * 255.0) as u8;
            let g = r;
            let b = r;
            colors.push(pixels::Color::RGB(r, g, b));
        }
        Palette { colors: colors }
    }
}

impl Index<Float> for Palette {
    type Output = Color;
    fn index(&self, magnitude: Float) -> &Color {
        &self.colors[(self.colors.len() as Float * magnitude) as usize]
    }
}
